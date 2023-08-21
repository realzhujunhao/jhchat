use std::{fs::create_dir_all, path::Path, net::{SocketAddr, ToSocketAddrs}};

use colored::*;
use models::{
    config::{ClientConfig, Config},
    error::{GlobalResult, ExternalError},
};

pub fn config() -> GlobalResult<ClientConfig> {
    let config = ClientConfig::init()?;
    Ok(config)
}

/// read path from config, then create missing directories
/// config is returned to fulfill chain call
pub fn directory(config: ClientConfig) -> GlobalResult<ClientConfig> {
    let rsa_self = Path::new(&config.crypto.rsa_self_dir);
    let rsa_safe = Path::new(&config.crypto.rsa_safe_dir);
    let rsa_unsafe = Path::new(&config.crypto.rsa_unsafe_dir);

    let create_success = [rsa_self, rsa_safe, rsa_unsafe]
        .into_iter()
        .filter(|dir| !dir.is_dir())
        .map(|dir| create_dir_all(dir))
        .all(|r| r.is_ok());

    if !create_success {
        return Err(ExternalError::IO.into());
    }
    Ok(config)
}

pub fn socket_addr(config: ClientConfig) -> GlobalResult<(SocketAddr, ClientConfig)> {
    let addr = config
        .server_host
        .to_socket_addrs()
        .expect(&"to_socket_addrs() failed".red().bold().to_string())
        .next()
        .expect("no address is resolved");
    Ok((addr, config))
}
