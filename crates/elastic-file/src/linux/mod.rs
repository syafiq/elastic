use std::sync::Arc;

use crate::common::{FileConfig, FileError, FileMetadata, FileOperations};

mod file;

pub struct FileContext {
    manager: Arc<file::FileManager>,
}

impl FileContext {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(file::FileManager::new()),
        }
    }
}

impl FileOperations for FileContext {
    fn open(&self, config: &FileConfig) -> Result<u32, FileError> {
        self.manager.open(config).map_err(|e| FileError::OperationFailed(e))
    }

    fn close(&self, handle: u32) -> Result<(), FileError> {
        self.manager.close(handle).map_err(|e| FileError::OperationFailed(e))
    }

    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        self.manager.read(handle, buf).map_err(|e| FileError::OperationFailed(e))
    }

    fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, FileError> {
        self.manager.write(handle, buf).map_err(|e| FileError::OperationFailed(e))
    }

    fn seek(&self, handle: u32, offset: i64, whence: i32) -> Result<u64, FileError> {
        self.manager.seek(handle, offset, whence).map_err(|e| FileError::OperationFailed(e))
    }

    fn flush(&self, handle: u32) -> Result<(), FileError> {
        self.manager.flush(handle).map_err(|e| FileError::OperationFailed(e))
    }

    fn metadata(&self, handle: u32) -> Result<FileMetadata, FileError> {
        self.manager.metadata(handle).map_err(|e| FileError::OperationFailed(e))
    }
}

impl Default for FileContext {
    fn default() -> Self {
        Self::new()
    }
} 