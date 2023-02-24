use crate::handler;
use models::{command::Command, msg_codec::MsgCodec, server_state::OnlineUsers, message::Message};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};
use futures::SinkExt;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use models::error::{Error, Result};

pub async fn process(
    mut stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<Mutex<OnlineUsers>>,
) -> Result<()> {
    let (rd, wt) = stream.split();
    let mut rd_frame = FramedRead::new(rd, MsgCodec::new());
    let mut wt_frame = FramedWrite::new(wt, MsgCodec::new());
    wt_frame.send(Message::plain_text(Command::Login, "please login with username")).await.map_err(|_| Error::Disconnect)?;
    let (username, mut rx) = handler::login(Arc::clone(&online_users), &mut rd_frame, addr).await?;
    loop {
        tokio::select! {
            result = rd_frame.next() => {
                match result {
                    Some(Ok(msg)) => match msg.command {
                        Command::OnlineList => {
                            handler::online_list(Arc::clone(&online_users), &username).await?
                        }
                        Command::SendMsgToUser => {
                            handler::send_msg(Arc::clone(&online_users), &username, msg).await?
                        }
                        _ => handler::help(Arc::clone(&online_users), &username).await?,
                    },
                    _ => {
                        tracing::info!("user {} with ip {} has left the server.", username, addr);
                        break;
                    }
                }
            }
            Some(msg) = rx.recv() => {
                handler::recv_msg(msg, &mut wt_frame).await?;
            }
        }
    }

    Ok(())
}
