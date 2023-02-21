use std::error::Error;
use crate::user::User;

pub trait Communicate {
    fn send_msg(from: &User, msg: &str) -> Result<(), Box<dyn Error>>;
    fn recv_msg(uid: &str) -> Result<String, Box<dyn Error>>;

    // fn send_file() -> Result<(), Box<dyn Error>>;
    // fn recv_file() -> Result<&'static [u8], Box<dyn Error>>;
}
