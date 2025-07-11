use elastic_crypto::{ElasticCrypto, KeyConfig, KeyType};
use base64::Engine;
use std::env;

fn main() {
    // Debug: Print all environment variables
    println!("Environment variables:");
    for (key, value) in env::vars() {
        println!("{} = {}", key, value);
    }
    
    // Initialize crypto
    let crypto = ElasticCrypto::new().expect("Failed to initialize crypto");
    
    // Generate a key
    let key_config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    let key_handle = crypto.generate_key(key_config).expect("Failed to generate key");
    
    // Test data
    let test_data = b"Hello, Elastic Crypto!";
    
    // Encrypt
    let encrypted = crypto.encrypt(key_handle, test_data.to_vec()).expect("Encryption failed");
    
    // Decrypt
    let decrypted = crypto.decrypt(key_handle, encrypted.clone()).expect("Decryption failed");
    
    // Verify
    assert_eq!(test_data, decrypted.as_slice(), "Decrypted data doesn't match original");
    
    // Print results in a consistent format
    let platform = if crypto.is_sevsnp() { "SEV-SNP" } else { "Linux" };
    println!("Platform: {}", platform);
    println!("Test data: {}", String::from_utf8_lossy(test_data));
    println!("Encrypted (base64): {}", base64::engine::general_purpose::STANDARD.encode(&encrypted));
    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted));
    
    // Clean up
    crypto.delete_key(key_handle).expect("Failed to delete key");
} 