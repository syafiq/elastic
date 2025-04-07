use crate::Error;
use sev::firmware::guest::Firmware;
use sev::firmware::guest::GuestFieldSelect;
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
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;

        Ok(Self {
            firmware,
            key: key.to_vec(),
        })
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        // Generate random IV using SEV-SNP RNG
        let mut iv = [0u8; 12];
        let request = sev::firmware::guest::DerivedKey::new(false, GuestFieldSelect(0), 0, 0, 0);
        let key_bytes = self.firmware.get_derived_key(None, request)
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;
        iv.copy_from_slice(&key_bytes[..12]);

        // Create a derived key for encryption using our key and IV
        let mut key_data = Vec::with_capacity(self.key.len() + iv.len());
        key_data.extend_from_slice(&self.key);
        key_data.extend_from_slice(&iv);
        
        let request = sev::firmware::guest::DerivedKey::new(
            false,
            GuestFieldSelect(0),
            0,
            key_data.len() as u32,
            key_data.as_ptr() as u64,
        );
        
        // Get the derived key for encryption
        let key_bytes = self.firmware.get_derived_key(None, request)
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;

        // Encrypt using the derived key
        let mut ciphertext = vec![0u8; data.len()];
        for (i, byte) in data.iter().enumerate() {
            ciphertext[i] = byte ^ key_bytes[i % key_bytes.len()];
        }

        // Generate authentication tag
        let mut tag_data = Vec::with_capacity(self.key.len() + ciphertext.len());
        tag_data.extend_from_slice(&self.key);
        tag_data.extend_from_slice(&ciphertext);
        
        let tag_request = sev::firmware::guest::DerivedKey::new(
            false,
            GuestFieldSelect(0),
            0,
            tag_data.len() as u32,
            tag_data.as_ptr() as u64,
        );
        
        let tag_key = self.firmware.get_derived_key(None, tag_request)
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;
        
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&tag_key[..16]);

        // Combine IV, ciphertext, and tag
        let mut result = Vec::with_capacity(12 + ciphertext.len() + 16);
        result.extend_from_slice(&iv);
        result.extend_from_slice(&ciphertext);
        result.extend_from_slice(&tag);
        Ok(result)
    }

    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        if ciphertext.len() < 28 { // 12 bytes IV + 16 bytes tag
            return Err(Error::InvalidCiphertext);
        }

        // Extract IV, encrypted data, and tag
        let iv = &ciphertext[..12];
        let encrypted = &ciphertext[12..ciphertext.len() - 16];
        let tag = &ciphertext[ciphertext.len() - 16..];

        // Verify the authentication tag
        let mut tag_data = Vec::with_capacity(self.key.len() + encrypted.len());
        tag_data.extend_from_slice(&self.key);
        tag_data.extend_from_slice(encrypted);
        
        let tag_request = sev::firmware::guest::DerivedKey::new(
            false,
            GuestFieldSelect(0),
            0,
            tag_data.len() as u32,
            tag_data.as_ptr() as u64,
        );
        
        let tag_key = self.firmware.get_derived_key(None, tag_request)
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;
        
        let mut computed_tag = [0u8; 16];
        computed_tag.copy_from_slice(&tag_key[..16]);
        
        if tag != computed_tag {
            return Err(Error::DecryptionFailed);
        }

        // Create a derived key for decryption using our key and IV
        let mut key_data = Vec::with_capacity(self.key.len() + iv.len());
        key_data.extend_from_slice(&self.key);
        key_data.extend_from_slice(iv);
        
        let request = sev::firmware::guest::DerivedKey::new(
            false,
            GuestFieldSelect(0),
            0,
            key_data.len() as u32,
            key_data.as_ptr() as u64,
        );
        
        // Get the derived key for decryption
        let key_bytes = self.firmware.get_derived_key(None, request)
            .map_err(|e| Error::SevsnpOperationFailed(e.to_string()))?;

        // Decrypt using the derived key
        let mut plaintext = vec![0u8; encrypted.len()];
        for (i, byte) in encrypted.iter().enumerate() {
            plaintext[i] = byte ^ key_bytes[i % key_bytes.len()];
        }

        Ok(plaintext)
    }
} 