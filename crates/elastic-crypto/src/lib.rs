use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wit_bindgen::rt::vec::Vec;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey},
    RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt,
    signature::{SignatureEncoding, RandomizedSigner, Verifier},
    pss::{BlindedSigningKey, VerifyingKey, Signature},
    sha2::Sha256,
};
use sha2::{Sha512, Digest};
use hmac::{Hmac, Mac};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key")]
    InvalidKey,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Signing failed")]
    SigningFailed,
    #[error("Verification failed")]
    VerificationFailed,
    #[error("MAC calculation failed")]
    MacFailed,
    #[error("Unsupported algorithm")]
    UnsupportedAlgorithm,
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
    Sha256,
    Sha512,
}

#[derive(Debug)]
pub enum CryptoKey {
    Symmetric(Vec<u8>),
    Private(RsaPrivateKey),
    Public(RsaPublicKey),
}

pub struct CryptoContext {
    keys: Arc<Mutex<HashMap<u32, CryptoKey>>>,
    next_handle: Arc<Mutex<u32>>,
}

impl CryptoContext {
    pub fn new() -> Self {
        CryptoContext {
            keys: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(1)),
        }
    }

    pub fn load_key(&self, key: &[u8], key_type: KeyType, algorithm: Algorithm) -> Result<u32, CryptoError> {
        println!("Loading key with type: {:?}, algorithm: {:?}, key length: {}", key_type, algorithm, key.len());
        let crypto_key = match (key_type, algorithm) {
            (KeyType::Symmetric, Algorithm::Aes256Gcm) => {
                if key.len() != 32 {
                    println!("Invalid symmetric key length: {}", key.len());
                    return Err(CryptoError::InvalidKey);
                }
                CryptoKey::Symmetric(key.to_vec())
            },
            (KeyType::Asymmetric, Algorithm::Rsa2048) => {
                println!("Attempting to load RSA key of length {}", key.len());
                // First try loading as private key
                match RsaPrivateKey::from_pkcs8_der(key) {
                    Ok(private_key) => {
                        println!("Successfully loaded RSA private key");
                        CryptoKey::Private(private_key)
                    },
                    Err(private_err) => {
                        println!("Failed to load private key: {:?}", private_err);
                        println!("Attempting to load as public key...");
                        match RsaPublicKey::from_public_key_der(key) {
                            Ok(public_key) => {
                                println!("Successfully loaded RSA public key");
                                CryptoKey::Public(public_key)
                            },
                            Err(public_err) => {
                                println!("Failed to load public key: {:?}", public_err);
                                return Err(CryptoError::InvalidKey);
                            }
                        }
                    }
                }
            },
            _ => {
                println!("Unsupported algorithm combination: {:?}, {:?}", key_type, algorithm);
                return Err(CryptoError::UnsupportedAlgorithm);
            }
        };

        let mut handles = self.keys.lock().unwrap();
        let mut next_handle = self.next_handle.lock().unwrap();
        let handle = *next_handle;
        *next_handle += 1;

        println!("Assigning handle {} to key", handle);
        handles.insert(handle, crypto_key);
        Ok(handle)
    }

    pub fn unload_key(&self, handle: u32) -> Result<(), CryptoError> {
        let mut keys = self.keys.lock().unwrap();
        keys.remove(&handle).ok_or_else(|| CryptoError::InvalidKey)?;
        Ok(())
    }

    pub fn public_key_encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Public(public_key) => {
                let mut rng = OsRng;
                println!("Attempting to encrypt with public key");
                public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)
                    .map_err(|e| {
                        println!("Encryption error: {:?}", e);
                        CryptoError::EncryptionFailed
                    })
            },
            _ => {
                println!("Invalid key type for encryption");
                Err(CryptoError::InvalidKey)
            }
        }
    }

    pub fn public_key_decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or_else(|| CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Private(private_key) => {
                println!("Attempting to decrypt with private key");
                private_key.decrypt(Pkcs1v15Encrypt, data)
                    .map_err(|e| {
                        println!("Decryption error: {:?}", e);
                        CryptoError::DecryptionFailed
                    })
            },
            _ => {
                println!("Invalid key type for decryption");
                Err(CryptoError::InvalidKey)
            }
        }
    }

    pub fn sign(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Private(private_key) => {
                let mut rng = OsRng;
                println!("Attempting to sign with private key");
                let signing_key = BlindedSigningKey::<Sha256>::new(private_key.clone());
                signing_key.try_sign_with_rng(&mut rng, data)
                    .map(|sig| sig.to_vec())
                    .map_err(|e| {
                        println!("Signing error: {:?}", e);
                        CryptoError::SigningFailed
                    })
            }
            _ => {
                println!("Invalid key type for signing");
                Err(CryptoError::InvalidKey)
            }
        }
    }

    pub fn verify(&self, handle: u32, data: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Public(public_key) => {
                println!("Attempting to verify with public key");
                let verifying_key = VerifyingKey::<Sha256>::new(public_key.clone());
                let sig = Signature::try_from(signature)
                    .map_err(|_| CryptoError::VerificationFailed)?;
                Ok(verifying_key.verify(data, &sig).is_ok())
            }
            _ => {
                println!("Invalid key type for verification");
                Err(CryptoError::InvalidKey)
            }
        }
    }

    pub fn symmetric_encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Symmetric(key_data) => {
                let key = Key::<Aes256Gcm>::from_slice(key_data);
                let cipher = Aes256Gcm::new(key);
                
                // Generate a random nonce
                let mut nonce_bytes = [0u8; 12];
                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);
                
                // Encrypt the data
                let ciphertext = cipher.encrypt(nonce, data)
                    .map_err(|_| CryptoError::EncryptionFailed)?;
                
                // Combine nonce and ciphertext
                let mut result = Vec::with_capacity(12 + ciphertext.len());
                result.extend_from_slice(&nonce_bytes);
                result.extend_from_slice(&ciphertext);
                Ok(result)
            },
            _ => Err(CryptoError::InvalidKey),
        }
    }

    pub fn symmetric_decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 12 {
            return Err(CryptoError::DecryptionFailed);
        }

        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Symmetric(key_data) => {
                let key = Key::<Aes256Gcm>::from_slice(key_data);
                let cipher = Aes256Gcm::new(key);
                
                // Split data into nonce and ciphertext
                let nonce = Nonce::from_slice(&data[..12]);
                let ciphertext = &data[12..];
                
                // Decrypt the data
                cipher.decrypt(nonce, ciphertext)
                    .map_err(|_| CryptoError::DecryptionFailed)
            },
            _ => Err(CryptoError::InvalidKey),
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
            _ => Err(CryptoError::UnsupportedAlgorithm),
        }
    }

    pub fn calculate_mac(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(CryptoError::InvalidKey)?;

        match key {
            CryptoKey::Symmetric(key_data) => {
                let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key_data)
                    .map_err(|_| CryptoError::MacFailed)?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            _ => Err(CryptoError::InvalidKey),
        }
    }
}

impl Default for CryptoContext {
    fn default() -> Self {
        Self::new()
    }
} 