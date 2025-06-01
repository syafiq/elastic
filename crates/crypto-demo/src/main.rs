use elastic_crypto::{AesMode, Crypto};

#[cfg(feature = "linux")]
fn run_demo_linux() {
    println!("Crypto Demo (Linux)");
    let key = vec![0u8; 32];
    let data = b"Hello, ELASTIC Crypto!";
    let aes = elastic_crypto::AesKey::new(&key).unwrap();
    let encrypted = aes.encrypt(data, AesMode::GCM).unwrap();
    println!("Encrypted: {:x?}", encrypted);
    let decrypted = aes.decrypt(&encrypted, AesMode::GCM).unwrap();
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
}

#[cfg(feature = "sevsnp")]
fn run_demo_sevsnp() {
    println!("Crypto Demo (SEV-SNP)");
    let mut rng = elastic_crypto::SevsnpRng::new().unwrap();
    let key = rng.get_random_bytes(32).unwrap();
    let mut aes = elastic_crypto::SevsnpAes::new(&key).unwrap();
    let data = b"Hello, SEV-SNP Crypto!";
    let encrypted = aes.encrypt(data).unwrap();
    println!("Encrypted: {:x?}", encrypted);
    let decrypted = aes.decrypt(&encrypted).unwrap();
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
}

#[cfg(feature = "wasm")]
fn run_demo_wasm() {
    println!("Crypto Demo (WASM)");
    let crypto = elastic_crypto::WasmCrypto::new();
    let key = crypto.generate_key().unwrap();
    let data = b"Hello, WASM Crypto!";
    let encrypted = crypto.encrypt(&key, data, AesMode::GCM).unwrap();
    println!("Encrypted: {:x?}", encrypted);
    let decrypted = crypto.decrypt(&key, &encrypted, AesMode::GCM).unwrap();
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
}

fn main() {
    #[cfg(feature = "linux")]
    run_demo_linux();
    #[cfg(feature = "sevsnp")]
    run_demo_sevsnp();
    #[cfg(feature = "wasm")]
    run_demo_wasm();
}
