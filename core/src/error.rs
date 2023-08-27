use colored::*;
use std::{fmt::Display, str::FromStr};

pub type GlobalResult<T> = std::result::Result<T, GlobalError>;
pub type ClientResult<T> = std::result::Result<T, ClientError>;
impl std::error::Error for GlobalError {}

#[derive(Debug)]
pub struct GlobalError {
    pub err: ErrorType,
    pub info: Option<String>,
}

#[derive(Debug)]
pub enum ErrorType {
    Client(ClientError),
    Server(ServerError),
    External(ExternalError),
}

impl ClientError {
    pub fn info(self, i: &str) -> GlobalError {
        GlobalError {
            err: ErrorType::Client(self),
            info: Some(i.into()),
        }
    }
}

impl ServerError {
    pub fn info(self, i: &str) -> GlobalError {
        GlobalError {
            err: ErrorType::Server(self),
            info: Some(i.into()),
        }
    }
}

impl ExternalError {
    pub fn info(self, i: &str) -> GlobalError {
        GlobalError {
            err: ErrorType::External(self),
            info: Some(i.into()),
        }
    }
}

impl From<ClientError> for GlobalError {
    fn from(value: ClientError) -> Self {
        Self {
            err: ErrorType::Client(value),
            info: None,
        }
    }
}

impl From<ServerError> for GlobalError {
    fn from(value: ServerError) -> Self {
        Self {
            err: ErrorType::Server(value),
            info: None,
        }
    }
}

impl From<ExternalError> for GlobalError {
    fn from(value: ExternalError) -> Self {
        Self {
            err: ErrorType::External(value),
            info: None,
        }
    }
}

impl From<String> for GlobalError {
    fn from(value: String) -> Self {
        use ErrorType::*;
        let (source, error_t) = value
            .split_once('-')
            .unwrap_or(("Client", "Unknown: cannot deserialize this error"));
        let (error, info) = match error_t.split_once(": ") {
            Some((e, i)) => (e, Some(i.into())),
            None => (error_t, None),
        };
        match source {
            "Server" => Self {
                err: Server(ServerError::from_str(error).unwrap_or(ServerError::Unknown)),
                info,
            },
            "External" => Self {
                err: External(ExternalError::from_str(error).unwrap_or(ExternalError::Unknown)),
                info,
            },
            _ => Self {
                err: Client(ClientError::from_str(error).unwrap_or(ClientError::Unknown)),
                info,
            },
        }
    }
}

impl From<GlobalError> for String {
    fn from(value: GlobalError) -> Self {
        use ErrorType::*;
        let info = if let Some(i) = value.info {
            format!(": {}", i)
        } else {
            String::new()
        };
        match value.err {
            Client(e) => format!("Client-{}{}", e.as_ref(), info),
            Server(e) => format!("Server-{}{}", e.as_ref(), info),
            External(e) => format!("External-{}{}", e.as_ref(), info),
        }
    }
}

impl Display for GlobalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorType::*;
        let info = if let Some(i) = &self.info {
            format!(": {}", i.yellow())
        } else {
            String::new()
        };
        match &self.err {
            Client(e) => write!(f, "Client-{}{}", e.as_ref().red(), info),
            Server(e) => write!(f, "Server-{}{}", e.as_ref().red(), info),
            External(e) => write!(f, "External-{}{}", e.as_ref().red(), info),
        }
    }
}

#[derive(Debug, strum::AsRefStr, strum::EnumString)]
pub enum ClientError {
    ReceiverNotExist,
    EncryptKeyGeneration,
    EncryptKeyPersistence,
    Encryption,
    Decryption,
    CannotEstablishConnection,
    AuthenticationFailed,
    ServerDisconnected,
    Unknown,
}

#[derive(Debug, strum::AsRefStr, strum::EnumString)]
pub enum ServerError {
    UserDisconnect,
    DuplicatedAuth,
    UnexpectedFrame,
    Unknown,
}

#[derive(Debug, strum::AsRefStr, strum::EnumString)]
pub enum ExternalError {
    Initialize,
    ListenPort,
    IO,
    Concurrent,
    DeserializeToml,
    SerializeToml,
    DeserializeFrame,
    SerializeFrame,
    TokioChannel,
    Unknown,
}

// ? implicitly invokes `into()`, `From<T>` gives T.into() for free

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for GlobalError{
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        ExternalError::TokioChannel.info(&format!("{}", value))
    }
}

impl<T> From<tokio::sync::mpsc::error::SendTimeoutError<T>> for GlobalError {
    fn from(value: tokio::sync::mpsc::error::SendTimeoutError<T>) -> Self {
        ExternalError::TokioChannel.info(&format!("{}", value))
    }
}

impl<T> From<tokio::sync::mpsc::error::TrySendError<T>> for GlobalError {
    fn from(value: tokio::sync::mpsc::error::TrySendError<T>) -> Self {
        ExternalError::TokioChannel.info(&format!("{}", value))
    }
}

impl From<tokio::sync::mpsc::error::TryRecvError> for GlobalError {
    fn from(value: tokio::sync::mpsc::error::TryRecvError) -> Self {
        ExternalError::TokioChannel.info(&format!("{}", value))
    }
}

impl From<std::io::Error> for GlobalError {
    fn from(value: std::io::Error) -> Self {
        wrap_e(ExternalError::IO, value)
    }
}

impl From<toml::de::Error> for GlobalError {
    fn from(value: toml::de::Error) -> Self {
        wrap_e(ExternalError::DeserializeToml, value)
    }
}

impl From<toml::ser::Error> for GlobalError {
    fn from(value: toml::ser::Error) -> Self {
        wrap_e(ExternalError::SerializeToml, value)
    }
}

impl From<rsa::Error> for GlobalError {
    fn from(value: rsa::Error) -> Self {
        wrap_c(ClientError::EncryptKeyGeneration, value) 
    }
}

impl From<rsa::pkcs8::Error> for GlobalError {
    fn from(value: rsa::pkcs8::Error) -> Self {
        wrap_c(ClientError::EncryptKeyPersistence, value) 
    }
}

fn wrap_c(e: ClientError, v: impl std::error::Error) -> GlobalError {
    e.info(&format!("{}", v))
}

fn wrap_e(e: ExternalError, v: impl std::error::Error) -> GlobalError {
    e.info(&format!("{}", v))
}
