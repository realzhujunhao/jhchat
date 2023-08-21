use models::{
    codec::{
        command::Command,
        message::{Content, Message},
        msg_codec::MsgCodec,
    },
    error::{GlobalResult, ServerError, ErrorType, ExternalError},
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
            //     3. check `Content`
            Command::Login => match msg.content {
                // 3.1 correct format
                Content::Text(name) => {
                    let (tx, rx) = mpsc::unbounded_channel();
                    online_users.add_user(&name, tx).await;
                    tracing::info!("{} has joined server", name);
                    Ok((name, rx))
                }
                // 3.2 wrong format
                _ => {
                    tracing::warn!("{} command is login but content is not text", addr);
                    Err(ServerError::UnexpectedFrame.into())
                }
            },

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
        Err(e) => {
            match e.err {
                Client(_) => (),
                Server(e) => {
                    tracing::warn!("{}", &format!("recover() dropped connection due to {}", e.as_ref()));
                }
                External(e) => {
                    tracing::warn!("{}", &format!("recover() dropped connection due to {}", e.as_ref()));
                }
            }
        }
    }
}
