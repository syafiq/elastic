#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::sync::Mutex;
use std::collections::HashMap;
use crate::common::{FileError, FileMode, FileConfig, FileMetadata};

pub struct FileManager {
    files: Mutex<HashMap<u32, FileHandle>>,
    next_handle: Mutex<u32>,
}

struct FileHandle {
    file: File,
    config: FileConfig,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: Mutex::new(HashMap::new()),
            next_handle: Mutex::new(1),
        }
    }

    pub fn open(&self, config: &FileConfig) -> Result<u32, FileError> {
        let mut files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let mut next_handle = self.next_handle.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        
        let handle = *next_handle;
        *next_handle += 1;

        let file = OpenOptions::new()
            .read(matches!(config.mode, FileMode::Read | FileMode::ReadWrite))
            .write(matches!(config.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append))
            .append(matches!(config.mode, FileMode::Append))
            .create(matches!(config.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append))
            .open(&config.path)
            .map_err(|e| FileError::IoError(e))?;

        let file_handle = FileHandle {
            file,
            config: config.clone(),
        };

        files.insert(handle, file_handle);
        Ok(handle)
    }

    pub fn close(&self, handle: u32) -> Result<(), FileError> {
        let mut files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        files.remove(&handle).ok_or(FileError::NotFound)?;
        Ok(())
    }

    pub fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        if !matches!(file_handle.config.mode, FileMode::Read | FileMode::ReadWrite) {
            return Err(FileError::InvalidMode);
        }

        let mut file = &file_handle.file;
        file.read(buf).map_err(|e| FileError::IoError(e))
    }

    pub fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        if !matches!(file_handle.config.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append) {
            return Err(FileError::InvalidMode);
        }

        let mut file = &file_handle.file;
        file.write(buf).map_err(|e| FileError::IoError(e))
    }

    pub fn seek(&self, handle: u32, offset: i64, whence: i32) -> Result<u64, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        let mut file = &file_handle.file;
        let seek_from = match whence {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return Err(FileError::OperationFailed("Invalid seek whence".to_string())),
        };

        file.seek(seek_from).map_err(|e| FileError::IoError(e))
    }

    pub fn flush(&self, handle: u32) -> Result<(), FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::NotFound)?;

        let mut file = &file_handle.file;
        file.flush().map_err(|e| FileError::IoError(e))
    }

    pub fn metadata(&self, handle: u32) -> Result<FileMetadata, FileError> {
        let files = self.files.lock().map_err(|e| FileError::OperationFailed(e.to_string()))?;
        let file_handle = files.get(&handle).ok_or(FileError::InvalidHandle)?;
        
        let metadata = std::fs::metadata(&file_handle.config.path)
            .map_err(|e| FileError::IoError(e))?;
        
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            #[cfg(unix)]
            permissions: metadata.mode(),
            #[cfg(not(unix))]
            permissions: 0o644, // Default permissions for non-Unix systems
        })
    }
} 