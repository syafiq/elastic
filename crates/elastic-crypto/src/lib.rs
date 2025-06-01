#[cfg(feature = "linux")]
mod linux;
#[cfg(any(feature = "wasi", feature = "wasm"))]
pub mod wasm;
#[cfg(feature = "sevsnp")]
mod sev;

mod error;
pub mod aes;

pub use error::Error;
pub use aes::{AesKey, AesMode};

#[cfg(feature = "linux")]
pub use linux::*;
#[cfg(any(feature = "wasi", feature = "wasm"))]
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
    use std::path::Path;

    #[test]
    #[cfg(feature = "wasi")]
    fn test_wasm_crypto() {
        let crypto = WasmCrypto::new();
        let has_sevsnp = Path::new("/dev/sev-guest").exists();
        assert_eq!(crypto.is_sevsnp(), has_sevsnp, "SEV-SNP detection should match device existence");

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
    #[cfg(feature = "sevsnp")]
    fn test_sevsnp_rng() {
        let rng = SevsnpRng::new();
        let has_sevsnp = Path::new("/dev/sev-guest").exists();
        
        if has_sevsnp {
            let mut rng = rng.unwrap();
            let bytes = rng.get_random_bytes(32).unwrap();
            assert_eq!(bytes.len(), 32);
        } else {
            assert!(matches!(rng, Err(Error::SevsnpNotAvailable)));
        }
    }

    #[test]
    #[cfg(feature = "sevsnp")]
    fn test_sevsnp_aes() {
        let has_sevsnp = Path::new("/dev/sev-guest").exists();
        let aes = SevsnpAes::new(&[0u8; 32]);
        
        if has_sevsnp {
            let mut aes = aes.unwrap();
            let data = b"test data";
            let encrypted = aes.encrypt(data).unwrap();
            let decrypted = aes.decrypt(&encrypted).unwrap();
            assert_eq!(data, &decrypted[..]);
        } else {
            assert!(matches!(aes, Err(Error::SevsnpNotAvailable)));
        }
    }

    #[test]
    fn test_error_handling() {
        // Test invalid key length
        let key = vec![0u8; 16]; // Too short
        assert!(matches!(AesKey::new(&key), Err(Error::InvalidKeyLength)));

        // Test unsupported operation
        let key = vec![0u8; 32];
        let aes = AesKey::new(&key).unwrap();
        assert!(matches!(aes.encrypt(b"test", AesMode::CBC), Err(Error::UnsupportedOperation)));
    }
} 