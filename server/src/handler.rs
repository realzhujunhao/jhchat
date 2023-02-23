use models::{
    command::Command,
    message::{Content, Message},
    msg_codec::MsgCodec,
    user::{OnlineUsers, User},
};
use std::{error::Error, io, net::SocketAddr, sync::Arc};
use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};
// use bytes::Bytes;
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

pub async fn help(online_users: Arc<Mutex<OnlineUsers>>, username: &str) -> io::Result<()> {
    let mut online_users = online_users.lock().await;
    let _ = online_users
        .send_to_user(username, Command::help().into())
        .await;
    Ok(())
}

pub async fn online_list(online_users: Arc<Mutex<OnlineUsers>>, username: &str) -> io::Result<()> {
    let online_users = online_users.lock().await;
    let name_list = online_users.list();

    Ok(())
} 

// pub async fn send_msg(online_users: Arc<Mutex<OnlineUsers>>, msg: &Message, from: &str) -> io::Result<()> {
//     let target_user = msg.args.get(0).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
//     if let Content::Text(ref text) = msg.content {
//         let text_owned = text.clone();
//         let text_byte = Bytes::from(text_owned.as_bytes());
//         let mut online_users = online_users.lock().await;
//         online_users.send_to_user(target_user, text_byte).await; 
//     }
//     Ok(())
// }

pub async fn login(
    online_users: Arc<Mutex<OnlineUsers>>,
    rd_frame: &mut FramedRead<OwnedReadHalf, MsgCodec>,
    wt_frame: FramedWrite<OwnedWriteHalf, BytesCodec>,
    addr: SocketAddr,
) -> io::Result<String> {
    if let Some(Ok(msg)) = rd_frame.next().await {
        match msg.command {
            Command::Login => {
                if let Content::Text(name) = msg.content {
                    push_user(online_users, name.clone(), wt_frame).await;
                    tracing::info!("{} has joined server.", name);
                    Ok(name)
                } else {
                    Err(io::Error::from(io::ErrorKind::InvalidData))
                }
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    } else {
        tracing::error!("Failed to get username from {}. Client disconnected.", addr);
        Err(io::Error::from(io::ErrorKind::ConnectionAborted))
    }
}

async fn push_user(
    online_users: Arc<Mutex<OnlineUsers>>,
    username: String,
    frame: FramedWrite<OwnedWriteHalf, BytesCodec>,
) {
    let user = User::new(username.clone(), frame);
    let mut online_users = online_users.lock().await;
    online_users.list.insert(username.clone(), user);
}
