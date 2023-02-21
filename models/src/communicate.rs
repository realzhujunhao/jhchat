use std::error::Error;

pub trait Communicate {
    fn send_msg(uid: &str) -> Result<(), Box<dyn Error>>;
    fn recv_msg(uid: &str) -> Result<String, Box<dyn Error>>;

    // fn send_file() -> Result<(), Box<dyn Error>>;
    // fn recv_file() -> Result<&'static [u8], Box<dyn Error>>;
}
