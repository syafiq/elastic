#[cfg(target_os = "linux")]
use sev::firmware::guest::Firmware;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use crate::Error;
use std::fmt;
use rand::{RngCore};
use rand::rngs::ThreadRng;

// SEV-SNP specific implementation
#[derive(Debug, Clone)]
pub struct SevsnpRng {
    rng: ThreadRng,
}

impl SevsnpRng {
    pub fn new() -> Result<Self, Error> {
        Ok(Self { rng: rand::thread_rng() })
    }

    pub fn get_random_bytes(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0u8; len];
        self.rng.fill_bytes(&mut bytes);
        Ok(bytes)
    }
}

impl rand::RngCore for SevsnpRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).unwrap();
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.fill_bytes(dest);
        Ok(())
    }
}

// SEV-SNP specific AES implementation
#[derive(Clone)]
pub struct SevsnpAes {
    _key: Vec<u8>,
    #[cfg(not(target_os = "linux"))]
    cipher: Aes256Gcm,
}

impl fmt::Debug for SevsnpAes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SevsnpAes")
            .field("key_len", &self._key.len())
            .finish()
    }
}

impl SevsnpAes {
    pub fn new(key: &[u8]) -> Result<Self, Error> {
        #[cfg(target_os = "linux")]
        {
            Ok(Self {
                _key: key.to_vec(),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::EncryptionError("Failed to create cipher".to_string()))?;
            Ok(Self {
                _key: key.to_vec(),
                cipher,
            })
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            // For SEV-SNP, we'll use AES-GCM as a fallback since direct encryption is not available
            let cipher = Aes256Gcm::new_from_slice(&self._key).map_err(|_| Error::EncryptionError("Failed to create cipher".to_string()))?;
            let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a random nonce
            cipher.encrypt(nonce, data)
                .map_err(|_| Error::EncryptionError("Encryption failed".to_string()))
        }

        #[cfg(not(target_os = "linux"))]
        {
            let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a random nonce
            self.cipher.encrypt(nonce, data)
                .map_err(|_| Error::EncryptionError("Encryption failed".to_string()))
        }
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            // For SEV-SNP, we'll use AES-GCM as a fallback since direct decryption is not available
            let cipher = Aes256Gcm::new_from_slice(&self._key).map_err(|_| Error::DecryptionError("Decryption failed".to_string()))?;
            let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use the same nonce as encryption
            cipher.decrypt(nonce, data)
                .map_err(|_| Error::DecryptionError("Decryption failed".to_string()))
        }

        #[cfg(not(target_os = "linux"))]
        {
            let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use the same nonce as encryption
            self.cipher.decrypt(nonce, data)
                .map_err(|_| Error::DecryptionError("Decryption failed".to_string()))
        }
    }

    pub fn key(&self) -> &[u8] {
        &self._key
    }
} 