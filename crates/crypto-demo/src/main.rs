use elastic_crypto::{AesMode, ElasticCrypto, Crypto};

fn run_demo() {
    println!("Crypto Demo (Unified)");
    let crypto = ElasticCrypto::new().unwrap();
    let key = crypto.generate_key().unwrap();
    let data = b"Hello, ELASTIC Crypto!";
    
    println!("\nEncrypting data...");
    let encrypted = crypto.encrypt(&key, data, AesMode::GCM).unwrap();
    println!("Encrypted: {:x?}", encrypted);
    
    println!("\nDecrypting data...");
    let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM).unwrap();
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
}

fn main() {
    run_demo();
}
