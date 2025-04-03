pub mod common;

#[cfg(feature = "linux")]
mod linux;
#[cfg(feature = "sev")]
mod sev;

#[cfg(feature = "linux")]
pub use linux::FileContext;
#[cfg(feature = "sev")]
pub use sev::FileContext;

pub use common::{FileConfig, FileError, FileMetadata, FileMode, FileOperations};