#[cfg(target_os = "linux")]
use sev::firmware::guest::Firmware;
use crate::Error;
use rand::RngCore;
use rand::rngs::OsRng;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use std::fmt;

// SEV-SNP specific implementation
#[derive(Debug)]
pub struct SevsnpRng {
    #[cfg(target_os = "linux")]
    firmware: Firmware,
    #[cfg(not(target_os = "linux"))]
    rng: OsRng,
}

impl SevsnpRng {
    pub fn new() -> Result<Self, Error> {
        #[cfg(target_os = "linux")]
        {
            let firmware = Firmware::open().map_err(|_| Error::SevsnpNotAvailable)?;
            Ok(Self { firmware })
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(Self { rng: OsRng })
        }
    }

    pub fn get_random_bytes(&mut self, size: usize) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            let mut buf = vec![0u8; size];
            self.firmware.get_random(&mut buf).map_err(|_| Error::SevsnpNotAvailable)?;
            Ok(buf)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let mut buf = vec![0u8; size];
            self.rng.fill_bytes(&mut buf);
            Ok(buf)
        }
    }
}

impl rand::RngCore for SevsnpRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).unwrap();
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        #[cfg(target_os = "linux")]
        {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            let request = sev::firmware::guest::DerivedKey::new(
                false,
                sev::firmware::guest::GuestFieldSelect(0),
                timestamp,
                timestamp,
                timestamp as u64,
            );
            let key = self.firmware.get_derived_key(None, request)
                .map_err(|e| rand::Error::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            dest.copy_from_slice(&key[..dest.len()]);
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            self.rng.fill_bytes(dest);
            Ok(())
        }
    }
}

// SEV-SNP specific AES implementation
pub struct SevsnpAes {
    #[cfg(target_os = "linux")]
    firmware: Firmware,
    _key: Vec<u8>,
    #[cfg(not(target_os = "linux"))]
    cipher: Aes256Gcm,
}

impl fmt::Debug for SevsnpAes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SevsnpAes")
            .field("key_len", &self._key.len())
            .finish()
    }
}

impl SevsnpAes {
    pub fn new(key: &[u8]) -> Result<Self, Error> {
        #[cfg(target_os = "linux")]
        {
            let firmware = Firmware::open().map_err(|_| Error::SevsnpNotAvailable)?;
            Ok(Self {
                firmware,
                _key: key.to_vec(),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::EncryptionError)?;
            Ok(Self {
                _key: key.to_vec(),
                cipher,
            })
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            let mut output = vec![0u8; data.len()];
            self.firmware.encrypt(&self._key, data, &mut output)
                .map_err(|_| Error::EncryptionError)?;
            Ok(output)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a random nonce
            self.cipher.encrypt(nonce, data)
                .map_err(|_| Error::EncryptionError)
        }
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            let mut output = vec![0u8; data.len()];
            self.firmware.decrypt(&self._key, data, &mut output)
                .map_err(|_| Error::DecryptionError)?;
            Ok(output)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use the same nonce as encryption
            self.cipher.decrypt(nonce, data)
                .map_err(|_| Error::DecryptionError)
        }
    }
} 