use bytes::BytesMut;
use std::str::FromStr;

use crate::codec::message::Message;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::AsRefStr, strum::EnumString)]
pub enum Command {
    Help,
    Login,
    OnlineList,
    SendMsg,
    GetPubKey,
    SendPubKey,
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
        Message::send_text("", b"").set_sender("Server")
    }
}
