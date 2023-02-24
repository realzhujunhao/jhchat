use bytes::{BytesMut, BufMut};
use crate::command::Command;

#[derive(Debug)]
pub enum Content {
    Text(String),
    Bytes(BytesMut),
}

impl Into<BytesMut> for Content {
    fn into(self) -> BytesMut {
        match self {
            Self::Text(text) => BytesMut::from(text.as_bytes()),
            Self::Bytes(bytes) => BytesMut::from(bytes),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub command: Command,
    pub args: Vec<String>,
    pub content: Content,
}

impl Into<BytesMut> for Message {
    fn into(self) -> BytesMut {
        let mut bytes = BytesMut::new();
        let args = self.args.join(",");

        let cmd_bytes: BytesMut = self.command.into();
        let args_bytes = BytesMut::from(args.as_bytes());
        let content_bytes: BytesMut = self.content.into();
        
        bytes.reserve(cmd_bytes.len() + args_bytes.len() + content_bytes.len() + 3);

        bytes.put(cmd_bytes);
        bytes.put("#".as_bytes());
        bytes.put(args_bytes);
        bytes.put("|".as_bytes());
        bytes.put(content_bytes);
        bytes.put("$".as_bytes());
        bytes
    }
}

