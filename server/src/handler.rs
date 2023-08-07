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
use tokio::{net::tcp::ReadHalf, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

type Rx = mpsc::UnboundedReceiver<Message>;

pub async fn login(
    online_users: Arc<OnlineUsers>,
    rd_frame: &mut FramedRead<ReadHalf<'_>, MsgCodec>,
    addr: SocketAddr,
) -> Result<(String, Rx)> {
    if let Some(Ok(msg)) = rd_frame.next().await {
        match msg.command {
            Command::Login => {
                if let Content::Text(name) = msg.content {
                    let (tx, rx) = mpsc::unbounded_channel();
                    online_users.add_user(&name, tx).await;
                    tracing::info!("{} has joined server", name);
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
            Error::Offline(user) => tracing::warn!("interact with offline user {}", user),
            Error::Config => tracing::error!("failed to read config"),
            Error::ServerToClient => tracing::warn!("lost one pack from server to client"),
            Error::Disconnect => tracing::info!("user disconnect"),
            Error::Channel => tracing::warn!("channel does not work properly"),
            Error::Unreachable => tracing::warn!("unexpected logical error"),
            Error::RequestFormat => tracing::info!("received a request in wrong format"),
            Error::InvalidMessage => tracing::warn!("broken message"),
            Error::Listen(port) => tracing::error!("failed to bind TcpListener to port {}", port),
            Error::RwLock => tracing::error!("read write lock error"),
            Error::ClientToServer => tracing::error!("this should be a client error"),
            Error::FilePath(_) => tracing::error!("file path"),
            Error::ConnectionFail(_) => tracing::error!("this should be a client error"),
        },
    }
}
