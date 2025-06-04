// use crate::{Error, Crypto};
use crate::Error;
use crate::aes::AesMode;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use std::env;
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
#[derive(Debug)]
pub struct WasmCrypto {
    inner: UnsafeCell<WasmCryptoInner>,
}

// SAFETY: WasmCrypto is only used in single-threaded WASM context
unsafe impl Send for WasmCrypto {}
unsafe impl Sync for WasmCrypto {}

impl WasmCrypto {
    pub fn new() -> Self {
        println!("Checking SEV-SNP availability...");
        let is_sevsnp = env::var("ELASTIC_SEV_SNP").map(|v| v == "1").unwrap_or(false);
        println!("SEV-SNP environment variable: {}", is_sevsnp);

        #[cfg(feature = "sevsnp")]
        {
            if is_sevsnp {
                println!("SEV-SNP feature is enabled");
                match SevsnpRng::new() {
                    Ok(rng) => {
                        println!("SEV-SNP RNG initialized successfully");
                        return Self {
                            inner: UnsafeCell::new(WasmCryptoInner {
                                is_sevsnp: true,
                                rng: Some(rng),
                                aes: None,
                            }),
                        };
                    }
                    Err(e) => {
                        println!("Failed to initialize SEV-SNP RNG: {:?}", e);
                        println!("Falling back to Linux mode");
                    }
                }
            } else {
                println!("SEV-SNP environment variable not set to '1'");
            }
            
            println!("Using Linux mode (no SEV-SNP)");
            Self {
                inner: UnsafeCell::new(WasmCryptoInner {
                    is_sevsnp: false,
                    rng: None,
                    aes: None,
                }),
            }
        }

        #[cfg(not(feature = "sevsnp"))]
        {
            println!("SEV-SNP feature is not enabled in build");
            println!("Using Linux mode (no SEV-SNP)");
            Self {
                inner: UnsafeCell::new(WasmCryptoInner {
                    is_sevsnp: false,
                }),
            }
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
            Err(Error::SevSnpNotAvailable)
        }
    }

    #[cfg(not(feature = "sevsnp"))]
    fn generate_sevsnp_key(&self) -> Result<Vec<u8>, Error> {
        Err(Error::SevSnpNotAvailable)
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
            Err(Error::SevSnpNotAvailable)
        }
    }

    #[cfg(not(feature = "sevsnp"))]
    fn encrypt_sevsnp(&self, _key: &[u8], _data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        Err(Error::SevSnpNotAvailable)
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
            Err(Error::SevSnpNotAvailable)
        }
    }

    #[cfg(not(feature = "sevsnp"))]
    fn decrypt_sevsnp(&self, _key: &[u8], _data: &[u8], _mode: AesMode) -> Result<Vec<u8>, Error> {
        Err(Error::SevSnpNotAvailable)
    }
} 