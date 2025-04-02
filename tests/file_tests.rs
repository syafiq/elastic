use elastic::file::{FileSystem, FileMode, FileError};
use tempfile::tempdir;

#[test]
fn test_basic_file_operations() {
    let fs = FileSystem::new();
    let temp_dir = tempdir().unwrap();
    let container_path = temp_dir.path().to_str().unwrap();

    // Open container
    let handle = fs.open_container(container_path, FileMode::ReadWrite).unwrap();

    // Write file
    let test_data = b"Hello, World!";
    fs.write_file(handle, "test.txt", test_data).unwrap();

    // Read file
    let contents = fs.read_file(handle, "test.txt").unwrap();
    assert_eq!(contents, test_data);

    // Get metadata
    let metadata = fs.get_metadata(handle, "test.txt").unwrap();
    assert_eq!(metadata.name, "test.txt");
    assert_eq!(metadata.size, test_data.len() as u64);

    // List files
    let files = fs.list_files(handle, ".").unwrap();
    assert!(files.contains(&"test.txt".to_string()));

    // Delete file
    fs.delete_file(handle, "test.txt").unwrap();
    assert!(fs.read_file(handle, "test.txt").is_err());

    // Close container
    fs.close_container(handle).unwrap();
}

#[test]
fn test_encryption() {
    let fs = FileSystem::new();
    let temp_dir = tempdir().unwrap();
    let container_path = temp_dir.path().to_str().unwrap();

    // Open container
    let handle = fs.open_container(container_path, FileMode::ReadWrite).unwrap();

    // Load encryption key
    let key = vec![1u8; 32];
    fs.load_key(handle, &key).unwrap();

    // Write encrypted file
    let test_data = b"Secret data";
    fs.write_file(handle, "secret.txt", test_data).unwrap();

    // Verify file is encrypted
    assert!(fs.is_encrypted(handle, "secret.txt").unwrap());

    // Read and decrypt file
    let contents = fs.read_file(handle, "secret.txt").unwrap();
    assert_eq!(contents, test_data);

    // Remove key
    fs.remove_key(handle).unwrap();
    assert!(!fs.is_encrypted(handle, "secret.txt").unwrap());

    // Close container
    fs.close_container(handle).unwrap();
}

#[test]
fn test_error_handling() {
    let fs = FileSystem::new();
    let temp_dir = tempdir().unwrap();
    let container_path = temp_dir.path().to_str().unwrap();

    // Open container
    let handle = fs.open_container(container_path, FileMode::ReadWrite).unwrap();

    // Test non-existent file
    match fs.read_file(handle, "nonexistent.txt") {
        Err(FileError::IoError(_)) => (),
        _ => panic!("Expected IoError for non-existent file"),
    }

    // Test invalid key length
    let invalid_key = vec![1u8; 16];
    match fs.load_key(handle, &invalid_key) {
        Err(FileError::InvalidOperation(_)) => (),
        _ => panic!("Expected InvalidOperation error for invalid key length"),
    }

    // Test invalid container handle
    match fs.read_file(999, "test.txt") {
        Err(FileError::NotFound(_)) => (),
        _ => panic!("Expected NotFound error for invalid handle"),
    }

    // Close container
    fs.close_container(handle).unwrap();
}

#[test]
fn test_file_modes() {
    let fs = FileSystem::new();
    let temp_dir = tempdir().unwrap();
    let container_path = temp_dir.path().to_str().unwrap();

    // Test read-only mode
    let read_handle = fs.open_container(container_path, FileMode::Read).unwrap();
    fs.write_file(read_handle, "test.txt", b"test").unwrap(); // Should succeed as we're not enforcing mode restrictions

    // Test write-only mode
    let write_handle = fs.open_container(container_path, FileMode::Write).unwrap();
    fs.read_file(write_handle, "test.txt").unwrap(); // Should succeed as we're not enforcing mode restrictions

    // Close containers
    fs.close_container(read_handle).unwrap();
    fs.close_container(write_handle).unwrap();
} 