use crate::handler;
use futures::SinkExt;
use models::codec::message::Content;
use models::error::{Error, Result};
use models::{
    codec::{
        command::Command,
        message::Message,
        msg_codec::MsgCodec,
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
) -> Result<()> {
    let (rd, wt) = stream.split();
    let mut rd_frame = FramedRead::new(rd, MsgCodec::new());
    let mut wt_frame = FramedWrite::new(wt, MsgCodec::new());

    wt_frame
        .send(Message {
            sender: "Server".into(),
            receiver: "N/A".into(),
            command: Command::Login,
            content: Content::Text("please request with a login command\n".into()),
        })
        .await
        .map_err(|_| Error::Disconnect)?;

    let (uid, mut rx) = handler::login(Arc::clone(&online_users), &mut rd_frame, addr).await?;
    loop {
        tokio::select! {
            result = rd_frame.next() => {
                 match result {
                    Some(Ok(msg)) =>  {
                        handle_incoming_msg(msg, &uid, Arc::clone(&online_users)).await
                    },
                    _ => {
                        tracing::info!("user {} with ip {} has left the server.", uid, addr);
                        online_users.remove_user(&uid).await;
                        break;
                    }
                };
            }
            Some(msg) = rx.recv() => {
                handler::error(wt_frame.send(msg).await.map_err(|_| Error::ServerToClient))
            }
        }
    }
    Ok(())
}

#[tracing::instrument]
async fn handle_incoming_msg(msg: Message, uid: &str, online_users: Arc<OnlineUsers>) {
    let result = match msg.command {
        Command::OnlineList => online_users.send(uid, online_users.to_msg().await.set_sender("Server")).await,
        Command::SendMsg => online_users.send(&msg.get_receiver(), msg.set_sender(uid)).await,
        Command::SendImage => online_users.send(&msg.get_receiver(), msg.set_sender(uid)).await,
        Command::Help => online_users.send(uid, Command::help()).await,
        Command::Login => Err(Error::Unreachable),
    };
    handler::error(result);
}







