use bytes::BytesMut;
use std::str::FromStr;

use crate::codec::message::{Content, FileData, Message};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::AsRefStr, strum::EnumString)]
pub enum Command {
    Help,
    Login,
    OnlineList,
    SendMsg,
    SendImage,
    GetRSA,
    SendRSA,
    RemoteError,
}

impl From<BytesMut> for Command {
    fn from(value: BytesMut) -> Self {
        Self::from_str(&String::from_utf8_lossy(&value)).unwrap_or(Self::Help)
    }
}

impl From<Command> for BytesMut {
    fn from(value: Command) -> Self {
        BytesMut::from(value.as_ref())
    }
}

impl Command {
    pub fn help() -> Message {
        let content_text = String::from("");
        Message::send_text("", &content_text).set_sender("Server")
    }

    pub fn content(&self) -> Content {
        match self {
            Self::SendImage => Content::File(FileData::default()),
            _ => Content::Text(String::default()),
        }
    }
}
