use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

pub trait Config {
    type This;

    fn init() -> io::Result<Self::This>;

    fn write_string(content: &str) -> io::Result<()> {
        let mut file = Self::config_file()?;
        file.write_all(content.as_bytes())
    }

    fn read_string() -> io::Result<String> {
        let mut file = Self::config_file()?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string)?;
        Ok(json_string)
    }

    fn config_file() -> io::Result<File> {
        let path = Self::config_path()?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
    }

    fn exist() -> io::Result<bool> {
        let path = Self::config_path()?;
        path.try_exists()
    }

    /// config.json locates under same directory as executable
    fn config_path() -> io::Result<PathBuf> {
        let mut exe_path = env::current_exe()?;
        exe_path.pop();
        exe_path.push("config");
        exe_path.set_extension("json");
        Ok(exe_path)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub ip: String,
    pub port: String,
}

impl Config for ServerConfig {
    type This = Self;

    /**
     * if `config.json` exists, use it
     * if not, create a default config file
     */
    fn init() -> io::Result<Self::This> {
        let default_config = Self::default();
        match Self::exist()? {
            true => {
                let content = Self::read_string()?;
                Ok(serde_json::from_str(&content)?)
            }
            false => {
                let content = serde_json::to_string_pretty(&default_config)?;
                Self::write_string(&content)?;
                Ok(default_config)
            }
        }
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

// TODO
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    pub download_path: String,
}

impl Config for ClientConfig {
    type This = Self;

    fn init() -> io::Result<Self::This> {
        Ok(Self {
            download_path: "./down/".into(),
        })
    }
}
