use std::collections::HashMap;
use std::sync::Mutex;
use ring::{
    aead,
    rand::{self, SecureRandom},
    signature::{self, KeyPair},
};

use super::{KeyConfig, KeyType};

pub struct CryptoManager {
    keys: Mutex<HashMap<u32, Key>>,
    next_handle: Mutex<u32>,
}

struct Key {
    config: KeyConfig,
    data: Vec<u8>,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            keys: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
        }
    }

    pub fn generate_key(&self, config: &KeyConfig) -> Result<u32, String> {
        let mut keys = self.keys.lock().map_err(|e| e.to_string())?;
        let mut next_handle = self.next_handle.lock().map_err(|e| e.to_string())?;
        
        let handle = *next_handle;
        *next_handle += 1;

        let key_data = match config.key_type {
            KeyType::Symmetric => {
                let rng = rand::SystemRandom::new();
                let mut key = vec![0; config.key_size / 8];
                rng.fill(&mut key).map_err(|e| e.to_string())?;
                key
            }
            KeyType::Asymmetric => {
                let rng = rand::SystemRandom::new();
                let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
                    .map_err(|e| e.to_string())?
                    .as_ref()
                    .to_vec();
                pkcs8_bytes
            }
            KeyType::Hmac => {
                let rng = rand::SystemRandom::new();
                let mut key = vec![0; config.key_size / 8];
                rng.fill(&mut key).map_err(|e| e.to_string())?;
                key
            }
        };

        let key = Key {
            config: config.clone(),
            data: key_data,
        };

        keys.insert(handle, key);
        Ok(handle)
    }

    pub fn import_key(&self, key_data: &[u8], config: &KeyConfig) -> Result<u32, String> {
        let mut keys = self.keys.lock().map_err(|e| e.to_string())?;
        let mut next_handle = self.next_handle.lock().map_err(|e| e.to_string())?;
        
        let handle = *next_handle;
        *next_handle += 1;

        let key = Key {
            config: config.clone(),
            data: key_data.to_vec(),
        };

        keys.insert(handle, key);
        Ok(handle)
    }

    pub fn export_key(&self, handle: u32) -> Result<Vec<u8>, String> {
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys.get(&handle).ok_or_else(|| "Key not found".to_string())?;
        Ok(key.data.clone())
    }

    pub fn delete_key(&self, handle: u32) -> Result<(), String> {
        let mut keys = self.keys.lock().map_err(|e| e.to_string())?;
        keys.remove(&handle).ok_or_else(|| "Key not found".to_string())?;
        Ok(())
    }

    pub fn encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, String> {
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys.get(&handle).ok_or_else(|| "Key not found".to_string())?;

        match key.config.key_type {
            KeyType::Symmetric => {
                let algorithm = &aead::CHACHA20_POLY1305;
                let key = aead::UnboundKey::new(algorithm, &key.data)
                    .map_err(|e| e.to_string())?;
                let nonce = aead::Nonce::assume_unique_for_key([0; 12]);
                let aad = aead::Aad::empty();
                let mut in_out = data.to_vec();
                let key = aead::LessSafeKey::new(key);
                key.seal_in_place_append_tag(nonce, aad, &mut in_out)
                    .map_err(|e| e.to_string())?;
                Ok(in_out)
            }
            _ => Err("Unsupported key type for encryption".to_string()),
        }
    }

    pub fn decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, String> {
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys.get(&handle).ok_or_else(|| "Key not found".to_string())?;

        match key.config.key_type {
            KeyType::Symmetric => {
                let algorithm = &aead::CHACHA20_POLY1305;
                let key = aead::UnboundKey::new(algorithm, &key.data)
                    .map_err(|e| e.to_string())?;
                let nonce = aead::Nonce::assume_unique_for_key([0; 12]);
                let aad = aead::Aad::empty();
                let mut in_out = data.to_vec();
                let key = aead::LessSafeKey::new(key);
                let len = key.open_in_place(nonce, aad, &mut in_out)
                    .map_err(|e| e.to_string())?
                    .len();
                in_out.truncate(len);
                Ok(in_out)
            }
            _ => Err("Unsupported key type for decryption".to_string()),
        }
    }

    pub fn sign(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, String> {
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys.get(&handle).ok_or_else(|| "Key not found".to_string())?;

        match key.config.key_type {
            KeyType::Asymmetric => {
                let key_pair = signature::Ed25519KeyPair::from_pkcs8(&key.data)
                    .map_err(|e| e.to_string())?;
                Ok(key_pair.sign(data).as_ref().to_vec())
            }
            _ => Err("Unsupported key type for signing".to_string()),
        }
    }

    pub fn verify(&self, handle: u32, data: &[u8], sig: &[u8]) -> Result<bool, String> {
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys.get(&handle).ok_or_else(|| "Key not found".to_string())?;

        match key.config.key_type {
            KeyType::Asymmetric => {
                let key_pair = signature::Ed25519KeyPair::from_pkcs8(&key.data)
                    .map_err(|e| e.to_string())?;
                let public_key = key_pair.public_key();
                let public_key = signature::UnparsedPublicKey::new(
                    &signature::ED25519,
                    public_key.as_ref(),
                );
                Ok(public_key.verify(data, sig).is_ok())
            }
            _ => Err("Unsupported key type for verification".to_string()),
        }
    }
} 