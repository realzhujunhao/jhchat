use bytes::{Buf, BufMut, BytesMut};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::{cmp, usize};
use tokio_util::codec::{Decoder, Encoder};
use uuid::Uuid;

use crate::codec::{
    command::Command,
    message::{Content, Message},
};

#[derive(Debug)]
pub enum MsgCodecStatus {
    Command,
    Args,
    Content,
    Discarding,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum CodecRole {
    Server,
    Client,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MsgCodec {
    role: CodecRole,
    command: Option<Command>,
    // [content-length, sender, receiver, filename]
    args: Option<Vec<String>>,
    content_remain: usize,

    max_before_delimiter: usize,
    next_index: usize,
    is_discarding: bool,

    download_root: PathBuf,
    file_path: PathBuf,
    file_key: String,
}

impl MsgCodec {
    pub fn new(role: CodecRole, path: &str) -> Self {
        Self {
            role,
            command: None,
            args: None,
            next_index: 0,
            content_remain: 0,
            max_before_delimiter: 256,
            is_discarding: false,
            download_root: path.into(),
            file_path: PathBuf::new(),
            file_key: String::new(),
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
        self.file_path = PathBuf::new();
        self.next_index = 0;
        self.content_remain = 0;
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
        match item.command {
            Command::SendBytes => {
                let (cmd_arg_bytes, path) = item.cmd_arg_bytes_path()?;
                let mut temp = self.download_root.clone();
                temp.push(path);
                dst.reserve(cmd_arg_bytes.len());
                println!("{:?}", cmd_arg_bytes);
                dst.put(cmd_arg_bytes);
                println!("C");
                println!("{:?}", temp);
                let mut file = File::open(temp)?;
                println!("D");
                let mut buffer = [0u8; 2048];
                while let Ok(n) = file.read(&mut buffer) {
                    if n == 0 {
                        break;
                    }
                    println!("{:?}", &buffer[..n]);
                    dst.reserve(n);
                    dst.put(&buffer[..n]);
                }
                dst.reserve(1);
                dst.put("$".as_bytes());
                Ok(())
            }
            _ => {
                let bytes: BytesMut = item.into();
                dst.reserve(bytes.len());
                dst.put(bytes);
                Ok(())
            }
        }
    }
}

fn read_offset_limited(
    codec: &mut MsgCodec,
    buf: &mut BytesMut,
    delimiter: u8,
) -> Result<Option<usize>, ()> {
    let read_to = cmp::min(codec.max_before_delimiter, buf.len());
    let offset = buf[codec.next_index..read_to]
        .iter()
        .position(|b| *b == delimiter);
    match offset {
        None if buf.len() > codec.max_before_delimiter => {
            codec.is_discarding = true;
            return Err(());
        }
        None => {
            codec.next_index = read_to;
            return Ok(None);
        }
        Some(offset_from_next) => return Ok(Some(offset_from_next)),
    }
}

fn read_command(codec: &mut MsgCodec, buf: &mut BytesMut) -> Result<Option<Command>, ()> {
    match read_offset_limited(codec, buf, b'#')? {
        None => Ok(None),
        Some(offset_from_next) => {
            let cmd_end_index = codec.next_index + offset_from_next;
            let mut command_bytes = buf.split_to(cmd_end_index);
            trim_front(&mut command_bytes);
            buf.advance(1);
            codec.next_index = 0;
            Ok(Some(Command::from(command_bytes)))
        }
    }
}

fn read_args(codec: &mut MsgCodec, buf: &mut BytesMut) -> Result<Option<Vec<String>>, ()> {
    match read_offset_limited(codec, buf, b'|')? {
        None => Ok(None),
        Some(offset_from_next) => {
            let args_end_index = codec.next_index + offset_from_next;
            let args_bytes = buf.split_to(args_end_index);
            let args_string = String::from_utf8_lossy(&args_bytes).to_string();
            let args_vec: Vec<String> = args_string.split(",").map(|s| s.into()).collect();
            if args_vec.len() != 4 {
                return Err(());
            }
            buf.advance(1);
            codec.next_index = 0;
            codec.content_remain = args_vec[0].parse().map_err(|_| ())?;
            Ok(Some(args_vec))
        }
    }
}

fn init_file(mut dir_path: PathBuf) -> io::Result<(PathBuf, String)> {
    create_dir_all(&dir_path)?;
    let key = Uuid::new_v4().to_string();
    dir_path.push(&key);
    File::create(&dir_path)?;
    Ok((dir_path, key))
}

fn handle_remain(codec: &mut MsgCodec, buf: &mut BytesMut) -> Result<Option<()>, ()> {
    if codec.content_remain != 0 {
        return Ok(None);
    }
    if buf.len() == 0 {
        return Ok(None);
    }
    if *buf.first().unwrap() != b'$' {
        codec.is_discarding = true;
        return Err(());
    }
    buf.advance(1);
    Ok(Some(()))
}

impl Decoder for MsgCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.status() {
                MsgCodecStatus::Command => match read_command(self, buf) {
                    Err(_) => return Ok(Some(Command::help())),
                    Ok(None) => return Ok(None),
                    Ok(Some(cmd)) => {
                        self.command = Some(cmd);
                    }
                },
                MsgCodecStatus::Args => match read_args(self, buf) {
                    Err(_) => return Ok(Some(Command::help())),
                    Ok(None) => return Ok(None),
                    Ok(Some(args_vec)) => {
                        if Some(Command::SendBytes) == self.command {
                            match self.role {
                                CodecRole::Server => {
                                    let mut dir_path = self.download_root.clone();
                                    dir_path.push(&args_vec[2]);
                                    let (file_path, key) = init_file(dir_path)?;
                                    self.file_path = file_path;
                                    self.file_key = key;
                                }
                                CodecRole::Client => {
                                    let mut temp = self.download_root.clone();
                                    temp.push(&args_vec[3]);
                                    self.file_path = temp;
                                }
                            }
                        }
                        self.args = Some(args_vec);
                    }
                },
                MsgCodecStatus::Content => {
                    let args = self.args.as_ref().unwrap().clone();

                    match self.command {
                        Some(Command::SendBytes) => {
                            let read_to = cmp::min(buf.len(), self.content_remain);
                            let content_bytes = buf.split_to(read_to);
                            self.content_remain -= read_to;
                            let mut file = OpenOptions::new().append(true).open(&self.file_path)?;
                            file.write_all(&content_bytes)?;
                            match handle_remain(self, buf) {
                                Err(_) => return Ok(Some(Command::help())),
                                Ok(None) => return Ok(None),
                                Ok(Some(_)) => {
                                    let sender = args[1].clone();
                                    let receiver = args[2].clone();
                                    let filename = args[3].clone();
                                    let key = self.file_key.clone();
                                    self.init();
                                    return Ok(Some(Message::file_key(
                                        &sender, &receiver, &filename, &key,
                                    )));
                                }
                            }
                        }
                        _ => {
                            if buf.len() <= self.content_remain {
                                return Ok(None);
                            }
                            let content_bytes = buf.split_to(self.content_remain);
                            self.content_remain = 0;
                            match handle_remain(self, buf) {
                                Err(_) => return Ok(Some(Command::help())),
                                Ok(None) => return Ok(None),
                                Ok(Some(_)) => {
                                    let command = self.command.as_ref().unwrap().clone();
                                    let sender = args[1].clone();
                                    let receiver = args[2].clone();
                                    let content = match command.content() {
                                        Content::Text(_) => Content::Text(
                                            String::from_utf8_lossy(&content_bytes).to_string(),
                                        ),
                                        Content::File(_) => Content::file(&args[3], content_bytes),
                                    };
                                    self.init();
                                    return match command {
                                        Command::FileKey if CodecRole::Server == self.role => {
                                            let key = String::from_utf8_lossy(
                                                &content.into_filedata().unwrap().bytes,
                                            )
                                            .to_string();
                                            let mut dir = self.download_root.clone();
                                            dir.push(&receiver);
                                            dir.push(&key);
                                            Ok(Some(
                                                Message::send_file(
                                                    &receiver,
                                                    &dir.into_os_string().to_string_lossy(),
                                                )
                                                .set_sender(&sender),
                                            ))
                                        }
                                        _ => Ok(Some(Message {
                                            sender,
                                            receiver,
                                            command,
                                            content,
                                        })),
                                    };
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
