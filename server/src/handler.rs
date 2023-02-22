use bytes::Bytes;
use futures::SinkExt;
use models::user::{OnlineUsers, User};
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

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<Mutex<OnlineUsers>>,
) -> Result<(), Box<dyn Error>> {
    let (rd, wt) = stream.into_split();
    let mut rd_frame = FramedRead::new(rd, BytesCodec::new());
    let mut wt_frame = FramedWrite::new(wt, BytesCodec::new());
    wt_frame.send(Bytes::from("username: ")).await?;
    let username = request_username(&mut rd_frame, addr).await?;
    push_user(Arc::clone(&online_users), username.clone(), wt_frame).await;
    loop {
        match rd_frame.next().await {
            Some(Ok(msg)) => {
                let content = String::from_utf8(msg.to_vec())?.trim().to_string();
                println!("{}", content);
                online_users.lock().await.debug().await;
            },
            _ => {
                online_users.lock().await.kick(&username).await?;
                break;
            }
        }
    }

    Ok(())
}

async fn request_username(
    frame: &mut FramedRead<OwnedReadHalf, BytesCodec>,
    addr: SocketAddr,
) -> Result<String, Box<dyn Error>> {
    let username = match frame.next().await {
        Some(Ok(input)) => input,
        _ => {
            tracing::error!("Failed to get username from {}. Client disconnected.", addr);
            return Err(Box::new(io::Error::from(io::ErrorKind::ConnectionAborted)));
        }
    };
    let username = String::from_utf8(username.to_vec())?.trim().to_string();
    Ok(username)
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
