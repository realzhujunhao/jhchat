use bytes::BytesMut;
use futures::SinkExt;
use models::{
    codec::{message::Message, msg_codec::MsgCodec},
    error::{Error, Result},
};
use std::net::SocketAddr;
use tokio::net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}};
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

pub async fn login(writer: &mut Writer, uid: &str) -> Result<()> {
    let msg = Message::login(uid);
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}

pub async fn list(writer: &mut Writer) -> Result<()> {
    let msg = Message::online_list("");
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}

pub async fn send_text(writer: &mut Writer, to: &str, content: &str) -> Result<()> {
    let msg = Message::send_text(to, content);
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}

pub async fn send_image(
    writer: &mut Writer,
    to: &str,
    filename: &str,
    content: BytesMut,
) -> Result<()> {
    let msg = Message::send_image(to, filename, content);
    writer.send(msg).await.map_err(|_| Error::ClientToServer)?;
    Ok(())
}
