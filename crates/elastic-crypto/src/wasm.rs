use crate::{Error, Crypto};
use crate::aes::AesMode;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

// WASM-specific implementation that uses SEV-SNP when available
pub struct WasmCrypto {
    is_sevsnp: bool,
}

impl WasmCrypto {
    pub fn new() -> Self {
        println!("Checking SEV-SNP feature...");
        let is_sevsnp;
        #[cfg(feature = "sevsnp")]
        {
            println!("SEV-SNP feature is enabled");
            is_sevsnp = true;
        }
        #[cfg(not(feature = "sevsnp"))]
        {
            println!("SEV-SNP feature is NOT enabled");
            is_sevsnp = false;
        }
        Self { is_sevsnp }
    }

    pub fn is_sevsnp(&self) -> bool {
        self.is_sevsnp
    }

    fn generate_sevsnp_key(&self) -> Result<Vec<u8>, Error> {
        // In a real SEV-SNP environment, we would use the SEV-SNP RNG
        // For now, we'll use the standard RNG
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key)
    }

    fn encrypt_sevsnp(&self, key: &[u8], data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        // In a real SEV-SNP environment, we would use SEV-SNP specific encryption
        // For now, we'll use standard AES-GCM
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::InvalidKeyLength)?;
        let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a random nonce
        cipher.encrypt(nonce, data)
            .map_err(|_| Error::EncryptionError)
    }

    fn decrypt_sevsnp(&self, key: &[u8], data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        // In a real SEV-SNP environment, we would use SEV-SNP specific decryption
        // For now, we'll use standard AES-GCM
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::InvalidKeyLength)?;
        let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use the same nonce as encryption
        cipher.decrypt(nonce, data)
            .map_err(|_| Error::DecryptionError)
    }
}

impl Crypto for WasmCrypto {
    fn generate_key(&self) -> Result<Vec<u8>, Error> {
        if self.is_sevsnp {
            println!("Using SEV-SNP key generation");
            self.generate_sevsnp_key()
        } else {
            println!("Using standard key generation");
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            Ok(key)
        }
    }

    fn encrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
        if key.len() != 32 {
            return Err(Error::InvalidKeyLength);
        }

        if self.is_sevsnp {
            println!("Using SEV-SNP encryption");
            self.encrypt_sevsnp(key, data, mode)
        } else {
            println!("Using standard encryption");
            match mode {
                AesMode::GCM => {
                    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::InvalidKeyLength)?;
                    let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a random nonce
                    cipher.encrypt(nonce, data)
                        .map_err(|_| Error::EncryptionError)
                }
                _ => Err(Error::UnsupportedOperation),
            }
        }
    }

    fn decrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
        if key.len() != 32 {
            return Err(Error::InvalidKeyLength);
        }

        if self.is_sevsnp {
            println!("Using SEV-SNP decryption");
            self.decrypt_sevsnp(key, data, mode)
        } else {
            println!("Using standard decryption");
            match mode {
                AesMode::GCM => {
                    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::InvalidKeyLength)?;
                    let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use the same nonce as encryption
                    cipher.decrypt(nonce, data)
                        .map_err(|_| Error::DecryptionError)
                }
                _ => Err(Error::UnsupportedOperation),
            }
        }
    }
} 