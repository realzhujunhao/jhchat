use std::{
    fs::{create_dir_all, remove_file},
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
};

use colored::*;
use core::{
    config::{ClientConfig, Config},
    encryption::rsa_impl,
    error::{ExternalError, GlobalResult},
    traits::encrypt::Encrypt,
};

pub type Encryptor = rsa_impl::RsaEncryption;

pub fn config() -> GlobalResult<ClientConfig> {
    let config = ClientConfig::init()?;
    Ok(config)
}

pub fn socket_addr(config: ClientConfig) -> GlobalResult<(SocketAddr, ClientConfig)> {
    let addr = config
        .server_host
        .to_socket_addrs()
        .unwrap_or_else(|_| panic!("{}", "to_socket_addrs() failed".red().bold().to_string()))
        .next()
        .expect("no address is resolved");
    Ok((addr, config))
}

/// read path from config, then create missing directories
pub fn directory(config: ClientConfig) -> GlobalResult<ClientConfig> {
    let rsa_self = Path::new(&config.encryption.self_key_dir);
    let rsa_safe = Path::new(&config.encryption.safe_key_dir);
    let rsa_unsafe = Path::new(&config.encryption.unsafe_key_dir);
    let rsa_self_pub = rsa_self.join("public");
    let rsa_self_priv = rsa_self.join("private");

    // for any directory not exist, create them
    // create_success = false if any of creation fails, true otherwise
    let create_success = [
        rsa_self,
        rsa_safe,
        rsa_unsafe,
        &rsa_self_pub,
        &rsa_self_priv,
    ]
    .into_iter()
    .filter(|dir| !dir.is_dir())
    .map(create_dir_all)
    .all(|r| r.is_ok());

    if !create_success {
        return Err(ExternalError::IO.info("one or more directories cannot be created"));
    }
    Ok(config)
}

// ensures config has key pair, and self key files do exist
pub fn encrypt_key(mut config: ClientConfig) -> GlobalResult<ClientConfig> {
    let self_root_dir = PathBuf::from(&config.encryption.self_key_dir);
    let priv_key_path = self_root_dir.join("private").join(&config.uid);
    let pub_key_path = self_root_dir.join("public").join(&config.uid);

    // ture if both exist, false otherwise
    let exist = [&priv_key_path, &pub_key_path]
        .iter()
        .map(|f| f.try_exists().unwrap_or(false))
        .all(|r| r);

    // if both keys exist, read and mount to config
    if exist {
        let priv_key = Encryptor::read_priv_key(priv_key_path)?;
        let pub_key = Encryptor::read_pub_key(pub_key_path)?;
        config.encryption.rsa_self_priv_key = Some(priv_key);
        config.encryption.rsa_self_pub_key = Some(pub_key);
    }
    // otherwise reset key pair, mount to config, then write to file
    else {
        if priv_key_path.is_file() {
            remove_file(&priv_key_path)?;
        }
        if pub_key_path.is_file() {
            remove_file(&pub_key_path)?;
        }

        println!("{}", "key pair does not exist, generating...".yellow());

        let mut rng = rand::thread_rng();
        let (pub_key, priv_key) =
            Encryptor::generate_key_pair(&mut rng, config.encryption.key_len)?;
        Encryptor::persist_pub_key(&pub_key_path, &pub_key)?;
        Encryptor::persist_priv_key(&priv_key_path, &priv_key)?;
        config.encryption.rsa_self_priv_key = Some(priv_key);
        config.encryption.rsa_self_pub_key = Some(pub_key);

        println!("{}", "key pair initialization has completed".green());
    }
    Ok(config)
}
