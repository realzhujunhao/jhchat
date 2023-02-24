mod init;
mod process;
mod handler;
use std::{error::Error, sync::Arc, env};

use process::process;
use tokio::sync::Mutex;
use models::server_state::OnlineUsers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init::trace();
    env::set_var("RUST_BACKTRACE", "1");
    let online_users = Arc::new(Mutex::new(OnlineUsers::new()));
    let listener = init::connection().await?;

    loop {
        let online_users = Arc::clone(&online_users);
        let (stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            process(stream, addr, online_users).await.unwrap();
        });
    }
}
