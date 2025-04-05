use elastic_crypto::aes::{AesKey, AesMode};
use std::error::Error;
use hex;
use rand::Rng;

fn is_sev_snp() -> bool {
    if cfg!(target_arch = "wasm32") {
        // In WASM, use WASI env vars
        std::env::vars()
            .any(|(key, _)| key == "SEV_SNP")
    } else {
        // On native, check for /dev/sev-guest
        std::path::Path::new("/dev/sev-guest").exists()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Check if we're running in SEV-SNP environment
    let is_sev = is_sev_snp();
    println!("Running in SEV-SNP environment: {}", is_sev);

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
        println!("Success: encryption/decryption worked!");
    } else {
        println!("Error: decrypted data doesn't match original!");
    }
    
    Ok(())
} 