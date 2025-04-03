use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::sync::Mutex;

use crate::common::{FileConfig, FileMetadata, FileMode};

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

    pub fn open(&self, config: &FileConfig) -> Result<u32, String> {
        let mut files = self.files.lock().map_err(|e| e.to_string())?;
        let mut next_handle = self.next_handle.lock().map_err(|e| e.to_string())?;
        
        let handle = *next_handle;
        *next_handle += 1;

        let file = OpenOptions::new()
            .read(matches!(config.mode, FileMode::Read | FileMode::ReadWrite))
            .write(matches!(config.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append))
            .append(matches!(config.mode, FileMode::Append))
            .create(matches!(config.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append))
            .open(&config.path)
            .map_err(|e| e.to_string())?;

        let file_handle = FileHandle {
            file,
            config: config.clone(),
        };

        files.insert(handle, file_handle);
        Ok(handle)
    }

    pub fn close(&self, handle: u32) -> Result<(), String> {
        let mut files = self.files.lock().map_err(|e| e.to_string())?;
        files.remove(&handle).ok_or_else(|| "File not found".to_string())?;
        Ok(())
    }

    pub fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, String> {
        let files = self.files.lock().map_err(|e| e.to_string())?;
        let file_handle = files.get(&handle).ok_or_else(|| "File not found".to_string())?;

        if !matches!(file_handle.config.mode, FileMode::Read | FileMode::ReadWrite) {
            return Err("File not opened for reading".to_string());
        }

        let mut file = &file_handle.file;
        file.read(buf).map_err(|e| e.to_string())
    }

    pub fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, String> {
        let files = self.files.lock().map_err(|e| e.to_string())?;
        let file_handle = files.get(&handle).ok_or_else(|| "File not found".to_string())?;

        if !matches!(file_handle.config.mode, FileMode::Write | FileMode::ReadWrite | FileMode::Append) {
            return Err("File not opened for writing".to_string());
        }

        let mut file = &file_handle.file;
        file.write(buf).map_err(|e| e.to_string())
    }

    pub fn seek(&self, handle: u32, offset: i64, whence: i32) -> Result<u64, String> {
        let files = self.files.lock().map_err(|e| e.to_string())?;
        let file_handle = files.get(&handle).ok_or_else(|| "File not found".to_string())?;

        let mut file = &file_handle.file;
        let seek_from = match whence {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return Err("Invalid seek whence".to_string()),
        };

        file.seek(seek_from).map_err(|e| e.to_string())
    }

    pub fn flush(&self, handle: u32) -> Result<(), String> {
        let files = self.files.lock().map_err(|e| e.to_string())?;
        let file_handle = files.get(&handle).ok_or_else(|| "File not found".to_string())?;

        let mut file = &file_handle.file;
        file.flush().map_err(|e| e.to_string())
    }

    pub fn metadata(&self, handle: u32) -> Result<FileMetadata, String> {
        let files = self.files.lock().map_err(|e| e.to_string())?;
        let file_handle = files.get(&handle).ok_or_else(|| "File not found".to_string())?;

        let metadata = file_handle.file.metadata().map_err(|e| e.to_string())?;
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            permissions: metadata.mode(),
        })
    }
} 