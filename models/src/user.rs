use bytes::Bytes;
use futures::SinkExt;
use std::collections::HashMap;
use tokio::{io::{self, AsyncWriteExt}, net::tcp::OwnedWriteHalf};
use tokio_util::codec::{BytesCodec, FramedWrite};

#[derive(Debug)]
pub struct User {
    pub name: String,
    //TODO OTHER INFO
    pub msg_frame: FramedWrite<OwnedWriteHalf, BytesCodec>,
}

impl User {
    pub fn new(name: String, msg_frame: FramedWrite<OwnedWriteHalf, BytesCodec>) -> Self {
        Self { name, msg_frame }
    }

    pub async fn send(&mut self, content: Bytes) -> io::Result<()> {
        self.msg_frame.send(content).await?;
        Ok(())
    }

//     pub async fn recv(&mut self) -> io::Result<BytesMut> {
//         let content = self.msg_frame.next().await;
//         match content {
//             Some(Ok(msg)) => Ok(msg),
//             _ => Err(io::Error::new(
//                 io::ErrorKind::ConnectionAborted,
//                 self.name.clone(),
//             )),
//         }
//     }
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

    pub async fn send_to_user(&mut self, name: &str, content: Bytes) -> io::Result<()> {
        let target_user = self
            .list
            .get_mut(name)
            .ok_or(io::Error::new(io::ErrorKind::NotConnected, name.clone()))?;
        target_user.send(content).await?;
        Ok(())
    }

    pub async fn kick(&mut self, name: &str) -> io::Result<()> {
        let user = self.list.remove(name);
        match user {
            Some(mut user) => {
                let stream = user.msg_frame.get_mut();
                let _ = stream.shutdown().await;
                Ok(())
            },
            None => Ok(())
        }
    }

    pub fn list(&self) -> Vec<String> {
        let name_vec: Vec<String> = self.list.keys().map(|s| s.clone()).collect();
        name_vec
    }

    pub async fn debug(&self) {
        for (key, val) in self.list.iter() {
            println!("{:?} -> {:?}", key, val);
        }
    }
}
