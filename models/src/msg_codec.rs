use bytes::{Buf, BytesMut};
use std::{cmp, fmt::Display, io, usize};
use tokio_util::codec::Decoder;

use crate::{
    command::Command,
    message::{Content, Message},
};

#[derive(Debug)]
pub enum MsgCodecError {
    Io(io::Error),
    InvalidMessage,
}

impl Display for MsgCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::InvalidMessage => write!(f, "InvalidMessage"),
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
    cmd_index: usize,
    args_index: usize,
    max_arg_index: usize,
    next_index: usize,
    is_discarding: bool,
}

impl MsgCodec {
    pub fn new() -> Self {
        Self {
            cmd_index: 0,
            args_index: 0,
            next_index: 0,
            max_arg_index: 256,
            is_discarding: false,
        }
    }

    pub fn status(&self) -> MsgCodecStatus {
        if self.is_discarding {
            return MsgCodecStatus::Discarding;
        }
        if self.cmd_index == 0 {
            return MsgCodecStatus::Command;
        }
        if self.args_index == 0 {
            return MsgCodecStatus::Args;
        }
        MsgCodecStatus::Content
    }

    pub fn init(&mut self) {
        self.cmd_index = 0;
        self.args_index = 0;
        self.next_index = 0;
        self.is_discarding = false;
    }
}


impl Decoder for MsgCodec {
    type Item = Message;
    type Error = MsgCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            println!("{:?}", self);
            match self.status() {
                MsgCodecStatus::Command => {
                    let read_to = cmp::min(self.max_arg_index, buf.len());
                    let offset = buf[self.next_index..read_to]
                        .iter()
                        .position(|b| *b == b'#');
                    match offset {
                        None if buf.len() > self.max_arg_index => {
                            self.is_discarding = true;
                            return Err(MsgCodecError::InvalidMessage);
                        }
                        None => {
                            self.next_index = read_to;
                            return Ok(None);
                        }
                        Some(offset_from_next) => {
                            self.cmd_index = self.next_index + offset_from_next;
                            self.next_index = self.cmd_index;
                        }
                    }
                }
                MsgCodecStatus::Args => {
                    let read_to = cmp::min(self.max_arg_index, buf.len());
                    let offset = buf[self.next_index..read_to]
                        .iter()
                        .position(|b| *b == b'|');
                    match offset {
                        None if buf.len() > self.max_arg_index => {
                            self.is_discarding = true;
                            return Err(MsgCodecError::InvalidMessage);
                        }
                        None => {
                            self.next_index = read_to;
                            return Ok(None);
                        }
                        Some(offset_from_next) => {
                            self.args_index = self.next_index + offset_from_next;
                            self.next_index = self.args_index;
                        }
                    }
                }
                MsgCodecStatus::Content => {
                    let read_to = buf.len();
                    let end_offset = buf[self.next_index..read_to]
                        .iter()
                        .position(|b| *b == b'$');
                    match end_offset {
                        Some(offset) => {
                            let command_bytes = buf.split_to(self.cmd_index);
                            buf.advance(1); // TODO only works for ASCII char
                            let args_bytes = buf.split_to(self.args_index - self.cmd_index - 1);
                            buf.advance(1); // TODO only works for ASCII char
                            let content_bytes =
                                buf.split_to(self.args_index + offset - self.args_index - 1);

                            let command = Command::from(command_bytes);
                            let args_string = String::from_utf8(args_bytes.to_vec()).unwrap();
                            let args: Vec<String> =
                                args_string.split(",").map(|s| String::from(s)).collect();
                            let content = match command.content() {
                                Content::Text(_) => {
                                    let text = String::from_utf8(content_bytes.to_vec()).unwrap();
                                    Content::Text(text)
                                }
                                Content::Bytes(_) => Content::Bytes(content_bytes),
                            };
                            buf.advance(buf.len());
                            self.init();
                            return Ok(Some(Message {
                                command,
                                args,
                                content,
                            }));
                        }
                        None => {
                            self.next_index = read_to;
                            return Ok(None);
                        }
                    }
                }
                MsgCodecStatus::Discarding => {
                    println!("discard");
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
