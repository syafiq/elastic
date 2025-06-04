#[cfg(feature = "linux")]
mod linux;
#[cfg(any(feature = "wasi", feature = "wasm"))]
pub mod wasm;
#[cfg(feature = "sevsnp")]
mod sev;

mod error;
pub mod aes;

pub use aes::AesKey;

#[cfg(feature = "linux")]
pub use linux::*;
#[cfg(any(feature = "wasi", feature = "wasm"))]
pub use wasm::*;
#[cfg(feature = "sevsnp")]
pub use sev::{SevsnpRng, SevsnpAes};

use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use aes_gcm::aead::Aead;
use aes_gcm::KeyInit;
use std::env;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    #[error("Unsupported operation")]
    UnsupportedOperation,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Operation not permitted")]
    OperationNotPermitted,
    #[error("SEV-SNP not available")]
    SevSnpNotAvailable,
    #[error("SEV-SNP operation failed: {0}")]
    SevSnpOperationFailed(String),
    #[error("SEV-SNP RNG error: {0}")]
    SevSnpRngError(String),
    #[error("SEV-SNP AES error: {0}")]
    SevSnpAesError(String),
    #[error("Unsupported mode")]
    UnsupportedMode,
    #[error("Not implemented")]
    NotImplemented,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid ciphertext")]
    InvalidCiphertext,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    Symmetric,
    Asymmetric,
    Hmac,
}

#[derive(Debug, Clone, Copy)]
pub enum AesMode {
    Cbc,
    Gcm,
}

#[derive(Debug, Clone)]
pub struct KeyConfig {
    pub key_type: KeyType,
    pub key_size: u32,
    pub secure_storage: bool,
}

pub struct Key {
    data: Vec<u8>,
    config: KeyConfig,
}

pub struct ElasticCrypto {
    keys: Mutex<HashMap<u32, Key>>,
    next_handle: Mutex<u32>,
    aes: Mutex<Option<SevsnpAes>>,
}

impl ElasticCrypto {
    pub fn new() -> Result<Self> {
        println!("Initializing ElasticCrypto...");
        println!("Checking for SEV-SNP support...");
        
        let mut aes = None;
        
        #[cfg(feature = "sevsnp")]
        {
            println!("SEV-SNP feature is enabled in build");
            if std::path::Path::new("/dev/sev-guest").exists() {
                println!("SEV-SNP device found at /dev/sev-guest");
                // Initialize SEV-SNP AES hardware
                if let Ok(sev_aes) = SevsnpAes::new(&[0u8; 32]) {
                    aes = Some(sev_aes);
                }
            } else {
                println!("No SEV-SNP device found at /dev/sev-guest");
            }
        }
        
        #[cfg(not(feature = "sevsnp"))]
        {
            println!("SEV-SNP feature is not enabled in build");
        }
        
        Ok(Self {
            keys: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
            aes: Mutex::new(aes),
        })
    }

    fn get_next_handle(&self) -> u32 {
        let mut handle = self.next_handle.lock().unwrap();
        let current = *handle;
        *handle += 1;
        current
    }

    pub fn generate_key(&self, config: KeyConfig) -> Result<u32> {
        let key_data = match config.key_type {
            KeyType::Symmetric => {
                // Generate AES key
                let mut key = vec![0u8; (config.key_size / 8) as usize];
                // TODO: Use proper RNG
                for b in &mut key {
                    *b = rand::random();
                }
                key
            }
            _ => return Err(Error::NotImplemented),
        };

        let handle = self.get_next_handle();
        self.keys.lock().unwrap().insert(handle, Key { data: key_data, config });
        Ok(handle)
    }

    pub fn import_key(&self, key_data: Vec<u8>, config: KeyConfig) -> Result<u32> {
        let handle = self.get_next_handle();
        self.keys.lock().unwrap().insert(handle, Key { data: key_data, config });
        Ok(handle)
    }

    pub fn export_key(&self, handle: u32) -> Result<Vec<u8>> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(Error::KeyNotFound)?;
        if key.config.secure_storage {
            return Err(Error::OperationNotPermitted);
        }
        Ok(key.data.clone())
    }

    pub fn delete_key(&self, handle: u32) -> Result<()> {
        self.keys.lock().unwrap().remove(&handle).ok_or(Error::KeyNotFound)?;
        Ok(())
    }

    pub fn encrypt(&self, handle: u32, data: Vec<u8>) -> Result<Vec<u8>> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(Error::KeyNotFound)?;
        
        match key.config.key_type {
            KeyType::Symmetric => {
                // Use AES-GCM for symmetric encryption
                let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key.data)
                    .map_err(|e| Error::EncryptionError(e.to_string()))?;
                
                // Use a fixed nonce for consistent results across platforms
                // In production, you would use a random nonce
                let nonce = aes_gcm::Nonce::from_slice(b"elastic-nc12"); // 12 bytes
                
                // Use the same encryption method on both platforms
                if env::var("ELASTIC_SEV_SNP").unwrap_or_default() == "1" {
                    // On SEV-SNP, use hardware acceleration if available
                    if let Some(aes) = self.aes.lock().unwrap().as_mut() {
                        aes.encrypt(&data)
                    } else {
                        // Fallback to software implementation if hardware not available
                        cipher.encrypt(nonce, data.as_ref())
                            .map_err(|e| Error::EncryptionError(e.to_string()))
                    }
                } else {
                    // On Linux, use software implementation
                    cipher.encrypt(nonce, data.as_ref())
                        .map_err(|e| Error::EncryptionError(e.to_string()))
                }
            }
            _ => Err(Error::UnsupportedOperation),
        }
    }

    pub fn decrypt(&self, handle: u32, data: Vec<u8>) -> Result<Vec<u8>> {
        let keys = self.keys.lock().unwrap();
        let key = keys.get(&handle).ok_or(Error::KeyNotFound)?;
        
        match key.config.key_type {
            KeyType::Symmetric => {
                // Use AES-GCM for symmetric decryption
                let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key.data)
                    .map_err(|e| Error::DecryptionError(e.to_string()))?;
                
                // Use the same fixed nonce as encryption
                let nonce = aes_gcm::Nonce::from_slice(b"elastic-nc12"); // 12 bytes
                cipher.decrypt(nonce, data.as_ref())
                    .map_err(|e| Error::DecryptionError(e.to_string()))
            }
            _ => Err(Error::UnsupportedOperation),
        }
    }

    pub fn hash(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(hasher.finalize().to_vec())
    }

    pub fn hash_sha512(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        use sha2::{Sha512, Digest};
        let mut hasher = Sha512::new();
        hasher.update(&data);
        Ok(hasher.finalize().to_vec())
    }
}

fn debug_features() {
    println!("[ElasticCrypto] Debug: Checking feature flags...");
    
    #[cfg(feature = "linux")]
    println!("[ElasticCrypto] Debug: 'linux' feature is ENABLED");
    #[cfg(not(feature = "linux"))]
    println!("[ElasticCrypto] Debug: 'linux' feature is DISABLED");
    
    #[cfg(feature = "sevsnp")]
    println!("[ElasticCrypto] Debug: 'sevsnp' feature is ENABLED");
    #[cfg(not(feature = "sevsnp"))]
    println!("[ElasticCrypto] Debug: 'sevsnp' feature is DISABLED");
    
    #[cfg(feature = "wasm")]
    println!("[ElasticCrypto] Debug: 'wasm' feature is ENABLED");
    #[cfg(not(feature = "wasm"))]
    println!("[ElasticCrypto] Debug: 'wasm' feature is DISABLED");

    // Check if we're on Linux
    #[cfg(target_os = "linux")]
    println!("[ElasticCrypto] Debug: Running on Linux OS");
    #[cfg(not(target_os = "linux"))]
    println!("[ElasticCrypto] Debug: Not running on Linux OS");

    // Check if SEV-SNP device exists
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/dev/sev-guest").exists() {
            println!("[ElasticCrypto] Debug: SEV-SNP device exists");
        } else {
            println!("[ElasticCrypto] Debug: SEV-SNP device does not exist");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    #[cfg(feature = "wasi")]
    fn test_wasm_crypto() {
        let crypto = WasmCrypto::new();
        let has_sevsnp = Path::new("/dev/sev-guest").exists();
        assert_eq!(crypto.is_sevsnp(), has_sevsnp, "SEV-SNP detection should match device existence");

        // Test key generation
        let key = crypto.generate_key().unwrap();
        assert_eq!(key.len(), 32);

        // Test encryption/decryption
        let data = b"Hello, Crypto!";
        let encrypted = crypto.encrypt(&key, data, AesMode::GCM).unwrap();
        let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM).unwrap();
        assert_eq!(data, &decrypted[..]);
    }

    #[test]
    #[cfg(feature = "sevsnp")]
    fn test_sevsnp_rng() {
        let rng = SevsnpRng::new();
        let has_sevsnp = Path::new("/dev/sev-guest").exists();
        
        if has_sevsnp {
            let mut rng = rng.unwrap();
            let bytes = rng.get_random_bytes(32).unwrap();
            assert_eq!(bytes.len(), 32);
        } else {
            assert!(matches!(rng, Err(Error::SevsnpNotAvailable)));
        }
    }

    #[test]
    #[cfg(feature = "sevsnp")]
    fn test_sevsnp_aes() {
        let has_sevsnp = Path::new("/dev/sev-guest").exists();
        let aes = SevsnpAes::new(&[0u8; 32]);
        
        if has_sevsnp {
            let mut aes = aes.unwrap();
            let data = b"test data";
            let encrypted = aes.encrypt(data).unwrap();
            let decrypted = aes.decrypt(&encrypted).unwrap();
            assert_eq!(data, &decrypted[..]);
        } else {
            assert!(matches!(aes, Err(Error::SevsnpNotAvailable)));
        }
    }

    #[test]
    fn test_error_handling() {
        // Test invalid key length
        let key = vec![0u8; 16]; // Too short
        assert!(matches!(AesKey::new(&key), Err(Error::InvalidKeyLength)));

        // Test unsupported operation
        let key = vec![0u8; 32];
        let aes = AesKey::new(&key).unwrap();
        assert!(matches!(aes.encrypt(b"test", AesMode::CBC), Err(Error::UnsupportedOperation)));
    }
} 