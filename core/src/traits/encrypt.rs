use std::{io::{Write, Read}, path::Path};

use crate::error::{ClientError, GlobalResult};
use async_trait::async_trait;
use rand::rngs::ThreadRng;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

/// implement this trait to change encryption algorithm
#[async_trait]
pub trait Encrypt {
    type PublicKey: Send + Sync;
    type PrivateKey: Send + Sync;

    fn encrypt(
        raw: &[u8],
        pub_key: &Self::PublicKey,
        rand: &mut ThreadRng,
    ) -> GlobalResult<Vec<u8>>;
    fn decrypt(ciphertext: &[u8], priv_key: &Self::PrivateKey) -> GlobalResult<Vec<u8>>;

    fn generate_key_pair(
        rand: &mut ThreadRng,
        len: usize,
    ) -> GlobalResult<(Self::PublicKey, Self::PrivateKey)>;

    fn export_pub_key(key: &Self::PublicKey) -> GlobalResult<Vec<u8>>;
    fn export_priv_key(key: &Self::PrivateKey) -> GlobalResult<Vec<u8>>;

    fn import_pub_key(bytes: &[u8]) -> GlobalResult<Self::PublicKey>;
    fn import_priv_key(bytes: &[u8]) -> GlobalResult<Self::PrivateKey>;

    fn encrypt_from_str(
        raw: &str,
        pub_key: &Self::PublicKey,
        rand: &mut ThreadRng,
    ) -> GlobalResult<Vec<u8>> {
        Self::encrypt(raw.as_bytes(), pub_key, rand)
    }

    fn decrypt_to_string(ciphertext: &[u8], priv_key: &Self::PrivateKey) -> GlobalResult<String> {
        let bytes = Self::decrypt(ciphertext, priv_key)?;
        let raw_str = String::from_utf8(bytes)
            .map_err(|_| ClientError::Decryption.info("ciphertext not utf8"))?;
        Ok(raw_str)
    }

    fn persist_pub_key(path: impl AsRef<Path>, key: &Self::PublicKey) -> GlobalResult<()> {
        let bytes = Self::export_pub_key(key)?;
        Self::sync_write(&bytes, path)
    }


    fn persist_priv_key(path: impl AsRef<Path>, key: &Self::PrivateKey) -> GlobalResult<()> {
        let bytes = Self::export_priv_key(key)?;
        Self::sync_write(&bytes, path)
    }

    async fn async_persist_pub_key(
        path: impl AsRef<Path> + Send + Sync,
        key: &Self::PublicKey,
    ) -> GlobalResult<()> {
        let bytes = Self::export_pub_key(key)?;
        Self::async_write(&bytes, path).await
    }

    async fn async_persist_priv_key(
        path: impl AsRef<Path> + Send + Sync,
        key: &Self::PrivateKey,
    ) -> GlobalResult<()> {
        let bytes = Self::export_priv_key(key)?;
        Self::async_write(&bytes, path).await
    }

    fn read_pub_key(path: impl AsRef<Path>) -> GlobalResult<Self::PublicKey> {
        let buf = Self::sync_read(path)?;
        let pub_key = Self::import_pub_key(&buf)?;
        Ok(pub_key)
    }

    fn read_priv_key(path: impl AsRef<Path>) -> GlobalResult<Self::PrivateKey> {
        let buf = Self::sync_read(path)?;
        let priv_key = Self::import_priv_key(&buf)?;
        Ok(priv_key)
    }

    async fn async_read_pub_key(path: impl AsRef<Path> + Send + Sync) -> GlobalResult<Self::PublicKey> {
        let buf = Self::async_read(path).await?;
        let pub_key = Self::import_pub_key(&buf)?;
        Ok(pub_key)
    }

    async fn async_read_priv_key(path: impl AsRef<Path> + Send + Sync) -> GlobalResult<Self::PrivateKey> {
        let buf = Self::async_read(path).await?;
        let priv_key = Self::import_priv_key(&buf)?;
        Ok(priv_key)
    }

    // sync IO blocks current thread, async IO blocks current task
    fn sync_write(bytes: &[u8], path: impl AsRef<Path>) -> GlobalResult<()> {
        std::fs::File::create(path).and_then(|mut f| f.write_all(bytes))?;
        Ok(())
    }

    fn sync_read(path: impl AsRef<Path>) -> GlobalResult<Vec<u8>> {
        let mut buf = Vec::new();
        std::fs::File::open(path).and_then(|mut f| f.read_to_end(&mut buf))?;
        Ok(buf)
    }

    async fn async_write(bytes: &[u8], path: impl AsRef<Path> + Send + Sync) -> GlobalResult<()> {
        let mut f = tokio::fs::File::create(path).await?;
        f.write_all(bytes).await?;
        Ok(())
    }

    async fn async_read(path: impl AsRef<Path> + Send + Sync) -> GlobalResult<Vec<u8>> {
        let mut buf = Vec::new();
        let mut f = tokio::fs::File::open(path).await?;
        f.read_to_end(&mut buf).await?;
        Ok(buf)
    }

}
