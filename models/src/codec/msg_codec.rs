use bytes::{Buf, BufMut, BytesMut};
use std::io;
use std::{cmp, usize};
use tokio_util::codec::{Decoder, Encoder};

use crate::codec::{
    command::Command,
    message::{Content, Message},
};

/// the four states in MessageDecoder
/// `Command`, `Args`, `Content` are stages that parse sections w.r.t their name
/// `Discarding` stage clears the read buffer to make sure later frames are not affected  
#[derive(Debug)]
pub enum MsgCodecStatus {
    Command,
    Args,
    Content,
    Discarding,
}

/// necessary internal states
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MsgCodec {
    // Command in `Message`
    command: Option<Command>,

    // [content-length, sender, receiver, filename]
    args: Option<Vec<String>>,

    // max length of `Command` and `Args`
    max_before_delimiter: usize,

    // expected length of content
    content_len: usize,

    // triggered when byte stream is in wrong form -> discard to end
    is_discarding: bool,
}

impl MsgCodec {
    pub fn new() -> Self {
        Self {
            command: None,
            args: None,
            max_before_delimiter: 512,
            content_len: 0,
            is_discarding: false,
        }
    }

    /// for either `Discarding` or complete
    pub fn reset(&mut self) {
        self.command = None;
        self.args = None;
        self.max_before_delimiter = 512;
        self.content_len = 0;
        self.is_discarding = false;
    }

    /// determine current state
    /// Args, Command, Content are decoded only when the end delimiter arrives
    /// that is, bytes are not modified partially, but as a whole section
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
}

impl Default for MsgCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// remove white space from beginning of buffer
/// if `buf.first()` returns None, i.e buffer is empty, default false breaks the loop
/// otherwise, this while loop continues until first byte is not ascii white space
fn trim_front(buf: &mut BytesMut) {
    while buf
        .first()
        .map_or(false, |&byte| byte.is_ascii_whitespace())
    {
        buf.advance(1);
    }
}

/// simply serialize `Message` into bytes and put into buffer
impl Encoder<Message> for MsgCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes: BytesMut = item.into();
        dst.reserve(bytes.len());
        dst.put(bytes);
        Ok(())
    }
}

/// read the buffer to the limit or #bytes in buffer
/// search for byte index of delimiter as UTF-8 character
/// Ok(None) if not found
/// Ok(Some(offset)) if found
/// Err(()) if exceed limit
fn find_delimiter(
    codec: &mut MsgCodec,
    buf: &mut BytesMut,
    delimiter: char,
) -> Result<Option<usize>, ()> {
    let read_to = cmp::min(codec.max_before_delimiter, buf.len());
    let slice = &buf[0..read_to];
    let utf8_string = String::from_utf8_lossy(slice);
    let delimiter_idx = utf8_string
        .char_indices()
        .find(|(_, ch)| *ch == delimiter)
        .map(|(byte_idx, _)| byte_idx);

    match delimiter_idx {
        None if buf.len() > codec.max_before_delimiter => Err(()),
        None => Ok(None),
        Some(deli_idx) => Ok(Some(deli_idx)),
    }
}

/// read until limit, propagate error if exceed limit
/// if delimiter not found return Ok(None)
/// otherwise deserialize command and remove all bytes include delimiter from buffer
fn read_command(codec: &mut MsgCodec, buf: &mut BytesMut) -> Result<Option<Command>, ()> {
    match find_delimiter(codec, buf, '#')? {
        None => Ok(None),
        Some(delimiter_idx) => {
            let mut command_bytes = buf.split_to(delimiter_idx);
            buf.advance(1);

            trim_front(&mut command_bytes);
            Ok(Some(Command::from(command_bytes)))
        }
    }
}

/// read until limit, propagate error if exceed limit
/// if delimiter not found return Ok(None)
/// otherwise deserialize arguments and remove all bytes include delimiter from buffer
fn read_args(codec: &mut MsgCodec, buf: &mut BytesMut) -> Result<Option<Vec<String>>, ()> {
    match find_delimiter(codec, buf, '|')? {
        None => Ok(None),
        Some(delimiter_idx) => {
            let args_bytes = buf.split_to(delimiter_idx);
            buf.advance(1);

            let args_string = String::from_utf8_lossy(&args_bytes).to_string();
            let args_vec: Vec<String> = args_string.split(',').map(|s| s.into()).collect();
            if args_vec.len() != 4 {
                return Err(());
            }

            codec.content_len = args_vec[0].parse().map_err(|_| ())?;
            Ok(Some(args_vec))
        }
    }
}

impl Decoder for MsgCodec {
    type Item = Message;
    type Error = io::Error;

    /// MessageDecoder is a state machine with four states
    /// note that `Message` can be serialized into three sections
    /// [Command, Arguments, Content]
    /// where Arguments = `content-length,sender,receiver,filename`
    /// bytes format: `command#length,sender,receiver,filename|content$`
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.status() {
                // in case of wrong format, respond with help and reset buffer
                MsgCodecStatus::Command => match read_command(self, buf) {
                    Err(_) => {
                        self.is_discarding = true;
                        return Ok(Some(Command::help()));
                    }
                    Ok(None) => return Ok(None),
                    Ok(Some(cmd)) => {
                        self.command = Some(cmd);
                    }
                },
                MsgCodecStatus::Args => match read_args(self, buf) {
                    Err(_) => {
                        self.is_discarding = true;
                        return Ok(Some(Command::help()));
                    }
                    Ok(None) => return Ok(None),
                    Ok(Some(args_vec)) => {
                        self.args = Some(args_vec);
                    }
                },
                MsgCodecStatus::Content => {
                    if buf.len() <= self.content_len {
                        return Ok(None);
                    }
                    if buf[self.content_len] != b'$' {
                        self.is_discarding = true;
                        return Ok(Some(Command::help()));
                    }

                    let command = self.command.take().unwrap();
                    let args = self.args.take().unwrap();

                    let sender = args[1].clone();
                    let receiver = args[2].clone();

                    let content_bytes = buf.split_to(self.content_len);
                    let content = match command.content() {
                        Content::Text(_) => {
                            Content::Text(String::from_utf8_lossy(&content_bytes).to_string())
                        }
                        Content::File(_) => Content::file(&args[3], content_bytes),
                    };

                    buf.advance(1);
                    self.reset();

                    return Ok(Some(Message {
                        sender,
                        receiver,
                        command,
                        content,
                    }));
                }
                MsgCodecStatus::Discarding => {
                    let utf8_string = String::from_utf8_lossy(buf);
                    let delimiter_idx = utf8_string
                        .char_indices()
                        .find(|(_, ch)| *ch == '$')
                        .map(|(byte_idx, _)| byte_idx);
                    match delimiter_idx {
                        None => return Ok(None),
                        Some(idx) => {
                            buf.advance(idx);
                            self.is_discarding = false;
                            return Ok(None);
                        }
                    }
                }
            }
        }
    }
}
