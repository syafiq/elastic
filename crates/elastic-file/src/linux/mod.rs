use crate::common::{FileError, FileConfig, FileOperations, FileMetadata};

mod file;
use file::FileManager;

pub struct FileContext {
    manager: FileManager,
}

impl FileContext {
    pub fn new() -> Self {
        Self {
            manager: FileManager::new(),
        }
    }
}

impl Default for FileContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperations for FileContext {
    fn open(&self, config: &FileConfig) -> Result<u32, FileError> {
        self.manager.open(config)
    }

    fn close(&self, handle: u32) -> Result<(), FileError> {
        self.manager.close(handle)
    }

    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError> {
        self.manager.read(handle, buf)
    }

    fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, FileError> {
        self.manager.write(handle, buf)
    }

    fn seek(&self, handle: u32, offset: i64, whence: i32) -> Result<u64, FileError> {
        self.manager.seek(handle, offset, whence)
    }

    fn flush(&self, handle: u32) -> Result<(), FileError> {
        self.manager.flush(handle)
    }

    fn metadata(&self, handle: u32) -> Result<FileMetadata, FileError> {
        self.manager.metadata(handle)
    }
} 