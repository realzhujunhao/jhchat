use crate::error::{ClientError, GlobalResult};
use rand::rngs::ThreadRng;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};

use crate::traits::encrypt::Encrypt;
use async_trait::async_trait;

pub struct RsaEncryption;

#[async_trait]
impl Encrypt for RsaEncryption {
    type PublicKey = RsaPublicKey;
    type PrivateKey = RsaPrivateKey;

    fn encrypt(
        raw: &[u8],
        pub_key: &Self::PublicKey,
        rand: &mut ThreadRng,
    ) -> GlobalResult<Vec<u8>> {
        let ciphertext = pub_key
            .encrypt(rand, Pkcs1v15Encrypt, raw)
            .map_err(|_| ClientError::Encryption)?;
        Ok(ciphertext)
    }

    fn decrypt(ciphertext: &[u8], priv_key: &Self::PrivateKey) -> GlobalResult<Vec<u8>> {
        let raw = priv_key
            .decrypt(Pkcs1v15Encrypt, ciphertext)
            .map_err(|_| ClientError::Decryption)?;
        Ok(raw)
    }

    fn import_pub_key(bytes: &[u8]) -> GlobalResult<Self::PublicKey> {
        let pem_str = String::from_utf8(bytes.to_vec())
            .map_err(|_| ClientError::EncryptKeyPersistence.info("pem not utf8"))?;
        let pub_key = RsaPublicKey::from_public_key_pem(&pem_str).map_err(|_| {
            ClientError::EncryptKeyPersistence.info("cannot deserialize pem into public key")
        })?;
        Ok(pub_key)
    }

    fn import_priv_key(bytes: &[u8]) -> GlobalResult<Self::PrivateKey> {
        let pem_str = String::from_utf8(bytes.to_vec())
            .map_err(|_| ClientError::EncryptKeyPersistence.info("pem not utf8"))?;
        let priv_key = RsaPrivateKey::from_pkcs8_pem(&pem_str).map_err(|_| {
            ClientError::EncryptKeyPersistence.info("cannot deserialize pem into private key")
        })?;
        Ok(priv_key)
    }

    fn export_pub_key(key: &Self::PublicKey) -> GlobalResult<Vec<u8>> {
        let pem_str = key.to_public_key_pem(LineEnding::CRLF).map_err(|_| {
            ClientError::EncryptKeyPersistence.info("cannot generate pem for public key")
        })?;
        Ok(pem_str.as_bytes().to_vec())
    }

    fn export_priv_key(key: &Self::PrivateKey) -> GlobalResult<Vec<u8>> {
        let pem_str = key.to_pkcs8_pem(LineEnding::CRLF).map_err(|_| {
            ClientError::EncryptKeyPersistence.info("cannot generate pem for private key")
        })?;
        Ok(pem_str.as_bytes().to_vec())
    }

    fn generate_key_pair(
        rand: &mut ThreadRng,
        len: usize,
    ) -> GlobalResult<(Self::PublicKey, Self::PrivateKey)> {
        let priv_key = RsaPrivateKey::new(rand, len)?;
        let pub_key = RsaPublicKey::from(&priv_key);
        Ok((pub_key, priv_key))
    }
}
