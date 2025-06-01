pub mod common;

#[cfg(feature = "linux")]
mod linux;
#[cfg(feature = "sev")]
mod sev;
#[cfg(feature = "wasi")]
mod wasm;

#[cfg(all(feature = "linux", not(any(feature = "sev", feature = "wasi"))))]
pub use linux::FileContext;
#[cfg(all(feature = "sev", not(feature = "wasi")))]
pub use sev::SevFileContext as FileContext;
#[cfg(feature = "wasi")]
pub use wasm::WasmFileContext as FileContext;

pub use common::{FileConfig, FileError, FileMetadata, FileMode, FileOperations};

pub trait FileOps {
    fn open(&self, path: &str) -> Result<u32, FileError>;
    fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, FileError>;
    fn write(&self, handle: u32, data: &[u8]) -> Result<usize, FileError>;
    fn seek(&self, handle: u32, pos: u64) -> Result<u64, FileError>;
    fn metadata(&self, handle: u32) -> Result<(u64, bool, bool), FileError>;
    fn close(&self, handle: u32) -> Result<(), FileError>;
}