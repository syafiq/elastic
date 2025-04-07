use elastic_crypto::{CryptoContext, KeyConfig, KeyType, Error};
use tokio::time::sleep;
use std::time::Duration;
use tokio_test::block_on;

#[test]
fn test_sev_snp_initialization() {
    // Test that crypto context creation works in SEV-SNP environment
    let ctx = CryptoContext::new();
    assert!(ctx.is_ok(), "Failed to create crypto context");
}

#[test]
fn test_sev_snp_key_generation() {
    let ctx = CryptoContext::new().unwrap();
    
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: true,
    };
    
    let handle = block_on(ctx.generate_key(&config)).unwrap();
    assert!(handle > 0, "Failed to generate key");
    
    block_on(ctx.delete_key(handle)).unwrap();
}

#[test]
fn test_sev_snp_encryption() {
    let ctx = CryptoContext::new().unwrap();
    
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: true,
    };
    
    let handle = block_on(ctx.generate_key(&config)).unwrap();
    
    // Test encryption/decryption
    let plaintext = b"Hello, World!";
    let ciphertext = block_on(ctx.encrypt(handle, plaintext)).unwrap();
    let decrypted = block_on(ctx.decrypt(handle, &ciphertext)).unwrap();
    
    assert_eq!(plaintext, decrypted.as_slice());
    
    block_on(ctx.delete_key(handle)).unwrap();
}

#[test]
fn test_sev_snp_secure_storage() {
    let ctx = CryptoContext::new().unwrap();
    
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: true,
    };
    
    let handle = block_on(ctx.generate_key(&config)).unwrap();
    
    // Test that key cannot be exported when secure_storage is true
    assert!(block_on(ctx.export_key(handle)).is_err());
    
    block_on(ctx.delete_key(handle)).unwrap();
}

#[test]
fn test_sev_snp_error_handling() {
    let ctx = CryptoContext::new().unwrap();
    
    // Test invalid handle
    assert!(block_on(ctx.export_key(999)).is_err());
    assert!(block_on(ctx.delete_key(999)).is_err());
    
    // Test invalid key size
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 128, // Invalid size for SEV-SNP
        secure_storage: false,
    };
    
    assert!(block_on(ctx.generate_key(&config)).is_err());
}

#[test]
fn test_sev_snp_asymmetric_operations() {
    let ctx = CryptoContext::new().unwrap();
    
    let config = KeyConfig {
        key_type: KeyType::Asymmetric,
        key_size: 2048,
        secure_storage: true,
    };
    
    let handle = block_on(ctx.generate_key(&config)).unwrap();
    
    // Test signing/verification
    let data = b"Hello, World!";
    let signature = block_on(ctx.sign(handle, data)).unwrap();
    let verified = block_on(ctx.verify(handle, data, &signature)).unwrap();
    assert!(verified);
    
    // Test with modified data
    let modified_data = b"Hello, World?";
    let verified = block_on(ctx.verify(handle, modified_data, &signature)).unwrap();
    assert!(!verified);
    
    block_on(ctx.delete_key(handle)).unwrap();
}

#[test]
fn test_sev_snp_hmac_operations() {
    let ctx = CryptoContext::new().unwrap();
    
    let config = KeyConfig {
        key_type: KeyType::Hmac,
        key_size: 256,
        secure_storage: true,
    };
    
    let handle = block_on(ctx.generate_key(&config)).unwrap();
    
    // Test MAC calculation and verification
    let data = b"Hello, World!";
    let mac = block_on(ctx.calculate_mac(handle, data)).unwrap();
    let verified = block_on(ctx.verify_mac(handle, data, &mac)).unwrap();
    assert!(verified);
    
    // Test with modified data
    let modified_data = b"Hello, World?";
    let verified = block_on(ctx.verify_mac(handle, modified_data, &mac)).unwrap();
    assert!(!verified);
    
    block_on(ctx.delete_key(handle)).unwrap();
}

#[test]
fn test_sev_snp_hash_operations() {
    let ctx = CryptoContext::new().unwrap();
    
    // Test SHA-256
    let data = b"Hello, World!";
    let hash = block_on(ctx.hash(data)).unwrap();
    assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
    
    // Test SHA-512
    let hash = block_on(ctx.hash_sha512(data)).unwrap();
    assert_eq!(hash.len(), 64); // SHA-512 produces 64 bytes
} 