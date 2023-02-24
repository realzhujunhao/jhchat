use std::io;
use crate::{message::Message, user::User};

pub trait Communicate {
    fn recv_from(from: &User, msg: Message) -> io::Result<()>;

    // fn send_file() -> Result<(), Box<dyn Error>>;
    // fn recv_file() -> Result<&'static [u8], Box<dyn Error>>;
}
