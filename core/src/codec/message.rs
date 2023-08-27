use std::fmt::Display;

use crate::codec::command::Command;
use bytes::{BufMut, BytesMut};
use colored::*;

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub receiver: String,
    pub command: Command,
    pub content: Vec<u8>,
}

/// serialize `Message` into bytes
impl From<Message> for BytesMut {
    fn from(value: Message) -> Self {
        let args_bytes = BytesMut::from(value.args_string().as_bytes());
        let command_bytes: BytesMut = value.command.into();
        let content_slice: &[u8] = &value.content;
        let content_bytes: BytesMut = BytesMut::from(content_slice);
        let mut bytes = BytesMut::new();
        // command#args|content$
        bytes.reserve(command_bytes.len() + args_bytes.len() + content_bytes.len() + 3);
        bytes.put(command_bytes);
        bytes.put("#".as_bytes());
        bytes.put(args_bytes);
        bytes.put("|".as_bytes());
        bytes.put(content_bytes);
        bytes.put("$".as_bytes());
        bytes
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_str = String::from_utf8_lossy(&self.content);
        write!(
            f,
            "{} -{}-> {}:\n{}",
            self.sender.green(),
            self.command.as_ref().yellow(),
            self.receiver.green(),
            content_str.yellow()
        )
    }
}

impl Message {
    fn args_string(&self) -> String {
        format!(
            "{},{},{}",
            self.content.len(),
            self.sender,
            self.receiver,
        )
    }

    pub fn login(uid: &str) -> Self {
        Self {
            sender: uid.into(),
            receiver: "Server".into(),
            command: Command::Login,
            content: "".into(),
        }
    }

    pub fn get_pub_key(to: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::GetPubKey,
            content: "".into(),
        }
    }

    pub fn send_pub_key(to: &str, rsa: &[u8]) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendPubKey,
            content: rsa.to_vec(),
        }
    }

    pub fn send_text(to: &str, content: &[u8]) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendMsg,
            content: content.to_vec(),
        }
    }

    pub fn online_list(content: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: "".into(),
            command: Command::OnlineList,
            content: content.into(),
        }
    }

    pub fn set_sender(mut self, sender: &str) -> Self {
        self.sender = sender.into();
        self
    }

    pub fn set_receiver(mut self, receiver: &str) -> Self {
        self.receiver = receiver.into();
        self
    }

    pub fn get_receiver(&self) -> String {
        self.receiver.clone()
    }
}
