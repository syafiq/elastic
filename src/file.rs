use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::collections::HashMap;
use std::sync::Mutex;
use std::os::unix::fs::MetadataExt;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};

#[derive(Debug)]
pub enum FileError {
    NotFound(String),
    PermissionDenied(String),
    AlreadyExists(String),
    InvalidOperation(String),
    EncryptionError(String),
    DecryptionError(String),
    IoError(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FileError::NotFound(msg) => write!(f, "File not found: {}", msg),
            FileError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            FileError::AlreadyExists(msg) => write!(f, "File already exists: {}", msg),
            FileError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            FileError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            FileError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            FileError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for FileError {}

#[derive(Debug, Clone, Copy)]
pub enum FileMode {
    Read,
    Write,
    Append,
    ReadWrite,
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Regular,
    Directory,
    SymbolicLink,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub file_type: FileType,
    pub created: u64,
    pub modified: u64,
    pub accessed: u64,
    pub permissions: u32,
}

pub struct FileContainer {
    path: PathBuf,
    mode: FileMode,
    encryption_key: Option<Aes256Gcm>,
}

pub struct FileSystem {
    containers: Mutex<HashMap<u32, FileContainer>>,
    next_handle: Mutex<u32>,
}

impl FileSystem {
    pub fn new() -> Self {
        FileSystem {
            containers: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
        }
    }

    pub fn open_container(&self, path: &str, mode: FileMode) -> Result<u32, FileError> {
        let path = PathBuf::from(path);
        
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| FileError::IoError(e.to_string()))?;
        }

        let container = FileContainer {
            path,
            mode,
            encryption_key: None,
        };

        let mut handles = self.containers.lock().unwrap();
        let mut next_handle = self.next_handle.lock().unwrap();
        let handle = *next_handle;
        *next_handle += 1;

        handles.insert(handle, container);
        Ok(handle)
    }

    pub fn close_container(&self, handle: u32) -> Result<(), FileError> {
        let mut containers = self.containers.lock().unwrap();
        containers.remove(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        Ok(())
    }

    pub fn read_file(&self, handle: u32, path: &str) -> Result<Vec<u8>, FileError> {
        let containers = self.containers.lock().unwrap();
        let container = containers.get(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        let full_path = container.path.join(path);
        let mut file = File::open(&full_path).map_err(|e| FileError::IoError(e.to_string()))?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(|e| FileError::IoError(e.to_string()))?;

        if let Some(key) = &container.encryption_key {
            // Decrypt the contents
            let nonce = Nonce::from_slice(b"unique nonce"); // In production, use a unique nonce per file
            contents = key.decrypt(nonce, contents.as_ref())
                .map_err(|e| FileError::DecryptionError(e.to_string()))?;
        }

        Ok(contents)
    }

    pub fn write_file(&self, handle: u32, path: &str, data: &[u8]) -> Result<(), FileError> {
        let containers = self.containers.lock().unwrap();
        let container = containers.get(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        let full_path = container.path.join(path);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&full_path)
            .map_err(|e| FileError::IoError(e.to_string()))?;

        let data = if let Some(key) = &container.encryption_key {
            // Encrypt the data
            let nonce = Nonce::from_slice(b"unique nonce"); // In production, use a unique nonce per file
            key.encrypt(nonce, data)
                .map_err(|e| FileError::EncryptionError(e.to_string()))?
        } else {
            data.to_vec()
        };

        file.write_all(&data).map_err(|e| FileError::IoError(e.to_string()))?;
        Ok(())
    }

    pub fn delete_file(&self, handle: u32, path: &str) -> Result<(), FileError> {
        let containers = self.containers.lock().unwrap();
        let container = containers.get(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        let full_path = container.path.join(path);
        fs::remove_file(&full_path).map_err(|e| FileError::IoError(e.to_string()))?;
        Ok(())
    }

    pub fn list_files(&self, handle: u32, path: &str) -> Result<Vec<String>, FileError> {
        let containers = self.containers.lock().unwrap();
        let container = containers.get(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        let full_path = container.path.join(path);
        let entries = fs::read_dir(&full_path).map_err(|e| FileError::IoError(e.to_string()))?;
        
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| FileError::IoError(e.to_string()))?;
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
        Ok(files)
    }

    pub fn get_metadata(&self, handle: u32, path: &str) -> Result<FileMetadata, FileError> {
        let containers = self.containers.lock().unwrap();
        let container = containers.get(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        let full_path = container.path.join(path);
        let metadata = fs::metadata(&full_path).map_err(|e| FileError::IoError(e.to_string()))?;
        
        let file_type = if metadata.is_file() {
            FileType::Regular
        } else if metadata.is_dir() {
            FileType::Directory
        } else if metadata.file_type().is_symlink() {
            FileType::SymbolicLink
        } else {
            return Err(FileError::InvalidOperation("Unknown file type".to_string()));
        };

        Ok(FileMetadata {
            name: path.to_string(),
            size: metadata.len(),
            file_type,
            created: metadata.created()
                .map_err(|e| FileError::IoError(e.to_string()))?
                .duration_since(UNIX_EPOCH)
                .map_err(|e| FileError::IoError(e.to_string()))?
                .as_secs(),
            modified: metadata.modified()
                .map_err(|e| FileError::IoError(e.to_string()))?
                .duration_since(UNIX_EPOCH)
                .map_err(|e| FileError::IoError(e.to_string()))?
                .as_secs(),
            accessed: metadata.accessed()
                .map_err(|e| FileError::IoError(e.to_string()))?
                .duration_since(UNIX_EPOCH)
                .map_err(|e| FileError::IoError(e.to_string()))?
                .as_secs(),
            permissions: metadata.mode(),
        })
    }

    pub fn load_key(&self, handle: u32, key: &[u8]) -> Result<(), FileError> {
        let mut containers = self.containers.lock().unwrap();
        let container = containers.get_mut(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        if key.len() != 32 {
            return Err(FileError::InvalidOperation("Key must be 32 bytes".to_string()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        container.encryption_key = Some(Aes256Gcm::new(key));
        Ok(())
    }

    pub fn remove_key(&self, handle: u32) -> Result<(), FileError> {
        let mut containers = self.containers.lock().unwrap();
        let container = containers.get_mut(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        container.encryption_key = None;
        Ok(())
    }

    pub fn is_encrypted(&self, handle: u32, _path: &str) -> Result<bool, FileError> {
        let containers = self.containers.lock().unwrap();
        let container = containers.get(&handle).ok_or_else(|| FileError::NotFound("Container not found".to_string()))?;
        
        Ok(container.encryption_key.is_some())
    }
} 