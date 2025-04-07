use wasm_bindgen_test::*;
use elastic_crypto::{CryptoContext, KeyConfig, KeyType};
use web_sys::console;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_wasm_key_operations() {
    console::log_1(&"Starting WASM key operations test".into());
    
    let ctx = CryptoContext::new();
    
    let config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).await.unwrap();
    console::log_1(&format!("Generated key with handle: {}", handle).into());
    
    // Test encryption/decryption
    let plaintext = b"Hello, World!";
    let ciphertext = ctx.encrypt(handle, plaintext).await.unwrap();
    console::log_1(&"Encryption successful".into());
    
    let decrypted = ctx.decrypt(handle, &ciphertext).await.unwrap();
    console::log_1(&"Decryption successful".into());
    
    assert_eq!(plaintext, decrypted.as_slice());
    
    ctx.delete_key(handle).await.unwrap();
    console::log_1(&"Key deletion successful".into());
}

#[wasm_bindgen_test]
async fn test_wasm_error_handling() {
    console::log_1(&"Starting WASM error handling test".into());
    
    let ctx = CryptoContext::new();
    
    // Test invalid handle
    assert!(ctx.export_key(999).await.is_err());
    assert!(ctx.delete_key(999).await.is_err());
    
    console::log_1(&"Error handling test successful".into());
}

#[wasm_bindgen_test]
async fn test_wasm_hmac_operations() {
    console::log_1(&"Starting WASM HMAC operations test".into());
    
    let ctx = CryptoContext::new();
    
    let config = KeyConfig {
        key_type: KeyType::Hmac,
        key_size: 256,
        secure_storage: false,
    };
    
    let handle = ctx.generate_key(&config).await.unwrap();
    console::log_1(&"Generated HMAC key".into());
    
    // Test MAC calculation and verification
    let data = b"Hello, World!";
    let mac = ctx.calculate_mac(handle, data).await.unwrap();
    console::log_1(&"MAC calculation successful".into());
    
    let verified = ctx.verify_mac(handle, data, &mac).await.unwrap();
    console::log_1(&"MAC verification successful".into());
    assert!(verified);
    
    // Test with modified data
    let modified_data = b"Hello, World?";
    let verified = ctx.verify_mac(handle, modified_data, &mac).await.unwrap();
    assert!(!verified);
    
    ctx.delete_key(handle).await.unwrap();
    console::log_1(&"HMAC operations test completed".into());
}

#[wasm_bindgen_test]
async fn test_wasm_hash_operations() {
    console::log_1(&"Starting WASM hash operations test".into());
    
    let ctx = CryptoContext::new();
    
    // Test SHA-256
    let data = b"Hello, World!";
    let hash = ctx.hash(data).await.unwrap();
    console::log_1(&"SHA-256 hash calculation successful".into());
    assert_eq!(hash.len(), 32);
    
    // Test SHA-512
    let hash = ctx.hash_sha512(data).await.unwrap();
    console::log_1(&"SHA-512 hash calculation successful".into());
    assert_eq!(hash.len(), 64);
    
    console::log_1(&"Hash operations test completed".into());
} 