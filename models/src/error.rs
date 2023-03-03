use std::{fmt::Display, net::SocketAddr};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Listen(String),
    Config,
    ConnectionFail(SocketAddr),
    Disconnect,
    Offline(String),
    RequestFormat,
    Unreachable,
    ServerToClient,
    ClientToServer,
    InvalidMessage,
    Channel,
    RwLock,
    FilePath(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline(user) => write!(f, "Offline {}", user),
            Self::Disconnect => write!(f, "Disconnect"),
            Self::RequestFormat => write!(f, "RequestFormat"),
            Self::Unreachable => write!(f, "Unreachable"),
            Self::ServerToClient => write!(f, "ServerToClient"),
            Self::InvalidMessage => write!(f, "InvalidMessage"),
            Self::Channel => write!(f, "Channel"),
            Self::RwLock => write!(f, "RwLock"),
            Self::Listen(port) => write!(f, "Listen {}", port),
            Self::Config => write!(f, "Config"),
            Self::FilePath(path) => write!(f, "FilePath {:?}", path),

            Self::ConnectionFail(addr) => write!(f, "ConnectionFail {:?}", addr),
            Self::ClientToServer => write!(f, "ClientToServer"),
        }
    }
}

impl std::error::Error for Error {}
