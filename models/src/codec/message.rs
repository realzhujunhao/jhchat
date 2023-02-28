use crate::codec::command::Command;
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub enum Content {
    Text(String),
    File(FileData),
}

impl Into<BytesMut> for Content {
    fn into(self) -> BytesMut {
        match self {
            Self::Text(text) => BytesMut::from(text.as_bytes()),
            Self::File(file) => file.bytes,
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

impl Into<BytesMut> for Message {
    fn into(self) -> BytesMut {
        let filename = match self.content {
            Content::Text(_) => "".into(),
            Content::File(ref f) => f.name.clone(),
        };
        let args_string = format!("{},{},{}", self.sender, self.receiver, filename);

        let args_bytes = BytesMut::from(args_string.as_bytes());
        let command_bytes: BytesMut = self.command.into();
        let content_bytes: BytesMut = self.content.into();

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

impl Message {
    pub fn send_text(from: &str, to: &str, content: &str) -> Self {
        Self {
            sender: from.into(),
            receiver: to.into(),
            command: Command::SendMsgToUser,
            content: Content::Text(content.into()),
        }
    }

    pub fn send_file(from: &str, to: &str, filename: &str, content: BytesMut) -> Self {
        Self {
            sender: from.into(),
            receiver: to.into(),
            command: Command::SendFileToUser,
            content: Content::file(filename, content),
        }
    }

    pub fn send_image(from: &str, to: &str, filename: &str, content: BytesMut) -> Self {
        Self {
            sender: from.into(),
            receiver: to.into(),
            command: Command::SendImageToUser,
            content: Content::file(filename, content),
        }
    }
}
