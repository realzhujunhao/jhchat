use futures::SinkExt;
use models::{
    command::Command,
    error::{Error, Result},
    message::{Content, Message},
    msg_codec::MsgCodec,
    server_state::OnlineUsers,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::tcp::{ReadHalf, WriteHalf},
    sync::{mpsc, Mutex},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn help(online_users: Arc<Mutex<OnlineUsers>>, from: &str) -> Result<()> {
    let mut msg = Command::help();
    msg.args[0] = from.to_string();
    send_msg(online_users, from, msg).await?;
    Ok(())
}

pub async fn online_list(online_users: Arc<Mutex<OnlineUsers>>, from: &str) -> Result<()> {
    let temp = Arc::clone(&online_users);
    let temp = temp.lock().await;
    let mut msg = temp.msg_list();
    drop(temp);
    msg.args[0] = from.to_string();
    send_msg(online_users, from, msg).await?;
    Ok(())
}

pub async fn recv_msg(
    msg: Message,
    wt_frame: &mut FramedWrite<WriteHalf<'_>, MsgCodec>,
) -> Result<()> {
    wt_frame
        .send(msg)
        .await
        .map_err(|_| Error::ServerToClient)?;
    Ok(())
}

pub async fn send_msg(
    online_users: Arc<Mutex<OnlineUsers>>,
    from: &str,
    msg: Message,
) -> Result<()> {
    let mut online_users = online_users.lock().await;
    online_users.send_from(from, msg).await?;
    Ok(())
}

type Tx = mpsc::Sender<Message>;
type Rx = mpsc::Receiver<Message>;

pub async fn login(
    online_users: Arc<Mutex<OnlineUsers>>,
    rd_frame: &mut FramedRead<ReadHalf<'_>, MsgCodec>,
    addr: SocketAddr,
) -> Result<(String, Rx)> {
    if let Some(Ok(msg)) = rd_frame.next().await {
        match msg.command {
            Command::Login => {
                if let Content::Text(name) = msg.content {
                    let (tx, rx) = mpsc::channel(128);
                    push_user(online_users, name.clone(), tx).await;
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

pub async fn disconnect(online_users: Arc<Mutex<OnlineUsers>>, name: &str) -> Result<()> {
    let mut online_users = online_users.lock().await;
    online_users.kick(name)?;
    Ok(())
}

pub fn error(err: Result<()>) {
    match err {
        Ok(()) => (),
        Err(e) => match e {
            Error::Offline => tracing::info!("attempt to interact with offline user"),
            Error::Config => tracing::warn!("failed to read config."),
            Error::ServerToClient => tracing::warn!("lost one pack from server to client."),
            Error::Disconnect => tracing::info!("user disconnect."),
            Error::Channel => tracing::warn!("channel does not work properly."),
            Error::Listen => tracing::warn!("unable to listen to specified port."),
            Error::Unreachable => tracing::warn!("unexpected logical error."),
            Error::RequestFormat => tracing::info!("receive a request of wrong format."),
            Error::InvalidMessage => tracing::warn!("broken message."),
        },
    }
}

async fn push_user(online_users: Arc<Mutex<OnlineUsers>>, username: String, tx: Tx) {
    let mut online_users = online_users.lock().await;
    online_users.list.insert(username.clone(), tx);
}
