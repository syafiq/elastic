use elastic_file::{FileConfig, FileContext, FileMode};
use std::path::PathBuf;

fn main() {
    println!("Starting WASM file operations test...");
    
    // Create a file context
    let context = FileContext::new();
    
    // Test file path
    let test_path = PathBuf::from("test.txt");
    
    // Test writing
    let write_config = FileConfig {
        mode: FileMode::Write,
        path: test_path.clone(),
        secure: true,
    };
    
    println!("Opening file for writing...");
    let handle = match context.open(&write_config) {
        Ok(h) => h,
        Err(e) => {
            println!("Error opening file: {}", e);
            return;
        }
    };
    
    let test_data = b"Hello, WASM!";
    println!("Writing test data...");
    match context.write(handle, test_data) {
        Ok(len) => println!("Wrote {} bytes", len),
        Err(e) => println!("Error writing file: {}", e),
    }
    
    println!("Flushing file...");
    match context.flush(handle) {
        Ok(_) => println!("File flushed successfully"),
        Err(e) => println!("Error flushing file: {}", e),
    }
    
    println!("Closing file...");
    match context.close(handle) {
        Ok(_) => println!("File closed successfully"),
        Err(e) => println!("Error closing file: {}", e),
    }
    
    // Test reading
    let read_config = FileConfig {
        mode: FileMode::Read,
        path: test_path,
        secure: true,
    };
    
    println!("Opening file for reading...");
    let handle = match context.open(&read_config) {
        Ok(h) => h,
        Err(e) => {
            println!("Error opening file: {}", e);
            return;
        }
    };
    
    let mut read_buf = vec![0u8; 100];
    println!("Reading file...");
    match context.read(handle, &mut read_buf) {
        Ok(len) => {
            println!("Read {} bytes", len);
            if let Ok(s) = String::from_utf8(read_buf[..len].to_vec()) {
                println!("File contents: {}", s);
            }
        }
        Err(e) => println!("Error reading file: {}", e),
    }
    
    println!("Closing file...");
    match context.close(handle) {
        Ok(_) => println!("File closed successfully"),
        Err(e) => println!("Error closing file: {}", e),
    }
    
    println!("Test completed!");
} 