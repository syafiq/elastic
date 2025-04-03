use std::sync::Arc;
use super::common::{CryptoError, KeyConfig, KeyType};

#[derive(Clone)]
pub struct CryptoContext {
    manager: Arc<crypto::CryptoManager>,
}

impl CryptoContext {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(crypto::CryptoManager::new()),
        }
    }

    pub fn generate_key(&self, config: &KeyConfig) -> Result<u32, CryptoError> {
        self.manager.generate_key(config)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn import_key(&self, key_data: &[u8], config: &KeyConfig) -> Result<u32, CryptoError> {
        self.manager.import_key(key_data, config)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn export_key(&self, handle: u32) -> Result<Vec<u8>, CryptoError> {
        self.manager.export_key(handle)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn delete_key(&self, handle: u32) -> Result<(), CryptoError> {
        self.manager.delete_key(handle)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn encrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.manager.encrypt(handle, data)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn decrypt(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.manager.decrypt(handle, data)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn sign(&self, handle: u32, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.manager.sign(handle, data)
            .map_err(|e| CryptoError::OperationFailed(e))
    }

    pub fn verify(&self, handle: u32, data: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        self.manager.verify(handle, data, signature)
            .map_err(|e| CryptoError::OperationFailed(e))
    }
}

pub mod crypto; 