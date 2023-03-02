use models::{
    codec::{
        command::Command,
        message::{Content, Message},
        msg_codec::MsgCodec,
    },
    error::{Error, Result},
    server_state::OnlineUsers,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::tcp::ReadHalf,
    sync::mpsc,
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

type Rx = mpsc::Receiver<Message>;

pub async fn login(
    online_users: Arc<OnlineUsers>,
    rd_frame: &mut FramedRead<ReadHalf<'_>, MsgCodec>,
    addr: SocketAddr,
) -> Result<(String, Rx)> {
    if let Some(Ok(msg)) = rd_frame.next().await {
        match msg.command {
            Command::Login => {
                if let Content::Text(name) = msg.content {
                    let (tx, rx) = mpsc::channel(128);
                    online_users.add_user(&name, tx).await;
                    tracing::info!("{} has joined server.", name);
                    Ok((name, rx))
                } else {
                    Err(Error::InvalidMessage)
                }
            }
            _ => Err(Error::InvalidMessage),
        }
    } else {
        tracing::error!("Failed to get username from {}. Client disconnected.", addr);
        Err(Error::Disconnect)
    }
}

pub fn error(err: Result<()>) {
    match err {
        Ok(()) => (),
        Err(e) => match e {
            Error::Offline(user) => {
                tracing::info!("attempt to interact with offline user {}", user)
            }
            Error::Config => tracing::warn!("failed to read config."),
            Error::ServerToClient => tracing::warn!("lost one pack from server to client."),
            Error::Disconnect => tracing::info!("user disconnect."),
            Error::Channel => tracing::warn!("channel does not work properly."),
            Error::Unreachable => tracing::warn!("unexpected logical error."),
            Error::RequestFormat => tracing::info!("receive a request of wrong format."),
            Error::InvalidMessage => tracing::warn!("broken message."),
            Error::Listen(port) => println!("failed to bind TcpListener to port {}", port),
            Error::RwLock => tracing::warn!("Read Write Lock Error, online_user might not be secure"),
            _ => unreachable!()
        },
    }
}

