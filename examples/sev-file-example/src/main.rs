use elastic_file::{FileConfig, FileContext, FileMode, FileOperations};

fn main() {
    println!("Starting SEV-SNP file example...");
    
    let ctx = FileContext::new();
    let config = FileConfig {
        path: "test.txt".into(),
        mode: FileMode::ReadWrite,
        secure: true,
    };

    println!("Opening file...");
    let handle = ctx.open(&config).expect("Failed to open file");
    println!("File opened with handle: {}", handle);

    println!("Writing data...");
    let data = b"Hello, SEV-SNP!";
    let written = ctx.write(handle, data).expect("Failed to write data");
    println!("Wrote {} bytes", written);

    println!("Reading data...");
    let mut buf = vec![0u8; 1024];
    let read = ctx.read(handle, &mut buf).expect("Failed to read data");
    println!("Read {} bytes", read);
    println!("Data: {}", String::from_utf8_lossy(&buf[..read]));

    println!("Getting metadata...");
    let metadata = ctx.metadata(handle).expect("Failed to get metadata");
    println!("File size: {}", metadata.size);
    println!("Is file: {}", metadata.is_file);
    println!("Is directory: {}", metadata.is_dir);

    println!("Closing file...");
    ctx.close(handle).expect("Failed to close file");
    println!("✅ File operations successful!");
} 