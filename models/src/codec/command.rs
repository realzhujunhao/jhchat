use bytes::BytesMut;
use strum::{IntoStaticStr, EnumString};
use std::str::FromStr;

use crate::codec::message::{Content, Message, FileData};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IntoStaticStr, EnumString)]
pub enum Command {
    Help,
    Login,
    OnlineList,
    SendMsg,
    SendImage,
}

impl From<Command> for BytesMut {
    fn from(value: Command) -> Self {
        BytesMut::from(Into::<&str>::into(value))
    }
}

impl From<BytesMut> for Command {
    fn from(value: BytesMut) -> Self {
        Self::from_str(&String::from_utf8_lossy(&value)).unwrap_or(Self::Help)
    }
}

impl Command {
    pub fn help() -> Message {
        let content_text = format!(
            "\n{}\n{}\n{}\n",
            r"OnlineList#0,,|$ -> request online user list",
            r"SendMsg#{length},{receiver},|{msg}$  ->  send msg",
            r"SendImage#{length},{receiver},{filename}|{bytes}$  ->  send image",
        );
        Message::send_text("", &content_text).set_sender("Server")
    }

    pub fn content(&self) -> Content {
        match self {
            Self::SendImage => Content::File(FileData::default()),
            _ => Content::Text(String::default()),
        }
    }
}
