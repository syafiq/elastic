use elastic_crypto::{CryptoContext, KeyConfig, KeyType};

#[test]
fn test_key_operations() {
    let ctx = CryptoContext::new();
    
    // Test key generation
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).unwrap();
    
    // Test key export/import
    let key_data = ctx.export_key(handle).unwrap();
    let new_handle = ctx.import_key(&key_data, &config).unwrap();
    
    // Test encryption/decryption
    let plaintext = b"Hello, World!";
    let ciphertext = ctx.encrypt(handle, plaintext).unwrap();
    let decrypted = ctx.decrypt(handle, &ciphertext).unwrap();
    assert_eq!(plaintext, decrypted.as_slice());
    
    // Test key deletion
    ctx.delete_key(handle).unwrap();
    ctx.delete_key(new_handle).unwrap();
}

#[test]
fn test_asymmetric_operations() {
    let ctx = CryptoContext::new();
    
    // Test asymmetric key generation
    let config = KeyConfig {
        key_type: KeyType::Asymmetric,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).unwrap();
    
    // Test signing/verification
    let data = b"Hello, World!";
    let signature = ctx.sign(handle, data).unwrap();
    let verified = ctx.verify(handle, data, &signature).unwrap();
    assert!(verified);
    
    // Test key deletion
    ctx.delete_key(handle).unwrap();
}

#[test]
fn test_error_handling() {
    let ctx = CryptoContext::new();
    
    // Test invalid handle
    assert!(ctx.export_key(999).is_err());
    assert!(ctx.delete_key(999).is_err());
    
    // Test invalid operations
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).unwrap();
    assert!(ctx.sign(handle, b"data").is_err());
    assert!(ctx.verify(handle, b"data", b"signature").is_err());
    
    ctx.delete_key(handle).unwrap();
} 