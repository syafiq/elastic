use anyhow::Result;
use elastic_crypto::{ElasticCrypto, KeyConfig, KeyType};
use std::env;
use base64::Engine;

fn main() -> Result<()> {
    println!("Elastic Crypto Demo");
    println!("==================");
    if std::env::var("ELASTIC_SEV_SNP").unwrap_or_default() == "1" {
        println!("Running in SEV-SNP mode");
    } else {
        println!("Running in Linux mode");
    }
    println!();

    // Initialize crypto
    let crypto = ElasticCrypto::new()?;

    // Generate a symmetric key
    let key_config = KeyConfig {
        key_type: KeyType::Symmetric,
        key_size: 256,
        secure_storage: false,
    };
    let key_handle = crypto.generate_key(key_config)?;
    println!("\nGenerated AES-256 key with handle: {}", key_handle);

    // Encrypt some data
    let plaintext = b"Hello, Elastic Crypto!";
    let ciphertext = crypto.encrypt(key_handle, plaintext.to_vec())?;
    println!("\nPlaintext: {}", String::from_utf8_lossy(plaintext));
    println!("Ciphertext (base64): {}", base64::engine::general_purpose::STANDARD.encode(&ciphertext));

    // Decrypt the data
    let decrypted = crypto.decrypt(key_handle, ciphertext)?;
    println!("\nDecrypted: {}", String::from_utf8_lossy(&decrypted));

    // Calculate hash
    let hash = crypto.hash(plaintext.to_vec())?;
    println!("\nSHA-256 hash (hex): {}", hex::encode(&hash));

    // Clean up
    crypto.delete_key(key_handle)?;
    println!("\nDemo completed successfully!");

    Ok(())
} 