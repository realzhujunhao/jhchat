use std::str::FromStr;
use bytes::BytesMut;

use crate::message::Content;

#[derive(Debug)]
pub enum Command {
    Login,
    OnlineList,
    SendMsgToUser,
    SendFileToUser,
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

impl From<BytesMut> for Command {
    fn from(value: BytesMut) -> Self {
        match value.as_ref() {
            b"OnlineList" => Self::OnlineList,
            b"SendMsgToUser" => Self::SendMsgToUser,
            b"SendFileToUser" => Self::SendFileToUser,
            b"Login" => Self::Login,
            _ => Self::Help,
        }
    }
}

impl Command {
    pub fn help() -> String {
        format!(
            "{}\n{}\n{}\n",
            "OnlineList#|$  ->  Request name list of online users",
            "SendMsgToUser#{{username}}|{{msg}}$  ->  send msg to the specified user",
            "SendFileToUser#{{username}}|{{filepath}}$  ->  send file to the specified user"
        )
    }

    pub fn content(&self) -> Content {
        match self {
            Self::SendFileToUser => Content::Bytes(BytesMut::default()),
            _ => Content::Text(String::default()),
        }
    } 
}