use std::collections::HashMap;
use std::sync::Mutex;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rsa::{
    RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt,
    pkcs8::{EncodePublicKey, LineEnding},
};
use sha2::{Sha256, Sha512, Digest};
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
}

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    Symmetric,
    Asymmetric,
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
}

impl CryptoContext {
    pub fn new() -> Self {
        CryptoContext {
            keys: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
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
            (KeyType::Asymmetric, Algorithm::Rsa2048) => {
                let private_key = RsaPrivateKey::from_pkcs8_der(key)
                    .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                CryptoKey::AsymmetricPrivate(private_key)
            },
            (KeyType::Asymmetric, Algorithm::Rsa4096) => {
                let private_key = RsaPrivateKey::from_pkcs8_der(key)
                    .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                CryptoKey::AsymmetricPrivate(private_key)
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

    pub fn sign(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::AsymmetricPrivate(private_key) => {
                let mut rng = rand::thread_rng();
                private_key.sign(Pkcs1v15Encrypt, &mut rng, data)
                    .map_err(|e| CryptoError::SigningError(e.to_string()))
            },
            _ => Err(CryptoError::InvalidKey("Not a private key".to_string())),
        }
    }

    pub fn verify(&self, handle: u32, data: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::AsymmetricPublic(public_key) => {
                public_key.verify(Pkcs1v15Encrypt, data, signature)
                    .map_err(|e| CryptoError::VerificationError(e.to_string()))
            },
            _ => Err(CryptoError::InvalidKey("Not a public key".to_string())),
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

    pub fn calculate_mac(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey("Key not found".to_string()))?;

        match key {
            CryptoKey::Symmetric(cipher) => {
                let key = cipher.key();
                let mut mac = Hmac::<Sha256>::new_from_slice(key)
                    .map_err(|e| CryptoError::MacError(e.to_string()))?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            },
            _ => Err(CryptoError::InvalidKey("Not a symmetric key".to_string())),
        }
    }
} 