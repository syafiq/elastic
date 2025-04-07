#[cfg(feature = "sevsnp")]
use elastic_crypto::{SevsnpRng, SevsnpAes, Error};
use std::path::Path;
use rand_core::RngCore;

#[cfg(feature = "sevsnp")]
#[test]
fn test_sevsnp_rng_operations() {
    // Skip test if not in SEV-SNP environment
    if !Path::new("/dev/sev-guest").exists() {
        return;
    }

    let mut rng = SevsnpRng::new().unwrap();
    
    // Test generating random bytes
    let bytes1 = rng.get_random_bytes(32).unwrap();
    let bytes2 = rng.get_random_bytes(32).unwrap();
    
    // Verify we get different random values
    assert_ne!(bytes1, bytes2);
    
    // Test RngCore implementation
    let mut dest1 = [0u8; 32];
    let mut dest2 = [0u8; 32];
    rng.fill_bytes(&mut dest1);
    rng.fill_bytes(&mut dest2);
    assert_ne!(dest1, dest2);
}

#[cfg(feature = "sevsnp")]
#[test]
fn test_sevsnp_aes_operations() {
    // Skip test if not in SEV-SNP environment
    if !Path::new("/dev/sev-guest").exists() {
        return;
    }

    let key = [0u8; 32];
    let mut aes = SevsnpAes::new(&key).unwrap();
    
    // Test encryption and decryption
    let plaintext = b"Hello, SEV-SNP!";
    let ciphertext = aes.encrypt(plaintext).unwrap();
    let decrypted = aes.decrypt(&ciphertext).unwrap();
    
    assert_eq!(plaintext, &decrypted[..]);
    
    // Test error handling
    let result = SevsnpAes::new(&[0u8; 16]);
    assert!(matches!(result, Err(Error::InvalidKeyLength)));
    
    let result = aes.decrypt(&[0u8; 20]);
    assert!(matches!(result, Err(Error::InvalidCiphertext)));
}

#[cfg(feature = "sevsnp")]
#[test]
fn test_sevsnp_error_handling() {
    // Skip test if not in SEV-SNP environment
    if !Path::new("/dev/sev-guest").exists() {
        return;
    }

    // Test invalid key length
    let result = SevsnpAes::new(&[0u8; 16]);
    assert!(matches!(result, Err(Error::InvalidKeyLength)));
    
    // Test invalid ciphertext
    let key = [0u8; 32];
    let mut aes = SevsnpAes::new(&key).unwrap();
    let result = aes.decrypt(&[0u8; 20]);
    assert!(matches!(result, Err(Error::InvalidCiphertext)));
} 