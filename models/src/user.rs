use std::collections::HashMap;
use bytes::{Bytes, BytesMut};
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};
use futures::{SinkExt, StreamExt};
use tokio_util::codec::{Framed, BytesCodec};

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub msg_frame: Framed<TcpStream, BytesCodec>,
}

impl User {
    pub fn new(name: String, msg_frame: Framed<TcpStream, BytesCodec>) -> Self {
        Self { name, msg_frame }
    }

    pub async fn send(&mut self, content: Bytes) -> io::Result<()> {
        self.msg_frame.send(content).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> io::Result<BytesMut> {
        let content = self.msg_frame.next().await;
        match content {
            Some(Ok(msg)) => Ok(msg),
            _ => Err(io::Error::from(io::ErrorKind::ConnectionAborted))
        }
    }
}

#[derive(Debug)]
pub struct OnlineUsers {
    pub list: HashMap<String, User>,
}

impl OnlineUsers {
    pub fn new() -> Self {
        let list = HashMap::new();
        Self { list }
    }

    pub async fn kick(&mut self, name: &str) -> io::Result<()> {
        let user = self.list.remove(name);
        match user {
            Some(mut user) => {
                let stream = user.msg_frame.get_mut();
                stream.shutdown().await?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "attempt to kick an offline user.",
            )),
        }
    }

    pub async fn debug(&self) {
        for (key, val) in self.list.iter() {
            println!("{:?} -> {:?}", key, val);
        }
    }
}
