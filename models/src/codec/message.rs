use crate::codec::command::Command;
use bytes::{BufMut, BytesMut};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

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
        let args_bytes = BytesMut::from(self.args_string().as_bytes());
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
    pub fn cmd_arg_bytes_path(self) -> io::Result<(BytesMut, PathBuf)> {
        if let Content::Text(ref text) = self.content {
            let command_bytes: BytesMut = self.command.into();
            let file_path = Path::new(text);
            let filename = file_path
                .file_name()
                .ok_or(io::Error::new(io::ErrorKind::NotFound, text.as_str()))?
                .to_string_lossy()
                .to_string();
            let file_length = File::open(file_path)?.metadata()?.len();
            let args_bytes = format!(
                "{},{},{},{}",
                file_length, self.sender, self.receiver, filename
            );
            let args_bytes = args_bytes.as_bytes();
            let mut bytes = BytesMut::new();
            bytes.reserve(command_bytes.len() + args_bytes.len() + 2);
            bytes.put(command_bytes);
            bytes.put("#".as_bytes());
            bytes.put(args_bytes);
            bytes.put("|".as_bytes());

            let mut file_path = PathBuf::new();
            file_path.push(&self.receiver);
            file_path.push(&filename);
            return Ok((bytes, file_path));
        }
        unreachable!()
    }

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

    pub fn send_text(to: &str, content: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendMsg,
            content: Content::Text(content.into()),
        }
    }

    pub fn send_file(to: &str, path: &str) -> Self {
        Self {
            sender: "".into(),
            receiver: to.into(),
            command: Command::SendBytes,
            content: Content::Text(path.into()),
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

    pub fn file_key(from: &str, to: &str, filename: &str, key: &str) -> Self {
        Self {
            sender: from.into(),
            receiver: to.into(),
            command: Command::FileKey,
            content: Content::file(filename, BytesMut::from(key)),
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
