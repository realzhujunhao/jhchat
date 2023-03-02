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
    SendBytes,
    FileKey,
}


impl Into<BytesMut> for Command {
    fn into(self) -> BytesMut {
        BytesMut::from(Into::<&str>::into(self))
    }
}

impl From<BytesMut> for Command {
    fn from(value: BytesMut) -> Self {
        Self::from_str(&String::from_utf8_lossy(&value).to_string()).unwrap_or(Self::Help)
    }
}

impl Command {
    pub fn help() -> Message {
        let content_text = format!(
            "\n{}\n{}\n{}\n{}\n",
            r"OnlineList#0,,|$  ->  Request name list of online users",
            r"SendMsgToUser#{length},{receiver},|{msg}$  ->  send msg to the specified user",
            r"SendFileToUser#{length},{receiver},{filename}|{bytes}$  ->  send file to the specified user",
            r"AcceptFile#,{receiver},|{key}$  ->  accept file",
        );
        Message::send_text("", &content_text).set_sender("Server")
    }

    pub fn content(&self) -> Content {
        match self {
            Self::SendImage => Content::File(FileData::default()),
            Self::SendBytes => Content::File(FileData::default()),
            _ => Content::Text(String::default()),
        }
    }
}
