use bytes::BytesMut;
use futures::SinkExt;
use models::{
    error::{Error, Result},
    msg_codec::MsgCodec, message::Message,
};
use std::net::SocketAddr;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio_util::codec::{FramedRead, FramedWrite};

type Reader = FramedRead<OwnedReadHalf, MsgCodec>;
type Writer = FramedWrite<OwnedWriteHalf, MsgCodec>;

pub async fn connect(addr: SocketAddr) -> Result<(Reader, Writer)> {
    let stream = TcpStream::connect(addr)
        .await
        .map_err(|_| Error::ConnectionFail(addr))?;
    let (rd, wt) = stream.into_split();
    Ok((
        FramedRead::new(rd, MsgCodec::new()),
        FramedWrite::new(wt, MsgCodec::new()),
    ))
}

pub async fn send_text(writer: &mut Writer, to: &str, content: &str) -> Result<()> {
    let msg = Message::send_text(to, content);
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}

pub async fn send_file(writer: &mut Writer, to: &str, content: BytesMut) -> Result<()> {
    let msg = Message::send_file(to, content);
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}

pub async fn send_image(writer: &mut Writer, to: &str, content: BytesMut) -> Result<()> {
    let msg = Message::send_image(to, content);
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}
