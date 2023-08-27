mod init;
mod worker;

use std::{error::Error, sync::Arc};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (addr, config) = init::config()
        .and_then(init::directory)
        .and_then(init::encrypt_key)
        .and_then(init::socket_addr)
        .unwrap();

    let (rd, mut wt) = worker::connect(addr).await?;

    let (tx, rx) = mpsc::unbounded_channel();
    let config = Arc::new(config);

    worker::authenticate(&mut wt, Arc::clone(&config)).await?;

    let write_task = worker::write_stream(wt, rx);

    let read_task = worker::read_stream(rd, tx.clone(), Arc::clone(&config));

    let stdin_task = worker::read_stdin(tx.clone(), Arc::clone(&config));

    let _ = tokio::join!(write_task, read_task, stdin_task);

    Ok(())
}
