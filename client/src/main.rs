use bytes::Bytes;
use futures::{future, Sink, Stream, StreamExt, SinkExt};
use std::{env, error::Error, net::SocketAddr};
use tokio::{io, net::TcpStream};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let addr = args.first().ok_or("need addr as argument")?;
    let addr: SocketAddr = addr.parse()?;

    let stdin = FramedRead::new(io::stdin(), BytesCodec::new());
    let stdin = stdin.map(|i| i.map(|bytes| bytes.freeze()));
    let stdout = FramedWrite::new(io::stdout(), BytesCodec::new());

    connect(&addr, stdin, stdout).await?;

    Ok(())
}

async fn connect(
    addr: &SocketAddr,
    mut stdin: impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
    mut stdout: impl Sink<Bytes, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(addr).await?;
    let (rd, wt) = stream.split();
    let mut sink = FramedWrite::new(wt, BytesCodec::new());
    let mut stream = FramedRead::new(rd, BytesCodec::new()).filter_map(|i| match i {
        Ok(i) => future::ready(Some(i.freeze())),
        Err(e) => {
            println!("failed to read from socket; error = {}", e);
            future::ready(None)
        }
    }).map(Ok);

    match future::join(sink.send_all(&mut stdin), stdout.send_all(&mut stream)).await {
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
        _ => Ok(()),
    }
}













