use bytes::{Buf, BufMut, BytesMut};
use chrono::prelude::*;
use sha256::try_digest;
use std::fs::{create_dir_all, rename, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::{cmp, io, usize};
use tokio_util::codec::{Decoder, Encoder};

use crate::codec::{
    command::Command,
    message::{Content, Message},
};

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

//TODO
impl Encoder<Message> for MsgCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes: BytesMut = item.into();
        dst.reserve(bytes.len());
        dst.put(bytes);
        Ok(())
    }
}

impl Decoder for MsgCodec {
    type Item = Message;
    type Error = io::Error;

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
                            return Ok(Some(Command::help()));
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
                            return Ok(Some(Command::help()));
                        }
                        None => {
                            self.next_index = read_to;
                            return Ok(None);
                        }
                        Some(offset_from_next) => {
                            let args_end_index = self.next_index + offset_from_next;
                            let args_bytes = buf.split_to(args_end_index);
                            let args_string = String::from_utf8_lossy(&args_bytes).to_string();
                            let mut args_vec: Vec<String> =
                                args_string.split(",").map(|s| String::from(s)).collect();
                            if args_vec.len() != 3 {
                                return Ok(Some(Command::help()));
                            }

                            if Some(Command::SendFileToUser) == self.command {
                                let dir_path =
                                    format!("{}/{}", self.file_path, &args_vec[1]);
                                create_dir_all(&dir_path)?;
                                let time = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();
                                let file_path = format!("{}/{}", dir_path, time);
                                File::create(&file_path)?;
                                args_vec.push(file_path);
                                args_vec.push(dir_path);
                            }

                            self.args = Some(args_vec);
                            buf.advance(1); // remove args_end_delimiter
                            self.next_index = 0;
                        }
                    }
                }
                MsgCodecStatus::Content => {
                    let read_to = buf.len();

                    match self.command {
                        Some(Command::SendFileToUser) => {
                            //INFALLIBLE
                            let args = self.args.as_ref().unwrap();
                            if args.len() != 5 {
                                return Ok(Some(Command::help()));
                            }
                            let file_path = &args[3];
                            let dir_path = &args[4];
                            let mut file = OpenOptions::new().append(true).open(file_path)?;

                            let end_offset = buf[self.next_index..read_to]
                                .iter()
                                .position(|b| *b == b'$');
                            match end_offset {
                                None => {
                                    let written = buf.split_to(read_to);
                                    file.write_all(&written)?;
                                    self.next_index = 0;
                                    return Ok(None);
                                }
                                Some(offset_from_next) => {
                                    let written = buf.split_to(offset_from_next);
                                    file.write_all(&written)?;
                                    drop(file);
                                    let key = try_digest(Path::new(file_path))?;
                                    rename(file_path, format!("{}/{}", dir_path, key))?;
                                    let sender = args[0].clone();
                                    let receiver = args[1].clone();
                                    let filename = args[2].clone();
                                    self.init();
                                    return Ok(Some(Message::send_file(
                                        &sender,
                                        &receiver,
                                        &filename,
                                        BytesMut::from(key.as_bytes()),
                                    )));
                                }
                            }
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
                                    // INFALLIBLE
                                    let command = self.command.as_ref().unwrap().clone();
                                    let args = self.args.as_ref().unwrap().clone();

                                    let content_end_index = self.next_index + offset_from_next;
                                    let content_bytes = buf.split_to(content_end_index);
                                    let content = match command.content() {
                                        Content::Text(_) => Content::Text(
                                            String::from_utf8_lossy(&content_bytes).to_string(),
                                        ),
                                        Content::File(_) => Content::file(&args[2], content_bytes),
                                    };

                                    buf.advance(1); // remove content_end_delimiter
                                    self.init();
                                    return Ok(Some(Message {
                                        sender: args[0].clone(),
                                        receiver: args[1].clone(),
                                        command,
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
