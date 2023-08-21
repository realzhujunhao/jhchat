use crate::handler;
use futures::SinkExt;
use models::error::{GlobalResult, ServerError, ExternalError};
use models::{
    codec::{command::Command, message::Message, msg_codec::MsgCodec},
    server_state::OnlineUsers,
};
use tokio::sync::mpsc::unbounded_channel;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn process(
    stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<OnlineUsers>,
) -> GlobalResult<()> {
    let (rd, wt) = stream.into_split();
    let mut rd_frame = FramedRead::new(rd, MsgCodec::new());
    let mut wt_frame = FramedWrite::new(wt, MsgCodec::new());

    let (uid, mut rx) =
        handler::authenticate(Arc::clone(&online_users), &mut rd_frame, addr).await?;
    let uid_shared_1 = Arc::new(uid);
    let uid_shared_2 = Arc::clone(&uid_shared_1);

    let (e_tx, mut e_rx) = unbounded_channel();
    let e_tx_1 = e_tx.clone();
    let e_tx_2 = e_tx.clone();

    // task 1: peek the stream and handle frames
    tokio::spawn(async move {
        loop {
            let uid = Arc::clone(&uid_shared_1);
            let result = match rd_frame.next().await {
                Some(Ok(msg)) => handle_incoming_msg(msg, &uid, Arc::clone(&online_users)).await,
                _ => {
                    tracing::info!("user {} with ip {} has left the server", &uid, addr);
                    online_users.remove_user(&uid).await;
                    Err(ServerError::UserDisconnect.info(&uid))
                }
            };
            if let Err(e) = result {
                let _ = e_tx_1.send(e);
                break;
            }
        }
    });

    // task 2: send frames to client
    tokio::spawn(async move {
        loop {
            let uid = Arc::clone(&uid_shared_2);
            let result =
                match rx.recv().await {
                    Some(msg) => wt_frame
                        .send(msg)
                        .await
                        .map_err(|e| ExternalError::TokioChannel.info(&format!("{}", e))),
                    None => Err(ExternalError::TokioChannel
                        .info(&format!("{}'s transmitter is dropped", &uid))),
                };
            if let Err(e) = result {
                let _ = e_tx_2.send(e);
            }
        }
    });
    if let Some(error) = e_rx.recv().await {
        return Err(error);
    }
    Ok(())
}

async fn handle_incoming_msg(
    msg: Message,
    uid: &str,
    online_users: Arc<OnlineUsers>,
) -> GlobalResult<()> {
    tracing::info!("user {} has sent a message to server\n{:?}", uid, msg);
    match msg.command {
        Command::OnlineList => {
            online_users
                .send(uid, online_users.to_msg().await.set_sender("Server"))
                .await
        }
        Command::SendMsg => {
            online_users
                .send(&msg.get_receiver(), msg.set_sender(uid))
                .await
        }
        Command::SendImage => {
            online_users
                .send(&msg.get_receiver(), msg.set_sender(uid))
                .await
        }
        Command::Help => online_users.send(uid, Command::help()).await,
        Command::GetRSA => {
            online_users
                .send(&msg.get_receiver(), msg.set_sender(uid))
                .await
        }
        Command::SendRSA => {
            online_users
                .send(&msg.get_receiver(), msg.set_sender(uid))
                .await
        }
        Command::Login => Err(ServerError::UnexpectedFrame
            .info(&format!("{} duplicated authentication request", &uid))),
        Command::RemoteError => Err(ServerError::Unknown.into()),
    }
}
