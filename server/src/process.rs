use bytes::Bytes;
use futures::SinkExt;
use models::{user::{OnlineUsers, User}, command::Command, msg_codec::MsgCodec, message::Content};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

pub async fn process(
    stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<Mutex<OnlineUsers>>,
) -> Result<(), Box<dyn Error>> {
    let (rd, wt) = stream.into_split();
    let mut rd_frame = FramedRead::new(rd, MsgCodec::new());
    let mut wt_frame = FramedWrite::new(wt, BytesCodec::new());
    wt_frame.send(Bytes::from("username: ")).await?;
    let username = request_username(&mut rd_frame, addr).await?;
    println!("{} has joined server.", username);
    push_user(Arc::clone(&online_users), username.clone(), wt_frame).await;

    loop {
        match rd_frame.next().await {
            Some(Ok(msg)) => {
                println!("receive");
                println!("{:?}", msg);
                match msg.command {
                    Command::Help => handle_help(Arc::clone(&online_users), &username).await.unwrap(),
                    _ => ()
                }
            },
            _ => {
                println!("disconnect");
                online_users.lock().await.kick(&username).await?;
                break;
            }
        }
    }

    Ok(())
}

async fn handle_help(online_users: Arc<Mutex<OnlineUsers>>, username: &str) -> io::Result<()> {
    let mut online_users = online_users.lock().await;
    let _ = online_users.send_to_user(username, Command::help().into()).await;
    Ok(())
}

async fn request_username(
    frame: &mut FramedRead<OwnedReadHalf, MsgCodec>,
    addr: SocketAddr,
) -> Result<String, Box<dyn Error>> {
    let username = match frame.next().await {
        Some(Ok(input)) => input,
        _ => {
            tracing::error!("Failed to get username from {}. Client disconnected.", addr);
            return Err(Box::new(io::Error::from(io::ErrorKind::ConnectionAborted)));
        }
    };
    if let Content::Text(username) = username.content {
        return Ok(username);
    } else {
        return Err(Box::new(io::Error::from(io::ErrorKind::InvalidData)));
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
