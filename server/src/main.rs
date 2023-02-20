mod init;
mod handler;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init::trace();
    let listener = init::connection().await?;

    loop {
        let (_stream, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            // TODO
        });
    }
}
