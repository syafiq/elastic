use elastic_crypto::{CryptoContext, KeyConfig, KeyType};

#[tokio::test]
async fn test_key_operations() {
    let ctx = CryptoContext::new();
    
    // Test key generation
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).await.unwrap();
    
    // Test key export/import
    let key_data = ctx.export_key(handle).await.unwrap();
    let new_handle = ctx.import_key(&key_data, &config).await.unwrap();
    
    // Test encryption/decryption
    let plaintext = b"Hello, World!";
    let ciphertext = ctx.encrypt(handle, plaintext).await.unwrap();
    let decrypted = ctx.decrypt(handle, &ciphertext).await.unwrap();
    assert_eq!(plaintext, decrypted.as_slice());
    
    // Test key deletion
    ctx.delete_key(handle).await.unwrap();
    ctx.delete_key(new_handle).await.unwrap();
}

#[tokio::test]
async fn test_asymmetric_operations() {
    let ctx = CryptoContext::new();
    
    // Test asymmetric key generation
    let config = KeyConfig {
        key_type: KeyType::Asymmetric,
        key_size: 2048,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).await.unwrap();
    
    // Test signing/verification
    let data = b"Hello, World!";
    let signature = ctx.sign(handle, data).await.unwrap();
    let verified = ctx.verify(handle, data, &signature).await.unwrap();
    assert!(verified);
    
    // Test key deletion
    ctx.delete_key(handle).await.unwrap();
}

#[tokio::test]
async fn test_error_handling() {
    let ctx = CryptoContext::new();
    
    // Test invalid handle
    assert!(ctx.export_key(999).await.is_err());
    assert!(ctx.delete_key(999).await.is_err());
    
    // Test invalid operations
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).await.unwrap();
    assert!(ctx.sign(handle, b"data").await.is_err());
    assert!(ctx.verify(handle, b"data", b"signature").await.is_err());
    
    ctx.delete_key(handle).await.unwrap();
}

#[tokio::test]
async fn test_hmac_operations() {
    let ctx = CryptoContext::new();
    
    // Test HMAC key generation
    let config = KeyConfig {
        key_type: KeyType::Hmac,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).await.unwrap();
    
    // Test MAC calculation and verification
    let data = b"Hello, World!";
    let mac = ctx.calculate_mac(handle, data).await.unwrap();
    let verified = ctx.verify_mac(handle, data, &mac).await.unwrap();
    assert!(verified);
    
    // Test with modified data
    let modified_data = b"Hello, World?";
    let verified = ctx.verify_mac(handle, modified_data, &mac).await.unwrap();
    assert!(!verified);
    
    ctx.delete_key(handle).await.unwrap();
}

#[tokio::test]
async fn test_hash_operations() {
    let ctx = CryptoContext::new();
    
    // Test SHA-256
    let data = b"Hello, World!";
    let hash = ctx.hash(data).await.unwrap();
    assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
    
    // Test SHA-512
    let hash = ctx.hash_sha512(data).await.unwrap();
    assert_eq!(hash.len(), 64); // SHA-512 produces 64 bytes
} 