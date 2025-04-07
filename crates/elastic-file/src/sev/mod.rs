use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use crate::common::{FileError, FileMode, FileConfig, FileOperations, FileMetadata};
use sev::certs::sev::Usage;

struct FileHandle {
    #[allow(dead_code)]
    path: PathBuf,
    mode: FileMode,
    secure: bool,
    cipher: Option<Aes256Gcm>,
    file: Option<File>,
}

pub struct SevFileContext {
    files: Arc<Mutex<HashMap<u32, FileHandle>>>,
    next_handle: Arc<Mutex<u32>>,
}

impl SevFileContext {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(1)),
        }
    }

    fn create_cipher(&self, key: &[u8]) -> Result<Aes256Gcm, FileError> {
        Aes256Gcm::new_from_slice(key)
            .map_err(|_| FileError::OperationFailed("Failed to create cipher".to_string()))
    }

    fn open_file(&self, path: &PathBuf, mode: FileMode) -> Result<File, FileError> {
        let mut options = OpenOptions::new();
        match mode {
            FileMode::Read => options.read(true),
            FileMode::Write => options.write(true).create(true).truncate(true),
            FileMode::ReadWrite => options.read(true).write(true).create(true),
            FileMode::Append => options.append(true).create(true),
        };
        options.open(path)
            .map_err(|e| FileError::OperationFailed(format!("Failed to open file: {}", e)))
    }
}

impl Default for SevFileContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperations for SevFileContext {
    fn open(&self, config: &FileConfig) -> Result<u32, FileError> {
        let mut files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let mut next_handle = self.next_handle.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        
        let handle = *next_handle;
        *next_handle += 1;

        let file = self.open_file(&config.path, config.mode)?;

        let file_handle = FileHandle {
            path: config.path.clone(),
            mode: config.mode,
            secure: config.secure,
            cipher: if config.secure {
                Some(self.create_cipher(&[0u8; 32])?) // In production, use a proper key
            } else {
                None
            },
            file: Some(file),
        };

        files.insert(handle, file_handle);
        Ok(handle)
    }

    fn close(&self, handle: u32) -> Result<(), FileError> {
        let mut files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        files.remove(&handle).ok_or(FileError::NotFound)?;
        Ok(())
    }

    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        if !matches!(file_handle.mode, FileMode::Read | FileMode::ReadWrite) {
            return Err(FileError::OperationFailed("File not opened for reading".to_string()));
        }

        let mut file = file_handle.file.as_ref().ok_or(FileError::OperationFailed("File not open".to_string()))?;
        let mut data = vec![0u8; buf.len()];
        let len = file.read(&mut data).map_err(|e| FileError::OperationFailed(format!("Failed to read file: {}", e)))?;

        if file_handle.secure {
            if let Some(cipher) = &file_handle.cipher {
                let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a proper nonce
                let decrypted = cipher.decrypt(nonce, &data[..len])
                    .map_err(|_| FileError::OperationFailed("Decryption failed".to_string()))?;
                buf[..decrypted.len()].copy_from_slice(&decrypted);
                return Ok(decrypted.len());
            }
        }

        buf[..len].copy_from_slice(&data[..len]);
        Ok(len)
    }

    fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        if !matches!(file_handle.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append) {
            return Err(FileError::OperationFailed("File not opened for writing".to_string()));
        }

        let mut file = file_handle.file.as_ref().ok_or(FileError::OperationFailed("File not open".to_string()))?;
        let data = if file_handle.secure {
            if let Some(cipher) = &file_handle.cipher {
                let nonce = Nonce::from_slice(&[0u8; 12]); // In production, use a proper nonce
                cipher.encrypt(nonce, buf)
                    .map_err(|_| FileError::OperationFailed("Encryption failed".to_string()))?
            } else {
                buf.to_vec()
            }
        } else {
            buf.to_vec()
        };

        file.write(&data).map_err(|e| FileError::OperationFailed(format!("Failed to write file: {}", e)))
    }

    fn seek(&self, handle: u32, offset: i64, whence: i32) -> Result<u64, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        let mut file = file_handle.file.as_ref().ok_or(FileError::OperationFailed("File not open".to_string()))?;
        let seek_from = match whence {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return Err(FileError::OperationFailed("Invalid whence value".to_string())),
        };

        file.seek(seek_from).map_err(|e| FileError::OperationFailed(format!("Failed to seek file: {}", e)))
    }

    fn flush(&self, handle: u32) -> Result<(), FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        let mut file = file_handle.file.as_ref().ok_or(FileError::OperationFailed("File not open".to_string()))?;
        file.sync_all().map_err(|e| FileError::OperationFailed(format!("Failed to flush file: {}", e)))
    }

    fn metadata(&self, handle: u32) -> Result<FileMetadata, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;
        
        let metadata = std::fs::metadata(&file_handle.path)
            .map_err(|e| FileError::IoError(e))?;
        
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            permissions: 0o644, // Default permissions for all systems in SEV mode
        })
    }
} 