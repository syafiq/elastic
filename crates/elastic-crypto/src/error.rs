use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid key length")]
    InvalidKeyLength,
    
    #[error("Encryption failed")]
    EncryptionError,
    
    #[error("Decryption failed")]
    DecryptionError,
    
    #[error("Operation not supported")]
    UnsupportedOperation,
} 