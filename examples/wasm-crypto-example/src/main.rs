use elastic_crypto::{CryptoContext, KeyType, Algorithm};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let context = CryptoContext::new();
    
    // Generate a symmetric key
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill(&mut key[..]);
    
    // Load the key
    let handle = context.load_key(&key, KeyType::Symmetric, Algorithm::Aes256Gcm)?;
    
    // Encrypt a message
    let message = b"Hello, ELASTIC Crypto!";
    let ciphertext = context.symmetric_encrypt(handle, message)?;
    
    // Decrypt the message
    let plaintext = context.symmetric_decrypt(handle, &ciphertext)?;
    
    println!("Original message: {}", String::from_utf8_lossy(message));
    println!("Decrypted message: {}", String::from_utf8_lossy(&plaintext));
    
    // Clean up
    context.unload_key(handle)?;
    
    Ok(())
} 