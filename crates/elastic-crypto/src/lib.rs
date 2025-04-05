#[cfg(feature = "linux")]
mod linux;
#[cfg(feature = "wasm")]
pub mod wasm;
#[cfg(feature = "sevsnp")]
mod sev;

mod error;
pub use error::Error;
pub use crate::aes::AesMode;

#[cfg(feature = "linux")]
pub use linux::*;
#[cfg(feature = "wasm")]
pub use wasm::*;
#[cfg(feature = "sevsnp")]
pub use sev::*;

pub mod aes {
use aes_gcm::{
    aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::RngCore;
    use crate::Error;

    #[derive(Debug)]
    pub struct AesKey(Vec<u8>);

#[derive(Debug, Clone, Copy)]
    pub enum AesMode {
        CBC,
        GCM,
    }

    impl AesKey {
        pub fn new(key_bytes: &[u8]) -> Result<Self, Error> {
            if key_bytes.len() != 32 {
                return Err(Error::InvalidKeyLength);
            }
            Ok(AesKey(key_bytes.to_vec()))
        }

        pub fn encrypt(&self, data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
            match mode {
                AesMode::GCM => {
                    let cipher = Aes256Gcm::new_from_slice(&self.0)
                        .map_err(|_| Error::EncryptionError)?;
                    
                    let mut nonce_bytes = [0u8; 12];
                    rand::thread_rng().fill_bytes(&mut nonce_bytes);
                    let nonce = Nonce::from_slice(&nonce_bytes);
                    
                    let ciphertext = cipher
                        .encrypt(nonce, data)
                        .map_err(|_| Error::EncryptionError)?;
                    
                    // Prepend nonce to ciphertext
                    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
                    result.extend_from_slice(&nonce_bytes);
                    result.extend_from_slice(&ciphertext);
                    Ok(result)
                }
                AesMode::CBC => {
                    // For this example, we'll return an error as CBC is not implemented yet
                    Err(Error::UnsupportedOperation)
                }
            }
        }

        pub fn decrypt(&self, encrypted_data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
            match mode {
                AesMode::GCM => {
                    if encrypted_data.len() < 12 {
                        return Err(Error::DecryptionError);
                    }
                    
                    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
                    let cipher = Aes256Gcm::new_from_slice(&self.0)
                        .map_err(|_| Error::DecryptionError)?;
                    
                    let nonce = Nonce::from_slice(nonce_bytes);
                    cipher
                        .decrypt(nonce, ciphertext)
                        .map_err(|_| Error::DecryptionError)
                }
                AesMode::CBC => {
                    // For this example, we'll return an error as CBC is not implemented yet
                    Err(Error::UnsupportedOperation)
                }
            }
        }
    }
}

pub trait Crypto {
    fn generate_key(&self) -> Result<Vec<u8>, Error>;
    fn encrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error>;
    fn decrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    #[cfg(feature = "wasm")]
    fn test_wasm_crypto_non_sevsnp() {
        // Clear SEV_SNP env var to simulate non-SEV-SNP environment
        env::remove_var("SEV_SNP");
        assert!(env::var("SEV_SNP").is_err(), "SEV_SNP environment variable should be unset");
        
        let crypto = WasmCrypto::new().unwrap();
        assert!(!crypto.is_sevsnp);

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
    #[cfg(feature = "wasm")]
    fn test_wasm_crypto_sevsnp() {
        // First test without SEV-SNP to ensure clean state
        env::remove_var("SEV_SNP");
        let crypto = WasmCrypto::new().unwrap();
        assert!(!crypto.is_sevsnp);

        // Now test with SEV-SNP
        env::set_var("SEV_SNP", "1");
        let crypto = WasmCrypto::new().unwrap();
        assert!(crypto.is_sevsnp);

        // In a real SEV-SNP environment, these operations should succeed
        // For now, they return NotImplemented since we haven't implemented the SEV-SNP specific crypto yet
        let key = crypto.generate_key();
        assert!(matches!(key, Err(Error::NotImplemented)), "SEV-SNP key generation should be implemented");

        let test_key = [0u8; 32];
        let encrypt_result = crypto.encrypt(&test_key, b"test", AesMode::GCM);
        assert!(matches!(encrypt_result, Err(Error::NotImplemented)), "SEV-SNP encryption should be implemented");

        let decrypt_result = crypto.decrypt(&test_key, b"test", AesMode::GCM);
        assert!(matches!(decrypt_result, Err(Error::NotImplemented)), "SEV-SNP decryption should be implemented");
    }

    #[test]
    #[cfg(feature = "linux")]
    fn test_sevsnp_rng() {
        let rng = SevsnpRng::new();
        // This should fail since SEV-SNP implementation is not complete
        assert!(matches!(rng, Err(Error::SevsnpNotAvailable)));
    }

    #[test]
    #[cfg(feature = "linux")]
    fn test_sevsnp_aes() {
        let aes = SevsnpAes::new(&[0u8; 32]);
        // This should fail since SEV-SNP implementation is not complete
        assert!(matches!(aes, Err(Error::SevsnpNotAvailable)));
    }

    #[test]
    fn test_error_handling() {
        // Test invalid key length
        let key = vec![0u8; 16]; // Too short
        assert!(matches!(AesKey::new(&key), Err(Error::InvalidKeyLength)));

        // Test unsupported operation
        let key = AesKey::new(&[0u8; 32]).unwrap();
        assert!(matches!(key.encrypt(b"test", AesMode::CBC), Err(Error::UnsupportedOperation)));
        assert!(matches!(key.decrypt(b"test", AesMode::CBC), Err(Error::UnsupportedOperation)));
    }
} 