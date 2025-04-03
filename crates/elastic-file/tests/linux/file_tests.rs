use std::path::PathBuf;
use tempfile::tempdir;
use elastic_file::{FileContext, FileConfig, FileMode};

#[test]
fn test_file_operations() {
    let ctx = FileContext::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");

    // Test file creation and writing
    let config = FileConfig {
        mode: FileMode::Write,
        path: file_path.clone(),
        secure: false,
    };
    
    let handle = ctx.open(&config).unwrap();
    let data = b"Hello, World!";
    let written = ctx.write(handle, data).unwrap();
    assert_eq!(written, data.len());
    ctx.flush(handle).unwrap();
    ctx.close(handle).unwrap();

    // Test file reading
    let config = FileConfig {
        mode: FileMode::Read,
        path: file_path.clone(),
        secure: false,
    };
    
    let handle = ctx.open(&config).unwrap();
    let mut buf = vec![0; 13];
    let read = ctx.read(handle, &mut buf).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf, data);
    ctx.close(handle).unwrap();

    // Test file metadata
    let handle = ctx.open(&config).unwrap();
    let metadata = ctx.metadata(handle).unwrap();
    assert_eq!(metadata.size, data.len() as u64);
    assert!(metadata.is_file);
    assert!(!metadata.is_dir);
    ctx.close(handle).unwrap();
}

#[test]
fn test_file_seeking() {
    let ctx = FileContext::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("seek_test.txt");

    // Create and write to file
    let config = FileConfig {
        mode: FileMode::ReadWrite,
        path: file_path.clone(),
        secure: false,
    };
    
    let handle = ctx.open(&config).unwrap();
    let data = b"Hello, World!";
    ctx.write(handle, data).unwrap();
    
    // Test seeking
    let pos = ctx.seek(handle, 0, 0).unwrap(); // SEEK_SET
    assert_eq!(pos, 0);
    
    let pos = ctx.seek(handle, 6, 0).unwrap(); // SEEK_SET
    assert_eq!(pos, 6);
    
    let mut buf = vec![0; 7];
    let read = ctx.read(handle, &mut buf).unwrap();
    assert_eq!(read, 7);
    assert_eq!(&buf, b"World!");
    
    ctx.close(handle).unwrap();
}

#[test]
fn test_error_handling() {
    let ctx = FileContext::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("error_test.txt");

    // Test invalid handle
    assert!(ctx.close(999).is_err());
    assert!(ctx.read(999, &mut [0; 10]).is_err());
    assert!(ctx.write(999, b"test").is_err());
    
    // Test invalid operations
    let config = FileConfig {
        mode: FileMode::Read,
        path: file_path.clone(),
        secure: false,
    };
    
    // Try to write to read-only file
    let handle = ctx.open(&config).unwrap();
    assert!(ctx.write(handle, b"test").is_err());
    ctx.close(handle).unwrap();
    
    // Try to read from write-only file
    let config = FileConfig {
        mode: FileMode::Write,
        path: file_path,
        secure: false,
    };
    
    let handle = ctx.open(&config).unwrap();
    assert!(ctx.read(handle, &mut [0; 10]).is_err());
    ctx.close(handle).unwrap();
} 