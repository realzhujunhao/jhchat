use std::fmt::Display;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Listen,
    Config,
    Disconnect,
    Offline,
    RequestFormat,
    Unreachable,
    ServerToClient,
    InvalidMessage,
    Channel,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline => write!(f, "Offline"),
            Self::Disconnect => write!(f, "Disconnect"),
            Self::RequestFormat => write!(f, "RequestFormat"),
            Self::Unreachable => write!(f, "Unreachable"),
            Self::ServerToClient => write!(f, "ServerToClient"),
            Self::InvalidMessage => write!(f, "InvalidMessage"),
            Self::Channel => write!(f, "Channel"),
            Self::Listen => write!(f, "Listen"),
            Self::Config => write!(f, "Config"),
        }
    }
}

impl std::error::Error for Error {}
