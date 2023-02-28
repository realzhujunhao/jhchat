use bytes::{Buf, BufMut, BytesMut};
use std::{cmp, fmt::Display, io, usize};
use tokio_util::codec::{Decoder, Encoder};
use sha256::try_digest;
use std::fs::OpenOptions;

use crate::{
    command::Command,
    message::{Content, Message},
};

#[derive(Debug)]
pub enum MsgCodecError {
    Io(io::Error),
    InvalidCommand,
    InvalidArguments,
    InvalidContent,
}

impl Display for MsgCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::InvalidCommand => write!(f, "InvalidCommand"),
            Self::InvalidArguments => write!(f, "InvalidArgujments"),
            Self::InvalidContent => write!(f, "InvalidContent"),
        }
    }
}

impl From<io::Error> for MsgCodecError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::error::Error for MsgCodecError {}

pub enum MsgCodecStatus {
    Command,
    Args,
    Content,
    Discarding,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MsgCodec {
    command: Option<Command>,
    args: Option<Vec<String>>,
    max_before_delimiter: usize,
    next_index: usize,
    is_discarding: bool,
    file_path: String,
}

impl MsgCodec {
    pub fn new(path: &str) -> Self {
        Self {
            command: None,
            args: None,
            next_index: 0,
            max_before_delimiter: 256,
            is_discarding: false,
            file_path: path.into(),
        }
    }

    pub fn status(&self) -> MsgCodecStatus {
        if self.is_discarding {
            return MsgCodecStatus::Discarding;
        }
        if self.command.is_none() {
            return MsgCodecStatus::Command;
        }
        if self.args.is_none() {
            return MsgCodecStatus::Args;
        }
        MsgCodecStatus::Content
    }

    pub fn init(&mut self) {
        self.command = None;
        self.args = None;
        self.next_index = 0;
        self.is_discarding = false;
    }
}

fn trim_front(buf: &mut BytesMut) {
    while let Some(b'\n') = buf.first() {
        buf.advance(1);
    }
    while let Some(b'\r') = buf.first() {
        buf.advance(1);
    }
    while let Some(b' ') = buf.first() {
        buf.advance(1);
    }
}

impl Encoder<Message> for MsgCodec {
    type Error = MsgCodecError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes: BytesMut = item.into();
        dst.reserve(bytes.len());
        dst.put(bytes);
        Ok(())
    }
}

impl Decoder for MsgCodec {
    type Item = Message;
    type Error = MsgCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.status() {
                MsgCodecStatus::Command => {
                    let read_to = cmp::min(self.max_before_delimiter, buf.len());
                    let offset = buf[self.next_index..read_to]
                        .iter()
                        .position(|b| *b == b'#');
                    match offset {
                        None if buf.len() > self.max_before_delimiter => {
                            self.is_discarding = true;
                            return Err(MsgCodecError::InvalidCommand);
                        }
                        None => {
                            self.next_index = read_to;
                            return Ok(None);
                        }
                        Some(offset_from_next) => {
                            let cmd_end_index = self.next_index + offset_from_next;
                            let mut command_bytes = buf.split_to(cmd_end_index);
                            trim_front(&mut command_bytes);
                            self.command = Some(Command::from(command_bytes));
                            buf.advance(1); // remove command_end_delimiter
                            self.next_index = 0;
                        }
                    }
                }
                MsgCodecStatus::Args => {
                    let read_to = cmp::min(self.max_before_delimiter, buf.len());
                    let offset = buf[self.next_index..read_to]
                        .iter()
                        .position(|b| *b == b'|');
                    match offset {
                        None if buf.len() > self.max_before_delimiter => {
                            self.is_discarding = true;
                            return Err(MsgCodecError::InvalidArguments);
                        }
                        None => {
                            self.next_index = read_to;
                            return Ok(None);
                        }
                        Some(offset_from_next) => {
                            let args_end_index = self.next_index + offset_from_next;
                            let args_bytes = buf.split_to(args_end_index);
                            let args_string = String::from_utf8_lossy(&args_bytes).to_string();
                            self.args =
                                Some(args_string.split(",").map(|s| String::from(s)).collect());
                            buf.advance(1); // remove args_end_delimiter
                            self.next_index = 0;
                        }
                    }
                }
                MsgCodecStatus::Content => {
                    let read_to = buf.len();

                    match self.command {
                        Some(Command::SendFileToUser) => {
                            let args = self.args.as_ref().ok_or(MsgCodecError::InvalidArguments)?;
                            let end_offset = buf[self.next_index..read_to].iter().position(|b| *b == b'$');
                            let file = OpenOptions::new().append(true).create(true).open("").map_err(|e| MsgCodecError::Io(e))?;
                            match end_offset {
                                None => {
                                    
                                }
                                Some(offset_from_next) => {

                                }
                            }
                            unreachable!()
                        }
                        _ => {
                            let end_offset = buf[self.next_index..read_to]
                                .iter()
                                .position(|b| *b == b'$');

                            match end_offset {
                                None => {
                                    self.next_index = read_to;
                                    return Ok(None);
                                }
                                Some(offset_from_next) => {
                                    let command = self.command.as_ref().unwrap().clone();
                                    let args = self.args.as_ref().unwrap().clone();

                                    let content_end_index = self.next_index + offset_from_next;
                                    let content_bytes = buf.split_to(content_end_index);
                                    let content = match command.content() {
                                        Content::Text(_) => Content::Text(
                                            String::from_utf8_lossy(&content_bytes).to_string(),
                                        ),
                                        Content::Bytes(_) => Content::Bytes(content_bytes),
                                    };

                                    buf.advance(1); // remove content_end_delimiter
                                    self.init();
                                    return Ok(Some(Message {
                                        command,
                                        args,
                                        content,
                                    }));
                                }
                            }
                        }
                    }
                }
                MsgCodecStatus::Discarding => {
                    let end_offset = buf[self.next_index..buf.len()]
                        .iter()
                        .position(|b| *b == b'$');
                    match end_offset {
                        Some(offset) => {
                            buf.advance(offset + self.next_index + 1);
                            self.init();
                        }
                        None => {
                            buf.advance(buf.len());
                            self.next_index = 0;
                            if buf.is_empty() {
                                return Ok(None);
                            }
                        }
                    }
                }
            }
        }
    }
}
