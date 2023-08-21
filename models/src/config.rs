use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    env,
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::error::GlobalResult;

// type AnyResult<T> = Result<T, Box<dyn Error>>;

/// associate type should always be `Self`, so that init() can return the specific config struct
pub trait Config: DeserializeOwned + Serialize {
    type This: DeserializeOwned + Serialize;

    /// the default value of configuration
    /// `Default` trait is not used here because it returns `Self`
    /// while the associate type `This` is required
    fn associated_default() -> Self::This;

    fn config_path() -> io::Result<PathBuf>;

    /// if `config_path` does exist, use it
    /// if not, create a default config file
    fn init() -> GlobalResult<Self::This> {
        match Self::exist()? {
            true => {
                let s = Self::read_string()?;
                Ok(toml::from_str(&s)?)
            }
            false => {
                let default_config = Self::associated_default();
                let content = toml::to_string_pretty(&default_config)?;
                Self::write_string(&content)?;
                Ok(default_config)
            }
        }
    }

    fn write_string(content: &str) -> io::Result<()> {
        Self::config_file().and_then(|mut f| f.write_all(content.as_bytes()))?;
        Ok(())
    }

    fn read_string() -> io::Result<String> {
        let mut buf = String::new();
        Self::config_file().and_then(|mut f| f.read_to_string(&mut buf))?;
        Ok(buf)
    }

    fn config_file() -> io::Result<File> {
        let path = Self::config_path()?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(file)
    }

    fn exist() -> io::Result<bool> {
        let exist = Self::config_path().and_then(|path| path.try_exists())?;
        Ok(exist)
    }

    fn cur_exe() -> io::Result<PathBuf> {
        env::current_exe()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub ip: String,
    pub port: String,
}

impl Config for ServerConfig {
    type This = Self;

    /// config.toml locates under same directory as executable
    fn config_path() -> io::Result<PathBuf> {
        let mut path = Self::cur_exe()?;
        path.pop();
        path.push("config");
        path.set_extension("toml");
        Ok(path)
    }

    fn associated_default() -> Self::This {
        Self::default()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".into(),
            port: "2333".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    pub server_host: String,
    pub crypto: Crypto,
}

impl Config for ClientConfig {
    type This = Self;
    /// config.toml locates under same directory as executable
    fn config_path() -> io::Result<PathBuf> {
        let exe = Self::cur_exe()?;
        let exe_dir = exe.parent().unwrap_or(Path::new(""));
        let config_path = exe_dir.join("config.toml");
        Ok(config_path)
    }

    fn associated_default() -> Self::This {
        Self::default()
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0:2333".into(),
            crypto: Crypto::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Crypto {
    // public key and private key of current user
    pub rsa_self_dir: String,

    // public keys from server
    // keys exchanged via chat server
    pub rsa_unsafe_dir: String,

    // public keys from third party communication
    // keys exchanged via email, offline, etc.
    pub rsa_safe_dir: String,

    // when the public key provided is not identical to the one in rsa_safe_dir
    // original message will be cancelled (with warning to user)
    // this message will be encrypted by server public key and sent to server
    // therefore the server will not be aware of the fact client has already detected its malicious behavior
    pub dummy_msg: String,

    // by default the client never sends original message if the public key from server mismatches
    // the one in rsa_safe_dir
    // if this value is set to `true`, only the first original message will be cancelled
    pub send_on_unsafe: bool,
}

impl Default for Crypto {
    fn default() -> Self {
        let exe = env::current_exe().unwrap_or_default();
        let exe_dir = exe.parent().unwrap_or(Path::new("./"));
        let rsa_self_dir = exe_dir.join("rsa_self").to_string_lossy().into();
        let rsa_unsafe_dir = exe_dir.join("rsa_unsafe").to_string_lossy().into();
        let rsa_safe_dir = exe_dir.join("rsa_safe").to_string_lossy().into();
        Self {
            rsa_self_dir,
            rsa_unsafe_dir,
            rsa_safe_dir,
            dummy_msg: "hello".into(),
            send_on_unsafe: false,
        }
    }
}
