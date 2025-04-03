use rand::rngs::OsRng;
use ring::aead::{self, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey};
use ring::digest::{self, Digest};
use ring::hmac;
use crate::{CryptoConfig, HashAlgorithm};

pub struct CryptoManager {
    rng: OsRng,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            rng: OsRng,
        }
    }

    pub fn generate_key(&self, config: &CryptoConfig) -> Result<Vec<u8>, String> {
        if config.key_size == 0 {
            return Err("Invalid key size".to_string());
        }
        let mut key = vec![0u8; config.key_size];
        self.rng
            .try_fill_bytes(&mut key)
            .map_err(|e| format!("Failed to generate key: {}", e))?;
        Ok(key)
    }

    pub fn hash(&self, data: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>, String> {
        let digest = match algorithm {
            HashAlgorithm::Sha256 => digest::digest(&digest::SHA256, data),
            HashAlgorithm::Sha384 => digest::digest(&digest::SHA384, data),
            HashAlgorithm::Sha512 => digest::digest(&digest::SHA512, data),
        };
        Ok(digest.as_ref().to_vec())
    }

    pub fn encrypt(&self, key: &[u8], data: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|e| format!("Failed to create key: {}", e))?;
        let nonce = Nonce::try_assume_unique_for_key(nonce)
            .map_err(|e| format!("Invalid nonce: {}", e))?;
        let mut sealing_key = SealingKey::new(unbound_key, nonce);
        let mut in_out = data.to_vec();
        sealing_key
            .seal_in_place_append_tag(aead::Aad::empty(), &mut in_out)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        Ok(in_out)
    }

    pub fn decrypt(&self, key: &[u8], data: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|e| format!("Failed to create key: {}", e))?;
        let nonce = Nonce::try_assume_unique_for_key(nonce)
            .map_err(|e| format!("Invalid nonce: {}", e))?;
        let mut opening_key = OpeningKey::new(unbound_key, nonce);
        let mut in_out = data.to_vec();
        opening_key
            .open_in_place(aead::Aad::empty(), &mut in_out)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        Ok(in_out)
    }

    pub fn verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool, String> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        Ok(hmac::verify(&key, data, signature).is_ok())
    }

    pub fn sign(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let signature = hmac::sign(&key, data);
        Ok(signature.as_ref().to_vec())
    }
} 