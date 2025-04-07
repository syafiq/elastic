use elastic_crypto::{SevsnpRng, SevsnpAes, Error};

#[test]
fn test_sevsnp_rng() -> Result<(), Error> {
    let mut rng = SevsnpRng::new()?;
    
    // Test generating random bytes
    let bytes = rng.get_random_bytes(32)?;
    assert_eq!(bytes.len(), 32);
    
    // Test generating multiple random values
    let bytes1 = rng.get_random_bytes(32)?;
    let bytes2 = rng.get_random_bytes(32)?;
    assert_ne!(bytes1, bytes2);
    
    Ok(())
}

#[test]
fn test_sevsnp_aes() -> Result<(), Error> {
    // Generate a random key
    let mut rng = SevsnpRng::new()?;
    let key = rng.get_random_bytes(32)?;
    
    // Initialize AES
    let mut aes = SevsnpAes::new(&key)?;
    
    // Test encryption and decryption
    let plaintext = b"Hello, SEV-SNP!";
    let ciphertext = aes.encrypt(plaintext)?;
    let decrypted = aes.decrypt(&ciphertext)?;
    
    assert_eq!(plaintext, &decrypted[..]);
    
    // Test with empty data
    let empty_ciphertext = aes.encrypt(b"")?;
    let empty_decrypted = aes.decrypt(&empty_ciphertext)?;
    assert_eq!(empty_decrypted, b"");
    
    // Test with larger data
    let large_data = vec![0u8; 1024];
    let large_ciphertext = aes.encrypt(&large_data)?;
    let large_decrypted = aes.decrypt(&large_ciphertext)?;
    assert_eq!(large_data, large_decrypted);
    
    Ok(())
}

#[test]
fn test_sevsnp_aes_invalid_key() {
    // Test with invalid key length
    let result = SevsnpAes::new(&[0u8; 16]);
    assert!(matches!(result, Err(Error::InvalidKeyLength)));
}

#[test]
fn test_sevsnp_aes_invalid_ciphertext() {
    let mut rng = SevsnpRng::new().unwrap();
    let key = rng.get_random_bytes(32).unwrap();
    let mut aes = SevsnpAes::new(&key).unwrap();
    
    // Test with too short ciphertext
    let result = aes.decrypt(&[0u8; 10]);
    assert!(matches!(result, Err(Error::InvalidCiphertext)));
    
    // Test with invalid ciphertext format (correct length but invalid data)
    let result = aes.decrypt(&[0u8; 50]);
    assert!(matches!(result, Err(Error::DecryptionFailed)));
} 