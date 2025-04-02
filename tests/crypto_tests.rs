use elastic::crypto::{CryptoContext, KeyType, Algorithm, CryptoError};
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
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let public_key = RsaPublicKey::from(&private_key);
    
    // Export keys
    let private_key_bytes = private_key.to_pkcs8_der().unwrap().as_bytes().to_vec();
    let public_key_bytes = public_key.to_public_key_der().unwrap().as_bytes().to_vec();
    
    // Load keys
    let private_handle = crypto.load_key(&private_key_bytes, KeyType::Asymmetric, Algorithm::Rsa2048).unwrap();
    let public_handle = crypto.load_key(&public_key_bytes, KeyType::Asymmetric, Algorithm::Rsa2048).unwrap();
    
    // Test data
    let data = b"Hello, World!";
    
    // Encrypt with public key
    let encrypted = crypto.public_key_encrypt(public_handle, data).unwrap();
    assert_ne!(encrypted, data);
    
    // Decrypt with private key
    let decrypted = crypto.public_key_decrypt(private_handle, &encrypted).unwrap();
    assert_eq!(decrypted, data);
    
    // Sign with private key
    let signature = crypto.sign(private_handle, data).unwrap();
    
    // Verify with public key
    let verified = crypto.verify(public_handle, data, &signature).unwrap();
    assert!(verified);
    
    // Clean up
    crypto.unload_key(private_handle).unwrap();
    crypto.unload_key(public_handle).unwrap();
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
        Err(CryptoError::InvalidKey(_)) => (),
        _ => panic!("Expected InvalidKey error for invalid key length"),
    }
    
    // Test invalid key type and algorithm combination
    let key = vec![1u8; 32];
    match crypto.load_key(&key, KeyType::Asymmetric, Algorithm::Aes256Gcm) {
        Err(CryptoError::InvalidAlgorithm(_)) => (),
        _ => panic!("Expected InvalidAlgorithm error for invalid key type and algorithm combination"),
    }
    
    // Test non-existent key handle
    match crypto.symmetric_encrypt(999, b"test") {
        Err(CryptoError::InvalidKey(_)) => (),
        _ => panic!("Expected InvalidKey error for non-existent handle"),
    }
} 