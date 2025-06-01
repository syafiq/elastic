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

#[derive(Debug)]
pub enum CryptoBackend {
    #[cfg(feature = "linux")]
    Linux {
        key: Vec<u8>,
        aes: AesKey,
    },
    #[cfg(feature = "sevsnp")]
    Sevsnp(SevsnpAes),
    #[cfg(feature = "wasm")]
    Wasm(WasmCrypto),
    #[cfg(not(any(feature = "linux", feature = "sevsnp", feature = "wasm")))]
    None,
}

pub struct ElasticCrypto {
    backend: CryptoBackend,
}

impl ElasticCrypto {
    pub fn new() -> Result<Self, Error> {
        #[cfg(feature = "linux")]
        {
            println!("[ElasticCrypto] Using Linux backend");
            let key = vec![0u8; 32];
            let aes = AesKey::new(&key)?;
            Ok(Self {
                backend: CryptoBackend::Linux { key, aes },
            })
        }
        #[cfg(all(not(feature = "linux"), feature = "sevsnp"))]
        {
            // Check if SEV-SNP is available
            if std::path::Path::new("/dev/sev-guest").exists() {
                println!("[ElasticCrypto] Using SEV-SNP backend");
                let key = vec![0u8; 32];
                Ok(Self {
                    backend: CryptoBackend::Sevsnp(SevsnpAes::new(&key)?),
                })
            } else {
                println!("[ElasticCrypto] SEV-SNP not available, falling back to Linux backend");
                let key = vec![0u8; 32];
                let aes = AesKey::new(&key)?;
                Ok(Self {
                    backend: CryptoBackend::Linux { key, aes },
                })
            }
        }
        #[cfg(all(not(feature = "linux"), not(feature = "sevsnp"), feature = "wasm"))]
        {
            println!("[ElasticCrypto] Using WASM backend");
            Ok(Self {
                backend: CryptoBackend::Wasm(WasmCrypto::new()),
            })
        }
        #[cfg(not(any(feature = "linux", feature = "sevsnp", feature = "wasm")))]
        {
            println!("[ElasticCrypto] No supported backend feature enabled");
            Err(Error::UnsupportedOperation)
        }
    }
}

impl Crypto for ElasticCrypto {
    fn generate_key(&self) -> Result<Vec<u8>, Error> {
        match &self.backend {
            #[cfg(feature = "linux")]
            CryptoBackend::Linux { key, .. } => Ok(key.clone()),
            #[cfg(feature = "sevsnp")]
            CryptoBackend::Sevsnp(backend) => Ok(backend.key().to_vec()),
            #[cfg(feature = "wasm")]
            CryptoBackend::Wasm(backend) => backend.generate_key(),
            #[cfg(not(any(feature = "linux", feature = "sevsnp", feature = "wasm")))]
            CryptoBackend::None => Err(Error::UnsupportedOperation),
        }
    }

    fn encrypt(&self, _key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
        match &self.backend {
            #[cfg(feature = "linux")]
            CryptoBackend::Linux { aes, .. } => aes.encrypt(data, mode),
            #[cfg(feature = "sevsnp")]
            CryptoBackend::Sevsnp(backend) => {
                let mut backend = backend.clone();
                backend.encrypt(data)
            }
            #[cfg(feature = "wasm")]
            CryptoBackend::Wasm(backend) => backend.encrypt(_key, data, mode),
            #[cfg(not(any(feature = "linux", feature = "sevsnp", feature = "wasm")))]
            CryptoBackend::None => Err(Error::UnsupportedOperation),
        }
    }

    fn decrypt(&self, _key: &[u8], data: &[u8], mode: AesMode) -> Result<Vec<u8>, Error> {
        match &self.backend {
            #[cfg(feature = "linux")]
            CryptoBackend::Linux { aes, .. } => aes.decrypt(data, mode),
            #[cfg(feature = "sevsnp")]
            CryptoBackend::Sevsnp(backend) => {
                let mut backend = backend.clone();
                backend.decrypt(data)
            }
            #[cfg(feature = "wasm")]
            CryptoBackend::Wasm(backend) => backend.decrypt(_key, data, mode),
            #[cfg(not(any(feature = "linux", feature = "sevsnp", feature = "wasm")))]
            CryptoBackend::None => Err(Error::UnsupportedOperation),
        }
    }
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