use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use models::user::{OnlineUsers, User};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex, io};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, BytesCodec};

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<Mutex<OnlineUsers>>,
) -> Result<(), Box<dyn Error>> {
    let mut frame = Framed::new(stream, BytesCodec::new());
    let username = request_username(&mut frame, addr).await?;
    push_user(online_users, username, frame).await;
    Ok(())
}

async fn request_username(frame: &mut Framed<TcpStream, BytesCodec>, addr: SocketAddr) -> Result<String, Box<dyn Error>> {
    frame.send(Bytes::from("username: ")).await?;
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

async fn push_user(online_users: Arc<Mutex<OnlineUsers>>, username: String, frame: Framed<TcpStream, BytesCodec>) {
    let user = User::new(username.clone(), frame);
    let mut online_users = online_users.lock().await;
    online_users.list.insert(username, user);
}
