mod request;

use bytes::BytesMut;
use models::codec::command::Command;
use request::*;
use tokio_stream::StreamExt;
use std::{env, error::Error};
use std::net::ToSocketAddrs;
use tokio::{io::{self, AsyncBufReadExt}, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let addr = args.first().ok_or("need addr as argument")?;
    let addr = addr.to_socket_addrs()?.next().ok_or("Address resolution error")?;


    let (mut rd, mut wt) = connect(addr).await?;

    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin);
    let mut line = String::new();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = rd.next().await {
            match msg.command {
                Command::Help => println!("{:?}", msg), 
                Command::SendMsg => println!("{:?}", msg),
                Command::SendImage => println!("{:?}", msg),
                Command::OnlineList => println!("{:?}", msg),
                Command::Login => println!("{:?}", msg),
            }
        }
    });

    loop {
        line.clear();
        reader.read_line(&mut line).await?;
        let args: Vec<String> = line.split_whitespace().map(|s| s.into()).collect();
        match args[0].as_str() {
            "login" => login(&mut wt, &args[1]).await?,
            "list" => list(&mut wt).await?,
            "text" => send_text(&mut wt, &args[1], &args[2]).await?,
            "image" => {
                let to = &args[1];
                let path = &args[2]; 
                let content = fs::read(path).await?;
                let filename = path.rsplit('/').next().unwrap_or(path);
                send_image(&mut wt, to, filename, BytesMut::from(content.as_slice())).await?
            }
            "exit" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}
