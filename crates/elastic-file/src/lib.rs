use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Invalid file path")]
    InvalidPath,
    #[error("File not found")]
    NotFound,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("File already exists")]
    AlreadyExists,
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileMode {
    Read,
    Write,
    Append,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct FileConfig {
    pub mode: FileMode,
    pub create_if_missing: bool,
    pub truncate_if_exists: bool,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            mode: FileMode::Read,
            create_if_missing: false,
            truncate_if_exists: false,
        }
    }
}

pub struct FileContext {
    manager: std::sync::Arc<file::FileManager>,
}

impl FileContext {
    pub fn new() -> Self {
        Self {
            manager: std::sync::Arc::new(file::FileManager::new()),
        }
    }

    pub async fn open(&self, path: impl Into<PathBuf>, config: &FileConfig) -> Result<u32, FileError> {
        self.manager.open(path.into(), config).await
            .map_err(|e| FileError::OperationFailed(e))
    }

    pub async fn close(&self, handle: u32) -> Result<(), FileError> {
        self.manager.close(handle).await
            .map_err(|e| FileError::OperationFailed(e))
    }

    pub async fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        self.manager.read(handle, buf).await
            .map_err(|e| FileError::OperationFailed(e))
    }

    pub async fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, FileError> {
        self.manager.write(handle, buf).await
            .map_err(|e| FileError::OperationFailed(e))
    }

    pub async fn seek(&self, handle: u32, offset: i64, whence: SeekFrom) -> Result<u64, FileError> {
        self.manager.seek(handle, offset, whence).await
            .map_err(|e| FileError::OperationFailed(e))
    }

    pub async fn flush(&self, handle: u32) -> Result<(), FileError> {
        self.manager.flush(handle).await
            .map_err(|e| FileError::OperationFailed(e))
    }

    pub async fn metadata(&self, handle: u32) -> Result<FileMetadata, FileError> {
        self.manager.metadata(handle).await
            .map_err(|e| FileError::OperationFailed(e))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeekFrom {
    Start,
    Current,
    End,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub permissions: u32,
    pub modified: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

mod file; 