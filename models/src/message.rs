use bytes::BytesMut;
use crate::command::Command;

#[derive(Debug)]
pub enum Content {
    Text(String),
    Bytes(BytesMut),
}

#[derive(Debug)]
pub struct Message {
    pub command: Command,
    pub args: Vec<String>,
    pub content: Content,
}

