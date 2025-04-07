use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found")]
    NotFound,
    #[error("Invalid file handle")]
    InvalidHandle,
    #[error("Invalid file mode")]
    InvalidMode,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<String> for FileError {
    fn from(err: String) -> Self {
        FileError::OperationFailed(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileMode {
    Read,
    Write,
    ReadWrite,
    Append,
}

impl FileMode {
    pub fn can_write(&self) -> bool {
        matches!(self, FileMode::Write | FileMode::ReadWrite | FileMode::Append)
    }
}

#[derive(Debug, Clone)]
pub struct FileConfig {
    pub mode: FileMode,
    pub path: PathBuf,
    pub secure: bool,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            mode: FileMode::Read,
            path: PathBuf::new(),
            secure: false,
        }
    }
}

pub trait FileOperations {
    fn open(&self, config: &FileConfig) -> Result<u32, FileError>;
    fn close(&self, handle: u32) -> Result<(), FileError>;
    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError>;
    fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, FileError>;
    fn seek(&self, handle: u32, offset: i64, whence: i32) -> Result<u64, FileError>;
    fn flush(&self, handle: u32) -> Result<(), FileError>;
    fn metadata(&self, handle: u32) -> Result<FileMetadata, FileError>;
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub permissions: u32,
} 