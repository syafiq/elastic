use elastic_file::{FileConfig, FileContext, FileMode, SeekFrom};
use std::io::Write;
use tempfile::NamedTempFile;
use tokio;

#[tokio::test]
async fn test_file_open_close() {
    let ctx = FileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    let config = FileConfig::default();
    
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    assert!(handle > 0);
    
    ctx.close(handle).await.unwrap();
}

#[tokio::test]
async fn test_file_read_write() {
    let ctx = FileContext::new();
    let mut temp_file = NamedTempFile::new().unwrap();
    
    // Write some data to the temp file
    write!(temp_file, "Hello, World!").unwrap();
    temp_file.flush().unwrap();
    
    // Open for reading
    let mut config = FileConfig::default();
    config.mode = FileMode::Read;
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    
    // Read the data
    let mut buf = vec![0; 13];
    let bytes_read = ctx.read(handle, &mut buf).await.unwrap();
    assert_eq!(bytes_read, 13);
    assert_eq!(String::from_utf8_lossy(&buf), "Hello, World!");
    
    ctx.close(handle).await.unwrap();
    
    // Open for writing
    config.mode = FileMode::Write;
    config.truncate_if_exists = true;
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    
    // Write new data
    let data = b"New content";
    let bytes_written = ctx.write(handle, data).await.unwrap();
    assert_eq!(bytes_written, data.len());
    
    ctx.close(handle).await.unwrap();
}

#[tokio::test]
async fn test_file_seek() {
    let ctx = FileContext::new();
    let mut temp_file = NamedTempFile::new().unwrap();
    
    // Write test data
    write!(temp_file, "Hello, World!").unwrap();
    temp_file.flush().unwrap();
    
    let mut config = FileConfig::default();
    config.mode = FileMode::ReadWrite;
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    
    // Seek to position 7 (start of "World")
    let pos = ctx.seek(handle, 7, SeekFrom::Start).await.unwrap();
    assert_eq!(pos, 7);
    
    // Read the word "World"
    let mut buf = vec![0; 5];
    let bytes_read = ctx.read(handle, &mut buf).await.unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(String::from_utf8_lossy(&buf), "World");
    
    ctx.close(handle).await.unwrap();
}

#[tokio::test]
async fn test_file_metadata() {
    let ctx = FileContext::new();
    let mut temp_file = NamedTempFile::new().unwrap();
    
    // Write some data
    write!(temp_file, "Hello, World!").unwrap();
    temp_file.flush().unwrap();
    
    let config = FileConfig::default();
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    
    let metadata = ctx.metadata(handle).await.unwrap();
    assert_eq!(metadata.size, 13);
    assert!(metadata.is_file);
    assert!(!metadata.is_dir);
    
    ctx.close(handle).await.unwrap();
}

#[tokio::test]
async fn test_invalid_handle() {
    let ctx = FileContext::new();
    
    let mut buf = vec![0; 10];
    assert!(ctx.read(0, &mut buf).await.is_err());
    assert!(ctx.write(0, b"test").await.is_err());
    assert!(ctx.seek(0, 0, SeekFrom::Start).await.is_err());
    assert!(ctx.flush(0).await.is_err());
    assert!(ctx.metadata(0).await.is_err());
    assert!(ctx.close(0).await.is_err());
}

#[tokio::test]
async fn test_file_modes() {
    let ctx = FileContext::new();
    let temp_file = NamedTempFile::new().unwrap();
    
    // Test read-only mode
    let mut config = FileConfig::default();
    config.mode = FileMode::Read;
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    assert!(ctx.write(handle, b"test").await.is_err());
    ctx.close(handle).await.unwrap();
    
    // Test write-only mode
    config.mode = FileMode::Write;
    let handle = ctx.open(temp_file.path(), &config).await.unwrap();
    let mut buf = vec![0; 10];
    assert!(ctx.read(handle, &mut buf).await.is_err());
    ctx.close(handle).await.unwrap();
} 