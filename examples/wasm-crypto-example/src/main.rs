use elastic_crypto::{WasmCrypto, AesMode, Crypto};
use hex;

fn main() {
    println!("Starting WASM crypto example...");
    
    // Initialize crypto with SEV-SNP detection
    let crypto = WasmCrypto::new();
    println!("Crypto initialized. SEV-SNP environment: {}", crypto.is_sevsnp());
    
    // Generate a secure key
    println!("Generating AES key...");
    let key = crypto.generate_key().expect("Failed to generate key");
    println!("Generated key: {}", hex::encode(&key));
    
    // Example data to encrypt
    let data = b"Hello, SEV-SNP!";
    println!("Original data: {}", String::from_utf8_lossy(data));
    
    // Encrypt the data
    println!("Encrypting data...");
    let encrypted = crypto.encrypt(&key, data, AesMode::GCM)
        .expect("Failed to encrypt data");
    println!("Encrypted data: {}", hex::encode(&encrypted));
    
    // Decrypt the data
    println!("Decrypting data...");
    let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM)
        .expect("Failed to decrypt data");
    println!("Decrypted data: {}", String::from_utf8_lossy(&decrypted));
    
    // Verify the result
    if data == &decrypted[..] {
        println!("✅ Encryption/Decryption successful!");
    } else {
        println!("❌ Encryption/Decryption failed!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_crypto_operations() {
        let crypto = WasmCrypto::new();
        println!("Running test in SEV-SNP environment: {}", crypto.is_sevsnp());
        
        let key = crypto.generate_key().expect("Failed to generate key");
        println!("Test key generated: {}", hex::encode(&key));
        
        let data = b"Test data";
        println!("Test data: {}", String::from_utf8_lossy(data));
        
        let encrypted = crypto.encrypt(&key, data, AesMode::GCM)
            .expect("Failed to encrypt test data");
        println!("Test data encrypted: {}", hex::encode(&encrypted));
        
        let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM)
            .expect("Failed to decrypt test data");
        println!("Test data decrypted: {}", String::from_utf8_lossy(&decrypted));
        
        assert_eq!(data, &decrypted[..]);
    }
} 