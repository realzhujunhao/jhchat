use std::{net::SocketAddr, path::Path, sync::Arc};

use colored::*;
use core::{
    codec::{command::Command, message::Message, msg_codec::MsgCodec},
    config::ClientConfig,
    error::{ClientError, GlobalResult},
    traits::encrypt::Encrypt,
};
use futures::SinkExt;
use tokio::{
    io::{self, AsyncBufReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tokio_util::codec::{FramedRead, FramedWrite};

use tokio_stream::StreamExt;

use crate::init::Encryptor;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

type Reader = FramedRead<OwnedReadHalf, MsgCodec>;
type Writer = FramedWrite<OwnedWriteHalf, MsgCodec>;

pub async fn authenticate(wt: &mut Writer, config: Arc<ClientConfig>) -> GlobalResult<()> {
    wt.send(Message::login(&config.uid)).await?;
    Ok(())
}

pub fn read_stream(
    mut rd: Reader,
    tx: UnboundedSender<Message>,
    config: Arc<ClientConfig>,
) -> tokio::task::JoinHandle<GlobalResult<()>> {
    tokio::spawn(async move {
        // read private key from local disk
        let priv_key =
            config.encryption.rsa_self_priv_key.as_ref().ok_or(
                ClientError::EncryptKeyPersistence.info("user's private key does not exist"),
            )?;
        let pub_key =
            config.encryption.rsa_self_pub_key.as_ref().ok_or(
                ClientError::EncryptKeyPersistence.info("user's public key does not exist"),
            )?;

        println!("{}", "polling the read stream".green());
        // poll read stream, deserialize message, then respond to command
        while let Some(Ok(msg)) = rd.next().await {
            match msg.command {
                // someone sends message to me -> decrypt & display
                Command::SendMsg => {
                    let message = Encryptor::decrypt_to_string(&msg.content, priv_key)?;
                    println!("{} {}: {}", "from".green(), &msg.sender.green(), message.green());
                }
                // someone requests for my public key -> notify write_stream
                Command::GetPubKey => {
                    println!("{}", "get pub key command received".yellow());
                    let content = Encryptor::export_pub_key(pub_key)?;
                    tx.send(Message::send_pub_key(&msg.sender, &content))?;
                }
                // receive someone's public key -> save to local disk & notify write_stream
                Command::SendPubKey => {
                    let pub_key_path =
                        Path::new(&config.encryption.unsafe_key_dir).join(&msg.sender);
                    let pub_key = Encryptor::import_pub_key(&msg.content)?;
                    Encryptor::async_persist_pub_key(pub_key_path, &pub_key).await?;
                }
                Command::OnlineList => {
                    println!("{}", String::from_utf8_lossy(&msg.content).to_string());
                }
                _ => println!("{:?}", msg),
            }
        }
        Ok(())
    })
}

pub fn write_stream(
    mut wt: Writer,
    mut rx: UnboundedReceiver<Message>,
) -> tokio::task::JoinHandle<GlobalResult<()>> {
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            wt.send(msg).await?;
        }
        Ok(())
    })
}

pub fn read_stdin(
    tx: UnboundedSender<Message>,
    config: Arc<ClientConfig>,
) -> tokio::task::JoinHandle<GlobalResult<()>> {
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            reader.read_line(&mut line).await?;
            let tokens: Vec<&str> = line.split_whitespace().collect();
            match tokens[0] {
                "list" => tx.send(Message::online_list(""))?,
                "send" => {
                    let receiver = tokens[1];
                    let receiver_key_path =
                        Path::new(&config.encryption.unsafe_key_dir).join(receiver);
                    tx.send(Message::get_pub_key(receiver))?;
                    if !receiver_key_path.is_file() {
                        println!(
                            "{}",
                            "receiver's key does not exist, requesting...".yellow()
                        );
                    }
                    while !receiver_key_path.is_file() {
                        tokio::task::yield_now().await;
                    }
                    println!(
                        "{} {:?}",
                        "receiver's key is saved at".green(),
                        receiver_key_path
                    );
                    let receiver_key = Encryptor::async_read_pub_key(receiver_key_path).await?;
                    let ciphertext = {
                        let mut rng = rand::thread_rng();
                        Encryptor::encrypt_from_str(tokens[2], &receiver_key, &mut rng)?
                    };
                    tx.send(Message::send_text(receiver, &ciphertext))?;
                }
                "exit" => break Ok(()),
                _ => (),
            }
        }
    })
}

pub async fn connect(addr: SocketAddr) -> GlobalResult<(Reader, Writer)> {
    println!("{} {}", "connecting to".green(), addr.to_string().yellow());
    let stream = TcpStream::connect(addr).await?;
    println!("{}", "connection established".green());
    let (rd, wt) = stream.into_split();

    Ok((
        FramedRead::new(rd, MsgCodec::new()),
        FramedWrite::new(wt, MsgCodec::new()),
    ))
}
