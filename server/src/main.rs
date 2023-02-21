mod init;
mod handler;
use std::{error::Error, sync::Arc};

use handler::handle_connection;
use tokio::sync::Mutex;
use models::user::OnlineUsers;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init::trace();
    let online_users = Arc::new(Mutex::new(OnlineUsers::new()));
    let listener = init::connection().await?;

    loop {
        let online_users = Arc::clone(&online_users);
        let (stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(stream, addr, online_users).await;
        });
    }
}
