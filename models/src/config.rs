use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

trait Config {
    fn write_string(content: &str) -> io::Result<()> {
        let mut file = Self::config_file()?;
        Ok(file.write_all(content.as_bytes())?)
    }

    fn read_string() -> io::Result<String> {
        let mut file = Self::config_file()?;
        let mut json_string = String::new();
        file.read_to_string(&mut json_string)?;
        Ok(json_string)
    }

    fn config_file() -> io::Result<File> {
        let path = Self::config_path()?;
        Ok(OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?)
    }

    fn is_exist() -> io::Result<bool> {
        let path = Self::config_path()?;
        path.try_exists()
    }

    fn config_path() -> io::Result<PathBuf> {
        let mut exe_path = env::current_exe()?;
        exe_path.pop();
        exe_path.push("config");
        exe_path.set_extension("json");
        Ok(exe_path)
    }
}

impl Config for ServerConfig {}
impl Config for ClientConfig {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub ip: String,
    pub port: String,
}

impl ServerConfig {
    pub fn init(self) -> io::Result<Self> {
        match Self::is_exist()? {
            true => {
                let content = Self::read_string()?;
                Ok(serde_json::from_str(&content)?)
            }
            false => {
                let content = serde_json::to_string(&self)?;
                Self::write_string(&content)?;
                println!("config.json is newly created, please restart the server after configuration.");
                Ok(self)
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    todo: i32,
}
