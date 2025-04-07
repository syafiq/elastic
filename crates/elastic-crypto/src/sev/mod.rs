#[cfg(target_os = "linux")]
use sev::firmware::guest::Firmware;
use crate::Error;
use std::io;

// SEV-SNP specific implementation
#[derive(Debug)]
pub struct SevsnpRng {
    #[cfg(target_os = "linux")]
    firmware: Firmware,
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
            Err(Error::SevsnpNotAvailable)
        }
    }

    pub fn get_random_bytes(&mut self, _size: usize) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            let mut buf = vec![0u8; _size];
            self.firmware.get_random(&mut buf).map_err(|_| Error::SevsnpNotAvailable)?;
            Ok(buf)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::SevsnpNotAvailable)
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

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
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
                .map_err(|e| rand::Error::new(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
            _dest.copy_from_slice(&key[.._dest.len()]);
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(rand::Error::new(io::Error::new(io::ErrorKind::Other, "SEV-SNP not available")))
        }
    }
}

// SEV-SNP specific AES implementation
#[derive(Debug)]
pub struct SevsnpAes {
    #[cfg(target_os = "linux")]
    firmware: Firmware,
    _key: Vec<u8>,
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
            Ok(Self {
                _key: key.to_vec(),
            })
        }
    }

    pub fn encrypt(&mut self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            let mut output = vec![0u8; _data.len()];
            self.firmware.encrypt(&self._key, _data, &mut output)
                .map_err(|_| Error::EncryptionError)?;
            Ok(output)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::SevsnpNotAvailable)
        }
    }

    pub fn decrypt(&mut self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        #[cfg(target_os = "linux")]
        {
            let mut output = vec![0u8; _data.len()];
            self.firmware.decrypt(&self._key, _data, &mut output)
                .map_err(|_| Error::DecryptionError)?;
            Ok(output)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::SevsnpNotAvailable)
        }
    }
} 