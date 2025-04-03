use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::SeekFrom as StdSeekFrom;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::{FileConfig, FileMetadata, FileMode, SeekFrom};

struct FileEntry {
    file: TokioFile,
    path: PathBuf,
    mode: FileMode,
}

pub struct FileManager {
    files: Arc<Mutex<HashMap<u32, FileEntry>>>,
    next_handle: Arc<Mutex<u32>>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn open(&self, path: PathBuf, config: &FileConfig) -> Result<u32, String> {
        let mut options = OpenOptions::new();

        match config.mode {
            FileMode::Read => {
                options.read(true);
            }
            FileMode::Write => {
                options.write(true);
            }
            FileMode::Append => {
                options.append(true);
            }
            FileMode::ReadWrite => {
                options.read(true).write(true);
            }
        }

        if config.create_if_missing {
            options.create(true);
        }
        if config.truncate_if_exists {
            options.truncate(true);
        }

        let file = TokioFile::from_std(options.open(&path).map_err(|e| e.to_string())?);

        let mut handles = self.next_handle.lock().unwrap();
        let handle = *handles;
        *handles += 1;

        let entry = FileEntry {
            file,
            path,
            mode: config.mode,
        };

        let mut files = self.files.lock().unwrap();
        files.insert(handle, entry);

        Ok(handle)
    }

    pub async fn close(&self, handle: u32) -> Result<(), String> {
        let mut files = self.files.lock().unwrap();
        files.remove(&handle)
            .map(|_| ())
            .ok_or_else(|| format!("File handle {} not found", handle))
    }

    pub async fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, String> {
        let mut files = self.files.lock().unwrap();
        let entry = files.get_mut(&handle)
            .ok_or_else(|| format!("File handle {} not found", handle))?;

        if !matches!(entry.mode, FileMode::Read | FileMode::ReadWrite) {
            return Err("File not opened for reading".to_string());
        }

        entry.file.read(buf).await
            .map_err(|e| e.to_string())
    }

    pub async fn write(&self, handle: u32, buf: &[u8]) -> Result<usize, String> {
        let mut files = self.files.lock().unwrap();
        let entry = files.get_mut(&handle)
            .ok_or_else(|| format!("File handle {} not found", handle))?;

        if !matches!(entry.mode, FileMode::Write | FileMode::Append | FileMode::ReadWrite) {
            return Err("File not opened for writing".to_string());
        }

        entry.file.write(buf).await
            .map_err(|e| e.to_string())
    }

    pub async fn seek(&self, handle: u32, offset: i64, whence: SeekFrom) -> Result<u64, String> {
        let mut files = self.files.lock().unwrap();
        let entry = files.get_mut(&handle)
            .ok_or_else(|| format!("File handle {} not found", handle))?;

        let std_whence = match whence {
            SeekFrom::Start => StdSeekFrom::Start(offset as u64),
            SeekFrom::Current => StdSeekFrom::Current(offset),
            SeekFrom::End => StdSeekFrom::End(offset),
        };

        entry.file.seek(std_whence).await
            .map_err(|e| e.to_string())
    }

    pub async fn flush(&self, handle: u32) -> Result<(), String> {
        let mut files = self.files.lock().unwrap();
        let entry = files.get_mut(&handle)
            .ok_or_else(|| format!("File handle {} not found", handle))?;

        entry.file.flush().await
            .map_err(|e| e.to_string())
    }

    pub async fn metadata(&self, handle: u32) -> Result<FileMetadata, String> {
        let files = self.files.lock().unwrap();
        let entry = files.get(&handle)
            .ok_or_else(|| format!("File handle {} not found", handle))?;

        let metadata = entry.file.metadata().await
            .map_err(|e| e.to_string())?;

        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            permissions: metadata.permissions().mode(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            created: metadata.created().ok(),
        })
    }
} 