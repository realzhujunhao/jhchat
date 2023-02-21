use std::collections::HashMap;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

use tokio_util::codec::{Framed, LinesCodec};

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub msg_frame: Framed<TcpStream, LinesCodec>,
}

impl User {
    pub fn new(name: String, msg_frame: Framed<TcpStream, LinesCodec>) -> Self {
        Self { name, msg_frame }
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
