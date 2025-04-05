use wasm_bindgen::prelude::*;
use elastic_crypto::aes::{AesKey, AesMode};
use rand::Rng;
use hex;
use std::error::Error;

#[wasm_bindgen]
pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill(&mut key[..]);
    key
}

#[wasm_bindgen]
pub fn encrypt_aes(key_bytes: &[u8], data: &[u8], mode: &str) -> Result<Vec<u8>, JsValue> {
    let key = AesKey::new(key_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let mode = match mode.to_lowercase().as_str() {
        "cbc" => AesMode::CBC,
        "gcm" => AesMode::GCM,
        _ => return Err(JsValue::from_str("Invalid mode. Supported modes: CBC, GCM"))
    };

    key.encrypt(data, mode)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn decrypt_aes(key_bytes: &[u8], encrypted_data: &[u8], mode: &str) -> Result<Vec<u8>, JsValue> {
    let key = AesKey::new(key_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let mode = match mode.to_lowercase().as_str() {
        "cbc" => AesMode::CBC,
        "gcm" => AesMode::GCM,
        _ => return Err(JsValue::from_str("Invalid mode. Supported modes: CBC, GCM"))
    };

    key.decrypt(encrypted_data, mode)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn demo_crypto() -> String {
    match try_demo_crypto() {
        Ok(result) => {
            let msg = format!("Crypto demo successful. Result: {}", String::from_utf8_lossy(&result));
            println!("{}", msg);
            msg
        },
        Err(e) => {
            let msg = format!("Error in crypto demo: {}", e);
            println!("{}", msg);
            msg
        }
    }
}

fn try_demo_crypto() -> Result<Vec<u8>, Box<dyn Error>> {
    // Generate a random key
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill(&mut key[..]);
    println!("Generated key (hex): {}", hex::encode(&key));
    
    // Create AES key
    let aes_key = AesKey::new(&key)?;
    
    // Example data to encrypt
    let data = b"Hello, Crypto!";
    println!("Original data: {}", String::from_utf8_lossy(data));
    
    // Encrypt using GCM mode
    println!("Encrypting data...");
    let encrypted = aes_key.encrypt(data, AesMode::GCM)?;
    println!("Encrypted data (hex): {}", hex::encode(&encrypted));
    
    // Decrypt
    println!("Decrypting data...");
    let decrypted = aes_key.decrypt(&encrypted, AesMode::GCM)?;
    println!("Decrypted data: {}", String::from_utf8_lossy(&decrypted));
    
    if data == &decrypted[..] {
        println!("Encryption/decryption successful - data matches!");
    } else {
        println!("Encryption/decryption failed - data mismatch!");
        return Err("Data mismatch after decryption".into());
    }
    
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_demo() {
        let result = try_demo_crypto();
        assert!(result.is_ok());
    }
} 