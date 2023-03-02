mod init;
mod process;
mod handler;
use std::{error::Error, sync::Arc, env};

use process::process;
use models::server_state::OnlineUsers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init::trace();
    env::set_var("RUST_BACKTRACE", "full");

    let config = init::config()?;
    init::file_structure(&config.file_dir);
    let online_users = Arc::new(OnlineUsers::new());
    let listener = init::connection(&config.ip, &config.port).await?;

    loop {
        let online_users = Arc::clone(&online_users);
        let (stream, addr) = listener.accept().await?;
        let file_dir = config.file_dir.clone();

        tokio::spawn(async move {
            let result = process(stream, addr, online_users, file_dir).await;
            handler::error(result);
        });
    }
}
