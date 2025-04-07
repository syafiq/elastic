use crate::Error;
use sev::firmware::guest::Firmware;
use std::path::Path;

// SEV-SNP specific implementation
pub struct SevsnpRng {
    firmware: Firmware,
}

impl SevsnpRng {
    pub fn new() -> Result<Self, Error> {
        // Check if we're in a SEV-SNP environment
        if !Path::new("/dev/sev-guest").exists() {
            return Err(Error::SevsnpNotAvailable);
        }

        // Initialize the SEV firmware interface
        let firmware = Firmware::open()
            .map_err(|_| Error::SevsnpNotAvailable)?;

        Ok(Self { firmware })
    }

    pub fn get_random_bytes(&self, len: usize) -> Result<Vec<u8>, Error> {
        // Use SEV-SNP's hardware RNG
        let mut buffer = vec![0u8; len];
        self.firmware
            .get_random_bytes(&mut buffer)
            .map_err(|_| Error::NotImplemented)?;
        Ok(buffer)
    }
}

impl rand::RngCore for SevsnpRng {
    fn next_u32(&mut self) -> u32 {
        // TODO: Implement proper SEV-SNP RNG
        thread_rng().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        // TODO: Implement proper SEV-SNP RNG
        thread_rng().next_u64()
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        // TODO: Implement proper SEV-SNP RNG
        thread_rng().fill_bytes(_dest);
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        // TODO: Implement proper SEV-SNP RNG
        thread_rng().try_fill_bytes(_dest)
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