use super::*;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_file_operations() {
    let ctx = SevFileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Test non-secure file operations
    let config = FileConfig {
        path: path.clone(),
        mode: FileMode::ReadWrite,
        secure: false,
    };

    let handle = ctx.open(&config).unwrap();
    
    // Write data
    let data = b"Hello, World!";
    let written = ctx.write(handle, data).unwrap();
    assert_eq!(written, data.len());

    // Read data
    let mut buf = vec![0u8; 1024];
    let read = ctx.read(handle, &mut buf).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf[..read], data);

    // Get metadata
    let metadata = ctx.metadata(handle).unwrap();
    assert_eq!(metadata.size, data.len() as u64);
    assert!(metadata.is_file);
    assert!(!metadata.is_dir);

    // Close file
    ctx.close(handle).unwrap();
}

#[test]
fn test_secure_file_operations() {
    let ctx = SevFileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Test secure file operations
    let config = FileConfig {
        path: path.clone(),
        mode: FileMode::ReadWrite,
        secure: true,
    };

    let handle = ctx.open(&config).unwrap();
    
    // Write encrypted data
    let data = b"Hello, SEV-SNP!";
    let written = ctx.write(handle, data).unwrap();
    assert!(written > data.len()); // Encrypted data should be larger

    // Read and decrypt data
    let mut buf = vec![0u8; 1024];
    let read = ctx.read(handle, &mut buf).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf[..read], data);

    // Get metadata
    let metadata = ctx.metadata(handle).unwrap();
    assert_eq!(metadata.size, written as u64);
    assert!(metadata.is_file);
    assert!(!metadata.is_dir);

    // Close file
    ctx.close(handle).unwrap();
}

#[test]
fn test_file_seek() {
    let ctx = SevFileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    let config = FileConfig {
        path: path.clone(),
        mode: FileMode::ReadWrite,
        secure: false,
    };

    let handle = ctx.open(&config).unwrap();
    
    // Write data
    let data = b"Hello, World!";
    ctx.write(handle, data).unwrap();

    // Seek to start
    let pos = ctx.seek(handle, 0, 0).unwrap();
    assert_eq!(pos, 0);

    // Read from start
    let mut buf = vec![0u8; 5];
    let read = ctx.read(handle, &mut buf).unwrap();
    assert_eq!(read, 5);
    assert_eq!(&buf[..read], b"Hello");

    // Seek to middle
    let pos = ctx.seek(handle, 7, 0).unwrap();
    assert_eq!(pos, 7);

    // Read from middle
    let mut buf = vec![0u8; 6];
    let read = ctx.read(handle, &mut buf).unwrap();
    assert_eq!(read, 6);
    assert_eq!(&buf[..read], b"World!");

    ctx.close(handle).unwrap();
}

#[test]
fn test_file_flush() {
    let ctx = SevFileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    let config = FileConfig {
        path: path.clone(),
        mode: FileMode::ReadWrite,
        secure: false,
    };

    let handle = ctx.open(&config).unwrap();
    
    // Write data
    let data = b"Hello, World!";
    ctx.write(handle, data).unwrap();

    // Flush should succeed
    assert!(ctx.flush(handle).is_ok());

    ctx.close(handle).unwrap();
}

#[test]
fn test_invalid_operations() {
    let ctx = SevFileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    let config = FileConfig {
        path: path.clone(),
        mode: FileMode::Read,
        secure: false,
    };

    let handle = ctx.open(&config).unwrap();
    
    // Try to write to read-only file
    let data = b"Hello, World!";
    assert!(ctx.write(handle, data).is_err());

    // Try to read from non-existent handle
    let mut buf = vec![0u8; 1024];
    assert!(ctx.read(999, &mut buf).is_err());

    ctx.close(handle).unwrap();
} 