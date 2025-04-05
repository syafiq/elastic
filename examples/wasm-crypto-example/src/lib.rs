use elastic_crypto::{CryptoContext, KeyType, Algorithm, CryptoError};
use std::error::Error;
use rand::Rng;
use hex;

// Generate a random symmetric key for AES-256-GCM
fn generate_symmetric_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill(&mut key[..]);
    key
}

// Generate a random message to encrypt
fn generate_random_message() -> Vec<u8> {
    let mut message = vec![0u8; 32];
    rand::thread_rng().fill(&mut message[..]);
    message
}

#[no_mangle]
pub extern "C" fn demo_symmetric_crypto() -> *const u8 {
    match try_demo_symmetric_crypto() {
        Ok(result) => {
            println!("Symmetric crypto demo successful");
            result.as_ptr()
        },
        Err(e) => {
            println!("Error in symmetric crypto demo: {}", e);
            std::ptr::null()
        }
    }
}

fn try_demo_symmetric_crypto() -> Result<Vec<u8>, Box<dyn Error>> {
    println!("Creating crypto context...");
    let context = CryptoContext::new();
    
    // Generate a symmetric key
    println!("Generating symmetric key...");
    let key = generate_symmetric_key();
    println!("Key (hex): {}", hex::encode(&key));
    
    // Load the key
    println!("Loading key into context...");
    let handle = context.load_key(&key, KeyType::Symmetric, Algorithm::Aes256Gcm)?;
    
    // Generate a random message
    println!("Generating random message...");
    let message = generate_random_message();
    println!("Message (hex): {}", hex::encode(&message));
    
    // Encrypt the message
    println!("Encrypting message...");
    let ciphertext = context.symmetric_encrypt(handle, &message)?;
    println!("Ciphertext (hex): {}", hex::encode(&ciphertext));
    
    // Decrypt the message
    println!("Decrypting message...");
    let plaintext = context.symmetric_decrypt(handle, &ciphertext)?;
    println!("Plaintext (hex): {}", hex::encode(&plaintext));
    
    // Verify the decryption
    if plaintext == message {
        println!("Decryption successful - plaintext matches original message");
    } else {
        println!("Decryption failed - plaintext does not match original message");
        return Err(Box::new(CryptoError::DecryptionFailed));
    }
    
    // Clean up
    println!("Unloading key...");
    context.unload_key(handle)?;
    
    Ok(plaintext)
}

#[no_mangle]
pub extern "C" fn _start() {
    println!("Starting WASM crypto example");
    let result = demo_symmetric_crypto();
    if !result.is_null() {
        println!("Crypto demo completed successfully");
    } else {
        println!("Crypto demo failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetric_crypto() {
        let result = try_demo_symmetric_crypto();
        assert!(result.is_ok());
    }
} 