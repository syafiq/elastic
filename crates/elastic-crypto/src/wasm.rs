use std::env;
use crate::Error;
use rand::{Rng, RngCore};

// WASM-specific implementation that uses SEV-SNP when available
pub struct WasmCrypto {
    pub is_sevsnp: bool,
}

impl WasmCrypto {
    pub fn new() -> Result<Self, Error> {
        let is_sevsnp = env::var("SEV_SNP").is_ok();
        Ok(Self { is_sevsnp })
    }

    pub fn generate_key(&self) -> Result<Vec<u8>, Error> {
        if self.is_sevsnp {
            // TODO: Use SEV-SNP RNG when implemented
            Err(Error::NotImplemented)
        } else {
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key[..]);
            Ok(key)
        }
    }

    pub fn encrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
        if self.is_sevsnp {
            // TODO: Use SEV-SNP AES when implemented
            Err(Error::NotImplemented)
        } else {
            let aes_key = AesKey::new(key)?;
            aes_key.encrypt(data, mode)
        }
    }

    pub fn decrypt(&self, key: &[u8], encrypted_data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
        if self.is_sevsnp {
            // TODO: Use SEV-SNP AES when implemented
            Err(Error::NotImplemented)
        } else {
            let aes_key = AesKey::new(key)?;
            aes_key.decrypt(encrypted_data, mode)
        }
    }
}

// Re-export the standard AES implementation for non-SEV-SNP environments
pub use crate::aes::*; 