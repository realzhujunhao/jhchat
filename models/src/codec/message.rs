use std::fmt::Display;

use crate::codec::command::Command;
use bytes::{BufMut, BytesMut};
use colored::*;

#[derive(Debug)]
pub enum Content {
    Text(String),
    File(FileData),
}

impl From<Content> for BytesMut {
    fn from(value: Content) -> Self {
        match value {
            Content::Text(text) => BytesMut::from(text.as_bytes()),
            Content::File(file) => file.bytes,
        }
    }
}

impl Content {
    pub fn file(name: &str, bytes: BytesMut) -> Self {
        Self::File(FileData {
            name: name.into(),
            bytes,
        })
    }

    pub fn into_filedata(self) -> Option<FileData> {
        match self {
            Content::Text(_) => None,
            Content::File(file) => Some(file),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::File(file) => file.bytes.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct FileData {
    pub name: String,
    pub bytes: BytesMut,
}

impl Default for FileData {
    fn default() -> Self {
        Self {
            name: "".into(),
            bytes: BytesMut::new(),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub sender: String,
    pub receiver: String,
    pub command: Command,
    pub content: Content,
}

/// serialize `Message` into bytes
impl From<Message> for BytesMut {
    fn from(value: Message) -> Self {
        let args_bytes = BytesMut::from(value.args_string().as_bytes());
        let command_bytes: BytesMut = value.command.into();
        let content_bytes: BytesMut = value.content.into();
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
        use Content::*;
        let content: &str = match &self.content {
            Text(ref s) => s,
            File(_) => {
                ""
            }
        };
        write!(
            f,
            "{} -{}-> {}: {}",
            self.sender.green(),
            self.command.as_ref().yellow(),
            self.receiver.green(),
            content.yellow()
        )
    }
}

impl Message {
    fn args_string(&self) -> String {
        let filename = match self.content {
            Content::Text(_) => "".into(),
            Content::File(ref f) => f.name.clone(),
        };
        format!(
            "{},{},{},{}",
            self.content.len(),
            self.sender,
            self.receiver,
            filename
        )
    }

    pub fn login(uid: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: "".into(),
            command: Command::Login,
            content: Content::Text(uid.into()),
        }
    }

    pub fn get_rsa(to: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::GetRSA,
            content: Content::Text("".into()),
        }
    }

    pub fn send_rsa(to: &str, rsa: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendRSA,
            content: Content::Text(rsa.into()),
        }
    }

    pub fn send_text(to: &str, content: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendMsg,
            content: Content::Text(content.into()),
        }
    }

    pub fn send_image(to: &str, filename: &str, content: BytesMut) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendImage,
            content: Content::file(filename, content),
        }
    }

    pub fn online_list(content: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: "".into(),
            command: Command::OnlineList,
            content: Content::Text(content.into()),
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
