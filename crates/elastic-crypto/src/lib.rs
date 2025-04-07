#[cfg(feature = "linux")]
mod linux;
#[cfg(feature = "wasm")]
pub mod wasm;
#[cfg(feature = "sevsnp")]
mod sev;

mod error;
mod aes;

pub use error::Error;
pub use aes::{AesKey, AesMode};

#[cfg(feature = "linux")]
pub use linux::*;
#[cfg(feature = "wasm")]
pub use wasm::*;
#[cfg(feature = "sevsnp")]
pub use sev::{SevsnpRng, SevsnpAes};

pub trait Crypto {
    fn generate_key(&self) -> Result<Vec<u8>, Error>;
    fn encrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error>;
    fn decrypt(&self, key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "wasm")]
    fn test_wasm_crypto_non_sevsnp() {
        // Clear SEV_SNP env var to simulate non-SEV-SNP environment
        std::env::remove_var("SEV_SNP");
        assert!(std::env::var("SEV_SNP").is_err(), "SEV_SNP environment variable should be unset");
        
        let crypto = WasmCrypto::new().unwrap();
        assert!(!crypto.is_sevsnp);

        // Test key generation
        let key = crypto.generate_key().unwrap();
        assert_eq!(key.len(), 32);

        // Test encryption/decryption
        let data = b"Hello, Crypto!";
        let encrypted = crypto.encrypt(&key, data, AesMode::GCM).unwrap();
        let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM).unwrap();
        assert_eq!(data, &decrypted[..]);
    }

    #[test]
    #[cfg(feature = "wasm")]
    fn test_wasm_crypto_sevsnp() {
        // First test without SEV-SNP to ensure clean state
        std::env::remove_var("SEV_SNP");
        let crypto = WasmCrypto::new().unwrap();
        assert!(!crypto.is_sevsnp);

        // Now test with SEV-SNP
        std::env::set_var("SEV_SNP", "1");
        let crypto = WasmCrypto::new().unwrap();
        assert!(crypto.is_sevsnp);

        // In a real SEV-SNP environment, these operations should succeed
        // For now, they return NotImplemented since we haven't implemented the SEV-SNP specific crypto yet
        let key = crypto.generate_key();
        assert!(matches!(key, Err(Error::NotImplemented)), "SEV-SNP key generation should be implemented");

        let test_key = [0u8; 32];
        let encrypt_result = crypto.encrypt(&test_key, b"test", AesMode::GCM);
        assert!(matches!(encrypt_result, Err(Error::NotImplemented)), "SEV-SNP encryption should be implemented");

        let decrypt_result = crypto.decrypt(&test_key, b"test", AesMode::GCM);
        assert!(matches!(decrypt_result, Err(Error::NotImplemented)), "SEV-SNP decryption should be implemented");
    }

    #[test]
    #[cfg(feature = "sevsnp")]
    fn test_sevsnp_rng() {
        let rng = SevsnpRng::new();
        // This should fail since SEV-SNP implementation is not complete
        assert!(matches!(rng, Err(Error::SevsnpNotAvailable)));
    }

    #[test]
    #[cfg(feature = "sevsnp")]
    fn test_sevsnp_aes() {
        let aes = SevsnpAes::new(&[0u8; 32]);
        // This should fail since SEV-SNP implementation is not complete
        assert!(matches!(aes, Err(Error::SevsnpNotAvailable)));
    }

    #[test]
    fn test_error_handling() {
        // Test invalid key length
        let key = vec![0u8; 16]; // Too short
        assert!(matches!(AesKey::new(&key), Err(Error::InvalidKeyLength)));

        // Test unsupported operation
        let key = AesKey::new(&[0u8; 32]).unwrap();
        assert!(matches!(key.encrypt(b"test", AesMode::CBC), Err(Error::UnsupportedOperation)));
        assert!(matches!(key.decrypt(b"test", AesMode::CBC), Err(Error::UnsupportedOperation)));
    }
} 