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