use crate::handler;
use futures::SinkExt;
use models::codec::message::Content;
use models::error::{Error, Result};
use models::{
    codec::{
        command::Command,
        message::Message,
        msg_codec::{CodecRole, MsgCodec},
    },
    server_state::OnlineUsers,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn process(
    mut stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<OnlineUsers>,
    file_dir: String,
) -> Result<()> {
    let (rd, wt) = stream.split();
    let mut rd_frame = FramedRead::new(rd, MsgCodec::new(CodecRole::Server, &file_dir));
    let mut wt_frame = FramedWrite::new(wt, MsgCodec::new(CodecRole::Server, &file_dir));
    wt_frame
        .send(Message {
            sender: "Server".into(),
            receiver: "N/A".into(),
            command: Command::Login,
            content: Content::Text("login with username\n".into()),
        })
        .await
        .map_err(|_| Error::Disconnect)?;
    let (username, mut rx) = handler::login(Arc::clone(&online_users), &mut rd_frame, addr).await?;
    loop {
        tokio::select! {
            result = rd_frame.next() => {
                let ok_error = match result {
                    Some(Ok(msg)) => match msg.command {
                        Command::OnlineList => {
                            online_users.send(&username, online_users.to_msg().await.set_sender("Server")).await
                        }
                        Command::SendMsg => {
                            online_users.send(&msg.get_receiver(), msg.set_sender(&username)).await
                        }
                        Command::FileKey => {
                            online_users.send(&msg.get_receiver(), msg.set_sender(&username)).await
                        }
                        Command::SendImage => {
                            online_users.send(&msg.get_receiver(), msg.set_sender(&username)).await
                        }
                        Command::Help => {
                            online_users.send(&username, Command::help()).await
                        }
                        // TODO accept file
                        _ => online_users.send(&username, Command::help()).await
                    },
                    _ => {
                        tracing::info!("user {} with ip {} has left the server.", username, addr);
                        online_users.remove_user(&username).await;
                        break;
                    }
                };
                handler::error(ok_error);
            }
            Some(msg) = rx.recv() => {
                handler::error(wt_frame.send(msg).await.map_err(|_| Error::ServerToClient))
            }
        }
    }
    Ok(())
}
