use bytes::BytesMut;
use std::str::FromStr;

use crate::codec::message::{Content, Message, FileData};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Command {
    Login,
    OnlineList,
    SendMsgToUser,
    SendFileToUser,
    SendImageToUser,
    AcceptFile,
    Help,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OnlineList" => Ok(Self::OnlineList),
            "SendMsgToUser" => Ok(Self::SendMsgToUser),
            "SendFileToUser" => Ok(Self::SendMsgToUser),
            "Login" => Ok(Self::Login),
            "Help" => Ok(Self::Help),
            _ => Err(()),
        }
    }
}

impl Into<BytesMut> for Command {
    fn into(self) -> BytesMut {
        match self {
            Self::SendMsgToUser => BytesMut::from("SendMsgToUser"),
            Self::SendFileToUser => BytesMut::from("SendFileToUser"),
            Self::SendImageToUser => BytesMut::from("SendImageToUser"),
            Self::OnlineList => BytesMut::from("OnlineList"),
            Self::Help => BytesMut::from("Help"),
            Self::Login => BytesMut::from("Login"),
            Self::AcceptFile => BytesMut::from("AcceptFile"),
        }
    }
}

impl From<BytesMut> for Command {
    fn from(value: BytesMut) -> Self {
        match value.as_ref() {
            b"OnlineList" => Self::OnlineList,
            b"SendMsgToUser" => Self::SendMsgToUser,
            b"SendFileToUser" => Self::SendFileToUser,
            b"Login" => Self::Login,
            b"SendImageToUser" => Self::SendImageToUser,
            b"AcceptFile" => Self::AcceptFile,
            _ => Self::Help,
        }
    }
}

impl Command {
    pub fn help() -> Message {
        let content_text = format!(
            "\n{}\n{}\n{}\n{}\n",
            r"OnlineList#,,|$  ->  Request name list of online users",
            r"SendMsgToUser#,{receiver},|{msg}$  ->  send msg to the specified user",
            r"SendFileToUser#,{receiver},{filename}|{bytes}$  ->  send file to the specified user",
            r"AcceptFile#,{receiver},|{key}$  ->  accept file",
        );
        Message::send_text("Server", "", &content_text)
    }

    pub fn content(&self) -> Content {
        match self {
            Self::SendImageToUser => Content::File(FileData::default()),
            _ => Content::Text(String::default()),
        }
    }
}
