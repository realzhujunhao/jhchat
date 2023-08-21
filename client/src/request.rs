use bytes::BytesMut;
use futures::SinkExt;
use models::{
    codec::{message::Message, msg_codec::MsgCodec},
    error::GlobalResult,
};
use std::net::SocketAddr;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use colored::*;

type Reader = FramedRead<OwnedReadHalf, MsgCodec>;
type Writer = FramedWrite<OwnedWriteHalf, MsgCodec>;

pub async fn connect(addr: SocketAddr) -> GlobalResult<(Reader, Writer)> {
    println!("{} {}", "connecting to".green(), addr.to_string().yellow());
    let stream = TcpStream::connect(addr)
        .await?;
    println!("{}", "connection established".green());
    let (rd, wt) = stream.into_split();

    Ok((
        FramedRead::new(rd, MsgCodec::new()),
        FramedWrite::new(wt, MsgCodec::new()),
    ))
}

pub async fn login(writer: &mut Writer, uid: &str) -> GlobalResult<()> {
    let msg = Message::login(uid);
    writer.send(msg).await?;
    Ok(())
}

pub async fn list(writer: &mut Writer) -> GlobalResult<()> {
    let msg = Message::online_list("");
    writer.send(msg).await?;
    Ok(())
}

pub async fn send_text(writer: &mut Writer, to: &str, content: &str) -> GlobalResult<()> {
    let msg = Message::send_text(to, content);
    writer.send(msg).await?;
    Ok(())
}

pub async fn send_image(
    writer: &mut Writer,
    to: &str,
    filename: &str,
    content: BytesMut,
) -> GlobalResult<()> {
    let msg = Message::send_image(to, filename, content);
    writer.send(msg).await?;
    Ok(())
}
