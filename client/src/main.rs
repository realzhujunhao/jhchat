mod handler;
mod init;
mod process;
mod request;

use request::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (addr, _config) = init::config()
        .and_then(init::directory)
        .and_then(init::socket_addr)
        .unwrap();

    let (rd, wt) = connect(addr).await?;

    process::listen_server(rd);
    process::listen_stdin(wt).await?;

    Ok(())
}
