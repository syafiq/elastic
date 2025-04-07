#[cfg(feature = "sevsnp")]
use elastic_crypto::{SevsnpRng, SevsnpAes, Error};

#[cfg(feature = "sevsnp")]
#[test]
fn test_sevsnp_rng_operations() {
    // Test RNG initialization
    let rng = SevsnpRng::new();
    if Path::new("/dev/sev-guest").exists() {
        // If we're in a SEV-SNP environment, test RNG operations
        let rng = rng.unwrap();
        let random_bytes = rng.get_random_bytes(32).unwrap();
        assert_eq!(random_bytes.len(), 32);
        
        // Test multiple calls
        let random_bytes2 = rng.get_random_bytes(32).unwrap();
        assert_ne!(random_bytes, random_bytes2);
    } else {
        // If not in SEV-SNP environment, should return SevsnpNotAvailable
        assert!(matches!(rng, Err(Error::SevsnpNotAvailable)));
    }
}

#[cfg(feature = "sevsnp")]
#[test]
fn test_sevsnp_aes_operations() {
    // Test AES initialization
    let key = [0u8; 32];
    let aes = SevsnpAes::new(&key);
    
    if Path::new("/dev/sev-guest").exists() {
        // If we're in a SEV-SNP environment, test AES operations
        let aes = aes.unwrap();
        
        // Test encryption/decryption
        let plaintext = b"Hello, SEV-SNP!";
        let ciphertext = aes.encrypt(plaintext).unwrap();
        assert_ne!(plaintext, &ciphertext[..]);
        
        let decrypted = aes.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
        
        // Test with different data
        let plaintext2 = b"Different message";
        let ciphertext2 = aes.encrypt(plaintext2).unwrap();
        assert_ne!(ciphertext, ciphertext2);
        
        let decrypted2 = aes.decrypt(&ciphertext2).unwrap();
        assert_eq!(plaintext2, &decrypted2[..]);
    } else {
        // If not in SEV-SNP environment, should return SevsnpNotAvailable
        assert!(matches!(aes, Err(Error::SevsnpNotAvailable)));
    }
}

#[cfg(feature = "sevsnp")]
#[test]
fn test_sevsnp_error_handling() {
    // Test invalid key length
    let key = [0u8; 16]; // Too short
    let aes = SevsnpAes::new(&key);
    assert!(matches!(aes, Err(Error::InvalidKeyLength)));
    
    // Test with valid key but in non-SEV environment
    let key = [0u8; 32];
    let aes = SevsnpAes::new(&key);
    if !Path::new("/dev/sev-guest").exists() {
        assert!(matches!(aes, Err(Error::SevsnpNotAvailable)));
    }
} 