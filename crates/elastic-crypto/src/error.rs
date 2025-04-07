use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid key length")]
    InvalidKeyLength,
    
    #[error("Encryption error")]
    EncryptionError,
    
    #[error("Decryption error")]
    DecryptionError,
    
    #[error("Unsupported operation")]
    UnsupportedOperation,
    
    #[error("Key not found")]
    KeyNotFound,
    
    #[error("Operation not permitted")]
    OperationNotPermitted,
    
    #[error("SEV-SNP not available")]
    SevsnpNotAvailable,
    
    #[error("SEV-SNP operation failed: {0}")]
    SevsnpOperationFailed(String),
    
    #[error("SEV-SNP RNG error: {0}")]
    SevsnpRngError(String),
    
    #[error("SEV-SNP AES error: {0}")]
    SevsnpAesError(String),
    
    #[error("Unsupported mode")]
    UnsupportedMode,
    
    #[error("Not implemented")]
    NotImplemented,
    
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed")]
    DecryptionFailed,
} 