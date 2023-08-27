use core::{
    codec::{
        command::Command,
        message::Message,
        msg_codec::MsgCodec,
    },
    error::{ErrorType, ExternalError, GlobalResult, ServerError},
    server_state::OnlineUsers,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

type Rx = mpsc::UnboundedReceiver<Message>;

pub async fn authenticate(
    online_users: Arc<OnlineUsers>,
    rd_frame: &mut FramedRead<OwnedReadHalf, MsgCodec>,
    addr: SocketAddr,
) -> GlobalResult<(String, Rx)> {
    // 1. get next frame
    match rd_frame.next().await {
        // 1.1 a frame is deserialized
        //     2. check `Command`
        Some(Ok(msg)) => match msg.command {
            // 2.1 `Command` is `Login`
            Command::Login => {
                let uid = msg.sender;
                let (tx, rx) = mpsc::unbounded_channel();
                online_users.add_user(&uid, tx).await;
                tracing::info!("{} has joined server", uid);
                Ok((uid, rx))
            }
            // 2.2 `Command` is NOT `Login`
            _ => {
                tracing::warn!("{} command is not login during authentication", addr);
                Err(ServerError::UnexpectedFrame.into())
            }
        },
        // 1.2 deserialization failed
        _ => {
            tracing::warn!("cannot deserialize tokens received from {}", addr);
            Err(ExternalError::DeserializeFrame.into())
        }
    }
}

pub fn record(result: GlobalResult<()>) {
    use ErrorType::*;
    match result {
        Ok(()) => (),
        Err(e) => match e.err {
            Client(_) => (),
            Server(e) => {
                tracing::warn!(
                    "{}",
                    &format!("recover() dropped connection due to {}", e.as_ref())
                );
            }
            External(e) => {
                tracing::warn!(
                    "{}",
                    &format!("recover() dropped connection due to {}", e.as_ref())
                );
            }
        },
    }
}
