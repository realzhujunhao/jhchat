use std::error::Error;

use bytes::BytesMut;
use models::codec::{msg_codec::MsgCodec, command::Command};
use tokio::{net::tcp::{OwnedReadHalf, OwnedWriteHalf}, io::{self, AsyncBufReadExt}, fs};
use tokio_util::codec::{FramedRead, FramedWrite};

use tokio_stream::StreamExt;

use crate::request::{login, list, send_text, send_image};

type Reader = FramedRead<OwnedReadHalf, MsgCodec>;
type Writer = FramedWrite<OwnedWriteHalf, MsgCodec>;

pub fn listen_server(mut rd: Reader) {
    tokio::spawn(async move {
        while let Some(Ok(msg)) = rd.next().await {
            // TODO
            match msg.command {
                Command::Help => println!("{}", msg),
                Command::SendMsg => println!("{}", msg),
                Command::SendImage => println!("{}", msg),
                Command::OnlineList => println!("{}", msg),
                Command::Login => println!("{}", msg),
                Command::GetRSA => println!("{}", msg),
                Command::SendRSA => println!("{}", msg),
                Command::RemoteError => println!("{}", msg),
            }
        }
    });
}

pub async fn listen_stdin(mut wt: Writer) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line).await?;
        let args: Vec<String> = line.split_whitespace().map(|s| s.into()).collect();
        match args[0].as_str() {
            "login" => login(&mut wt, &args[1]).await?,
            "list" => list(&mut wt).await?,
            "text" => send_text(&mut wt, &args[1], &args[2]).await?,
            "image" => {
                let to = &args[1];
                let path = &args[2];
                let content = fs::read(path).await?;
                let filename = path.rsplit('/').next().unwrap_or(path);
                send_image(&mut wt, to, filename, BytesMut::from(content.as_slice())).await?
            }
            "exit" => break,
            _ => unreachable!(),
        }
    }
    Ok(())
}
