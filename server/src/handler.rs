use futures::SinkExt;
use models::user::{OnlineUsers, User};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<Mutex<OnlineUsers>>,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());
    lines.send("username:").await?;

    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            tracing::error!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    let user = User::new(username.clone(), lines);
    let mut online_users = online_users.lock().await;
    online_users.list.insert(username, user);
    online_users.debug().await;
    Ok(())
}
