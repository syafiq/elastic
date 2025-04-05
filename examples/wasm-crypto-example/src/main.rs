use elastic_crypto::wasm::WasmCrypto;
use elastic_crypto::aes::AesMode;
use elastic_crypto::Error;
use std::error::Error as StdError;
use hex;

fn main() -> Result<(), Box<dyn StdError>> {
    // Initialize crypto with SEV-SNP detection
    let crypto = WasmCrypto::new()?;
    let is_sevsnp = crypto.is_sevsnp;
    println!("Running in SEV-SNP environment: {}", is_sevsnp);

    // Generate a secure key
    let key = crypto.generate_key()?;
    println!("Generated key (hex): {}", hex::encode(&key));
    
    // Example data to encrypt
    let data = b"Hello, Crypto!";
    println!("Original data: {}", String::from_utf8_lossy(data));
    
    // Encrypt using GCM mode
    println!("Encrypting data...");
    let encrypted = crypto.encrypt(&key, data, AesMode::GCM)?;
    println!("Encrypted data (hex): {}", hex::encode(&encrypted));
    
    // Decrypt
    println!("Decrypting data...");
    let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM)?;
    println!("Decrypted data: {}", String::from_utf8_lossy(&decrypted));
    
    if data == &decrypted[..] {
        println!("Success: encryption/decryption worked!");
    } else {
        println!("Error: decrypted data doesn't match original!");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_crypto_operations() {
        // Test in non-SEV-SNP environment
        env::remove_var("SEV_SNP");
        let crypto = WasmCrypto::new().unwrap();
        assert!(!crypto.is_sevsnp);

        // Test key generation
        let key = crypto.generate_key().unwrap();
        assert_eq!(key.len(), 32);

        // Test encryption/decryption
        let data = b"Hello, Crypto!";
        let encrypted = crypto.encrypt(&key, data, AesMode::GCM).unwrap();
        let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM).unwrap();
        assert_eq!(data, &decrypted[..]);

        // Test in SEV-SNP environment
        env::set_var("SEV_SNP", "1");
        let crypto = WasmCrypto::new().unwrap();
        assert!(crypto.is_sevsnp);

        // These should fail since SEV-SNP implementation is not complete
        assert!(matches!(crypto.generate_key(), Err(elastic_crypto::Error::SevsnpNotAvailable)));
        assert!(matches!(crypto.encrypt(&key, data, AesMode::GCM), Err(elastic_crypto::Error::SevsnpNotAvailable)));
        assert!(matches!(crypto.decrypt(&key, &encrypted, AesMode::GCM), Err(elastic_crypto::Error::SevsnpNotAvailable)));
    }

    #[test]
    fn test_error_handling() {
        let crypto = WasmCrypto::new().unwrap();

        // Test invalid key length
        let key = vec![0u8; 16]; // Too short
        assert!(matches!(crypto.encrypt(&key, b"test", AesMode::GCM), Err(elastic_crypto::Error::InvalidKeyLength)));
        assert!(matches!(crypto.decrypt(&key, b"test", AesMode::GCM), Err(elastic_crypto::Error::InvalidKeyLength)));

        // Test unsupported mode
        let key = crypto.generate_key().unwrap();
        assert!(matches!(crypto.encrypt(&key, b"test", AesMode::CBC), Err(elastic_crypto::Error::UnsupportedOperation)));
        assert!(matches!(crypto.decrypt(&key, b"test", AesMode::CBC), Err(elastic_crypto::Error::UnsupportedOperation)));
    }
} 