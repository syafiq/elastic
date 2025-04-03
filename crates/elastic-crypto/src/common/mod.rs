use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key configuration")]
    InvalidConfig,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyType {
    Symmetric,
    Asymmetric,
    Hmac,
}

#[derive(Debug, Clone)]
pub struct KeyConfig {
    pub key_type: KeyType,
    pub key_size: usize,
    pub secure_storage: bool,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            key_type: KeyType::Symmetric,
            key_size: 256,
            secure_storage: false,
        }
    }
}

pub trait CryptoOperations {
    fn generate_key(&self, config: &KeyConfig) -> Result<u32, CryptoError>;
    fn import_key(&self, key_data: &[u8], config: &KeyConfig) -> Result<u32, CryptoError>;
    fn export_key(&self, handle: u32) -> Result<Vec<u8>, CryptoError>;
    fn delete_key(&self, handle: u32) -> Result<(), CryptoError>;
    fn encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn sign(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn verify(&self, handle: u32, data: &[u8], signature: &[u8]) -> Result<bool, CryptoError>;
} 