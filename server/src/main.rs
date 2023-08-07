mod init;
mod process; mod handler;
use std::{error::Error, sync::Arc};

use process::process;
use models::server_state::OnlineUsers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _guard = init::trace();

    let config = init::config()?;
    let online_users = Arc::new(OnlineUsers::new());
    let listener = init::listen(&config.ip, &config.port).await?;

    tracing::info!("this is a test log");

    loop {
        let online_users = Arc::clone(&online_users);
        let (stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            let result = process(stream, addr, online_users).await;
            handler::error(result);
        });
    }

}
