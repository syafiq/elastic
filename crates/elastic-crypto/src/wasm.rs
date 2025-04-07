use crate::{Error, Crypto};
use crate::aes::AesMode;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use std::path::Path;
use std::cell::UnsafeCell;

#[cfg(feature = "sevsnp")]
use crate::sev::{SevsnpRng, SevsnpAes};

// Inner struct to hold the mutable state
#[cfg(feature = "sevsnp")]
struct WasmCryptoInner {
    is_sevsnp: bool,
    rng: Option<SevsnpRng>,
    aes: Option<SevsnpAes>,
}

#[cfg(not(feature = "sevsnp"))]
struct WasmCryptoInner {
    is_sevsnp: bool,
}

// WASM-specific implementation that uses SEV-SNP when available
pub struct WasmCrypto {
    inner: UnsafeCell<WasmCryptoInner>,
}

// SAFETY: WasmCrypto is only used in single-threaded WASM context
unsafe impl Send for WasmCrypto {}
unsafe impl Sync for WasmCrypto {}

impl WasmCrypto {
    pub fn new() -> Self {
        println!("Checking SEV-SNP availability...");
        let is_sevsnp = Path::new("/dev/sev-guest").exists();
        println!("SEV-SNP {} available", if is_sevsnp { "is" } else { "is not" });

        #[cfg(feature = "sevsnp")]
        if is_sevsnp {
            if let Ok(rng) = SevsnpRng::new() {
                return Self {
                    inner: UnsafeCell::new(WasmCryptoInner {
                        is_sevsnp: true,
                        rng: Some(rng),
                        aes: None,
                    }),
                };
            }
        }

        #[cfg(feature = "sevsnp")]
        {
            Self {
                inner: UnsafeCell::new(WasmCryptoInner {
                    is_sevsnp: false,
                    rng: None,
                    aes: None,
                }),
            }
        }

        #[cfg(not(feature = "sevsnp"))]
        Self {
            inner: UnsafeCell::new(WasmCryptoInner {
                is_sevsnp: false,
            }),
        }
    }

    pub fn is_sevsnp(&self) -> bool {
        // SAFETY: We're in a single-threaded WASM context
        unsafe { (*self.inner.get()).is_sevsnp }
    }

    #[cfg(feature = "sevsnp")]
    fn generate_sevsnp_key(&self) -> Result<Vec<u8>, Error> {
        // SAFETY: We're in a single-threaded WASM context
        let inner = unsafe { &mut *self.inner.get() };
        if let Some(rng) = inner.rng.as_mut() {
            rng.get_random_bytes(32)
        } else {
            Err(Error::SevsnpNotAvailable)
        }
    }

    #[cfg(not(feature = "sevsnp"))]
    fn generate_sevsnp_key(&self) -> Result<Vec<u8>, Error> {
        Err(Error::SevsnpNotAvailable)
    }

    #[cfg(feature = "sevsnp")]
    fn encrypt_sevsnp(&self, key: &[u8], data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        // SAFETY: We're in a single-threaded WASM context
        let inner = unsafe { &mut *self.inner.get() };
        if inner.aes.is_none() {
            inner.aes = Some(SevsnpAes::new(key)?);
        }
        if let Some(aes) = inner.aes.as_mut() {
            aes.encrypt(data)
        } else {
            Err(Error::SevsnpNotAvailable)
        }
    }

    #[cfg(not(feature = "sevsnp"))]
    fn encrypt_sevsnp(&self, _key: &[u8], _data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        Err(Error::SevsnpNotAvailable)
    }

    #[cfg(feature = "sevsnp")]
    fn decrypt_sevsnp(&self, key: &[u8], data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        // SAFETY: We're in a single-threaded WASM context
        let inner = unsafe { &mut *self.inner.get() };
        if inner.aes.is_none() {
            inner.aes = Some(SevsnpAes::new(key)?);
        }
        if let Some(aes) = inner.aes.as_mut() {
            aes.decrypt(data)
        } else {
            Err(Error::SevsnpNotAvailable)
        }
    }

    #[cfg(not(feature = "sevsnp"))]
    fn decrypt_sevsnp(&self, _key: &[u8], _data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        Err(Error::SevsnpNotAvailable)
    }
}

impl Crypto for WasmCrypto {
    fn generate_key(&self) -> Result<Vec<u8>, Error> {
        if self.is_sevsnp() {
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

        if self.is_sevsnp() {
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

        if self.is_sevsnp() {
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