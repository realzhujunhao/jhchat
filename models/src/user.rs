use std::collections::HashMap;
use tokio::{io::{self, AsyncWriteExt}, net::tcp::OwnedWriteHalf};
use tokio_util::codec::{BytesCodec, FramedWrite};

use crate::message::Message;

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

    pub async fn recv_from(&mut self, from: &User, msg: Message) -> io::Result<()> {

        // self.msg_frame.send().await?;
        Ok(())
    }
}

