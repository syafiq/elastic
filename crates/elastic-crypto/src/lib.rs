#[cfg(feature = "linux")]
mod linux;
#[cfg(feature = "wasm")]
mod wasm;

mod error;
pub use error::Error;

#[cfg(feature = "linux")]
pub use linux::*;
#[cfg(feature = "wasm")]
pub use wasm::*;

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