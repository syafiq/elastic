use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use wit_bindgen::rt::string::String;
use wit_bindgen::rt::vec::Vec;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use rsa::{
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey, Pkcs1v15Sign,
};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    #[error("Signing error: {0}")]
    SigningError(String),
    #[error("Verification error: {0}")]
    VerificationError(String),
    #[error("Hashing error: {0}")]
    HashingError(String),
    #[error("MAC error: {0}")]
    MacError(String),
    #[error("Unsupported operation")]
    UnsupportedOperation,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    Symmetric,
    Asymmetric,
    Rsa2048,
    Rsa4096,
    Hmac256,
}

#[derive(Debug, Clone, Copy)]
pub enum Algorithm {
    Aes256Gcm,
    Rsa2048,
    Rsa4096,
    Sha256,
    Sha512,
    HmacSha256,
}

enum CryptoKey {
    Symmetric(Aes256Gcm),
    AsymmetricPrivate(RsaPrivateKey),
    AsymmetricPublic(RsaPublicKey),
}

pub struct CryptoContext {
    keys: Mutex<HashMap<u32, CryptoKey>>,
    next_handle: Mutex<u32>,
    key_type: KeyType,
}

impl CryptoContext {
    pub fn new() -> Self {
        CryptoContext {
            keys: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
            key_type: KeyType::Symmetric,
        }
    }

    pub fn load_key(&self, key: &[u8], key_type: KeyType, algorithm: Algorithm) -> Result<u32, CryptoError> {
        let crypto_key = match (key_type, algorithm) {
            (KeyType::Symmetric, Algorithm::Aes256Gcm) => {
                if key.len() != 32 {
                    return Err(CryptoError::InvalidKey("AES-256-GCM requires a 32-byte key".to_string()));
                }
                let key = Key::<Aes256Gcm>::from_slice(key);
                CryptoKey::Symmetric(Aes256Gcm::new(key))
            },
            (KeyType::Asymmetric, Algorithm::Rsa2048) | (KeyType::Asymmetric, Algorithm::Rsa4096) => {
                match RsaPrivateKey::from_pkcs8_der(key) {
                    Ok(private_key) => CryptoKey::AsymmetricPrivate(private_key),
                    Err(_) => {
                        // Try loading as public key if private key loading fails
                        let public_key = RsaPublicKey::from_public_key_der(key)
                            .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                        CryptoKey::AsymmetricPublic(public_key)
                    }
                }
            },
            _ => return Err(CryptoError::InvalidAlgorithm("Unsupported key type and algorithm combination".to_string())),
        };

        let mut handles = self.keys.lock().unwrap();
        let mut next_handle = self.next_handle.lock().unwrap();
        let handle = *next_handle;
        *next_handle += 1;

        handles.insert(handle, crypto_key);
        Ok(handle)
    }

    pub fn unload_key(&self, handle: u32) -> Result<(), CryptoError> {
        let mut keys = self.keys.lock().unwrap();
        keys.remove(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;
        Ok(())
    }

    pub fn public_key_encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::AsymmetricPublic(public_key) => {
                let mut rng = rand::thread_rng();
                public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)
                    .map_err(|e| CryptoError::EncryptionError(e.to_string()))
            },
            _ => Err(CryptoError::InvalidKey("Not a public key".to_string())),
        }
    }

    pub fn public_key_decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::AsymmetricPrivate(private_key) => {
                private_key.decrypt(Pkcs1v15Encrypt, data)
                    .map_err(|e| CryptoError::DecryptionError(e.to_string()))
            },
            _ => Err(CryptoError::InvalidKey("Not a private key".to_string())),
        }
    }

    pub fn sign(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self.key_type {
            KeyType::Rsa2048 | KeyType::Rsa4096 => {
                let private_key = RsaPrivateKey::from_pkcs8_der(key)
                    .map_err(|_| CryptoError::InvalidKey)?;
                let mut hasher = Sha256::new();
                hasher.update(data);
                let hash = hasher.finalize();
                let signing_key = Pkcs1v15Sign::new();
                let signature = signing_key.sign(&hash, &private_key)
                    .map_err(|_| CryptoError::SigningError("Signing failed".to_string()))?;
                Ok(signature)
            }
            _ => Err(CryptoError::UnsupportedOperation),
        }
    }

    pub fn verify(&self, data: &[u8], signature: &[u8], key: &[u8]) -> Result<bool, CryptoError> {
        match self.key_type {
            KeyType::Rsa2048 | KeyType::Rsa4096 => {
                let public_key = RsaPublicKey::from_public_key_der(key)
                    .map_err(|_| CryptoError::InvalidKey)?;
                let mut hasher = Sha256::new();
                hasher.update(data);
                let hash = hasher.finalize();
                let verifying_key = Pkcs1v15Sign::new();
                Ok(verifying_key.verify(&hash, signature, &public_key).is_ok())
            }
            _ => Err(CryptoError::UnsupportedOperation),
        }
    }

    pub fn symmetric_encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::Symmetric(cipher) => {
                let nonce = Nonce::from_slice(b"unique nonce"); // In production, use a unique nonce per encryption
                cipher.encrypt(nonce, data)
                    .map_err(|e| CryptoError::EncryptionError(e.to_string()))
            },
            _ => Err(CryptoError::InvalidKey("Not a symmetric key".to_string())),
        }
    }

    pub fn symmetric_decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::Symmetric(cipher) => {
                let nonce = Nonce::from_slice(b"unique nonce"); // In production, use a unique nonce per encryption
                cipher.decrypt(nonce, data)
                    .map_err(|e| CryptoError::DecryptionError(e.to_string()))
            },
            _ => Err(CryptoError::InvalidKey("Not a symmetric key".to_string())),
        }
    }

    pub fn hash(&self, algorithm: Algorithm, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match algorithm {
            Algorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            },
            Algorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            },
            _ => Err(CryptoError::InvalidAlgorithm("Unsupported hashing algorithm".to_string())),
        }
    }

    pub fn calculate_mac(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self.key_type {
            KeyType::Hmac256 => {
                let mut mac = Hmac::<Sha256>::new(key.into());
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            _ => Err(CryptoError::UnsupportedOperation),
        }
    }
} 