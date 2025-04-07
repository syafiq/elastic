use crate::Error;
use sev::firmware::guest::Firmware;
use std::path::Path;
use std::io;

// SEV-SNP specific implementation
pub struct SevsnpRng {
    firmware: Firmware,
    counter: u32,
}

impl SevsnpRng {
    pub fn new() -> Result<Self, Error> {
        // Check if we're in a SEV-SNP environment
        if !Path::new("/dev/sev-guest").exists() {
            return Err(Error::SevsnpNotAvailable);
        }

        // Initialize the SEV firmware interface
        let firmware = Firmware::open()
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;

        Ok(Self { firmware, counter: 0 })
    }

    pub fn get_random_bytes(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0u8; len];
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        self.counter = self.counter.wrapping_add(1);
        let request = sev::firmware::guest::DerivedKey::new(
            false,
            GuestFieldSelect(0),
            self.counter,
            timestamp,
            timestamp as u64,
        );
        let key = self.firmware.get_derived_key(None, request)
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;
        buffer.copy_from_slice(&key[..len]);
        Ok(buffer)
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
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        self.counter = self.counter.wrapping_add(1);
        let request = sev::firmware::guest::DerivedKey::new(
            false,
            GuestFieldSelect(0),
            self.counter,
            timestamp,
            timestamp as u64,
        );
        let key = self.firmware.get_derived_key(None, request)
            .map_err(|e| rand::Error::new(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
        dest.copy_from_slice(&key[..dest.len()]);
        Ok(())
    }
}

// SEV-SNP specific AES implementation
pub struct SevsnpAes {
    firmware: Firmware,
    key: Vec<u8>,
}

impl SevsnpAes {
    pub fn new(key: &[u8]) -> Result<Self, Error> {
        // Check if we're in a SEV-SNP environment
        if !Path::new("/dev/sev-guest").exists() {
            return Err(Error::SevsnpNotAvailable);
        }

        // Validate key length
        if key.len() != 32 {
            return Err(Error::InvalidKeyLength);
        }

        // Initialize the SEV firmware interface
        let firmware = Firmware::open()
            .map_err(|_| Error::SevsnpNotAvailable)?;

        Ok(Self {
            firmware,
            key: key.to_vec(),
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        // Use SEV-SNP's hardware AES encryption
        let mut ciphertext = vec![0u8; data.len() + 16]; // Add space for IV and tag
        self.firmware
            .aes_encrypt(&self.key, data, &mut ciphertext)
            .map_err(|_| Error::NotImplemented)?;
        Ok(ciphertext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        // Use SEV-SNP's hardware AES decryption
        let mut plaintext = vec![0u8; ciphertext.len() - 16]; // Remove IV and tag
        self.firmware
            .aes_decrypt(&self.key, ciphertext, &mut plaintext)
            .map_err(|_| Error::NotImplemented)?;
        Ok(plaintext)
    }
} 