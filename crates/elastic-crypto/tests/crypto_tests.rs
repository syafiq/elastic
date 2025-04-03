use elastic_crypto::{CryptoContext, KeyType, Algorithm, CryptoError};
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey},
};
use rand::rngs::OsRng;

#[test]
fn test_symmetric_encryption() {
    let crypto = CryptoContext::new();
    
    // Generate a random 32-byte key for AES-256-GCM
    let key = vec![1u8; 32];
    
    // Load the key
    let handle = crypto.load_key(&key, KeyType::Symmetric, Algorithm::Aes256Gcm).unwrap();
    
    // Test data
    let data = b"Hello, World!";
    
    // Encrypt
    let encrypted = crypto.symmetric_encrypt(handle, data).unwrap();
    assert_ne!(encrypted, data);
    
    // Decrypt
    let decrypted = crypto.symmetric_decrypt(handle, &encrypted).unwrap();
    assert_eq!(decrypted, data);
    
    // Clean up
    crypto.unload_key(handle).unwrap();
}

#[test]
fn test_public_key_operations() {
    let crypto = CryptoContext::new();
    
    // Generate RSA key pair
    println!("Generating RSA key pair...");
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let public_key = RsaPublicKey::from(&private_key);
    println!("RSA key pair generated successfully");
    
    // Export keys
    println!("Exporting keys to DER format...");
    let private_key_der = private_key.to_pkcs8_der().unwrap();
    let private_key_bytes = private_key_der.as_bytes();
    let public_key_der = public_key.to_public_key_der().unwrap();
    let public_key_bytes = public_key_der.as_bytes();
    println!("Private key length: {}, Public key length: {}", private_key_bytes.len(), public_key_bytes.len());
    
    // Load keys
    println!("Loading private key...");
    let private_handle = crypto.load_key(private_key_bytes, KeyType::Asymmetric, Algorithm::Rsa2048).unwrap();
    println!("Loading public key...");
    let public_handle = crypto.load_key(public_key_bytes, KeyType::Asymmetric, Algorithm::Rsa2048).unwrap();
    println!("Keys loaded successfully with handles: private={}, public={}", private_handle, public_handle);
    
    // Test data
    let data = b"Hello, World!";
    println!("Testing encryption with public key...");
    
    // Encrypt with public key
    let encrypted = crypto.public_key_encrypt(public_handle, data).unwrap();
    assert_ne!(encrypted, data);
    println!("Encryption successful");
    
    // Decrypt with private key
    println!("Testing decryption with private key...");
    let decrypted = crypto.public_key_decrypt(private_handle, &encrypted).unwrap();
    assert_eq!(decrypted, data);
    println!("Decryption successful");
    
    // Sign with private key
    println!("Testing signing with private key...");
    let signature = crypto.sign(private_handle, data).unwrap();
    assert!(!signature.is_empty());
    println!("Signing successful");
    
    // Verify with public key
    println!("Testing verification with public key...");
    let verified = crypto.verify(public_handle, data, &signature).unwrap();
    assert!(verified);
    println!("Verification successful");
    
    // Clean up
    println!("Cleaning up keys...");
    crypto.unload_key(private_handle).unwrap();
    crypto.unload_key(public_handle).unwrap();
    println!("Test completed successfully");
}

#[test]
fn test_hashing() {
    let crypto = CryptoContext::new();
    
    // Test data
    let data = b"Hello, World!";
    
    // SHA-256
    let hash256 = crypto.hash(Algorithm::Sha256, data).unwrap();
    assert_eq!(hash256.len(), 32);
    
    // SHA-512
    let hash512 = crypto.hash(Algorithm::Sha512, data).unwrap();
    assert_eq!(hash512.len(), 64);
}

#[test]
fn test_mac() {
    let crypto = CryptoContext::new();
    
    // Generate a random 32-byte key for HMAC
    let key = vec![1u8; 32];
    
    // Load the key
    let handle = crypto.load_key(&key, KeyType::Symmetric, Algorithm::Aes256Gcm).unwrap();
    
    // Test data
    let data = b"Hello, World!";
    
    // Calculate MAC
    let mac = crypto.calculate_mac(handle, data).unwrap();
    assert_eq!(mac.len(), 32);
    
    // Clean up
    crypto.unload_key(handle).unwrap();
}

#[test]
fn test_error_handling() {
    let crypto = CryptoContext::new();
    
    // Test invalid key length
    let invalid_key = vec![1u8; 16];
    match crypto.load_key(&invalid_key, KeyType::Symmetric, Algorithm::Aes256Gcm) {
        Err(CryptoError::InvalidKey) => (),
        _ => panic!("Expected InvalidKey error for invalid key length"),
    }
    
    // Test invalid key type and algorithm combination
    let key = vec![1u8; 32];
    match crypto.load_key(&key, KeyType::Asymmetric, Algorithm::Aes256Gcm) {
        Err(CryptoError::UnsupportedAlgorithm) => (),
        _ => panic!("Expected UnsupportedAlgorithm error for invalid key type and algorithm combination"),
    }
    
    // Test non-existent key handle
    match crypto.symmetric_encrypt(999, b"test") {
        Err(CryptoError::InvalidKey) => (),
        _ => panic!("Expected InvalidKey error for non-existent handle"),
    }
} 