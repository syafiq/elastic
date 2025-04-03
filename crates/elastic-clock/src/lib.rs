mod common;
pub use common::*;

#[cfg(feature = "linux")]
mod linux;

#[cfg(feature = "sev")]
mod sev;

#[cfg(feature = "linux")]
pub use linux::*;

#[cfg(feature = "sev")]
pub use sev::*;

pub mod clock;

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClockError {
    #[error("Invalid clock configuration")]
    InvalidConfig,
    #[error("Clock handle not found")]
    HandleNotFound,
    #[error("Clock operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockType {
    System,
    Monotonic,
    Process,
    Thread,
}

#[derive(Debug, Clone)]
pub struct ClockConfig {
    pub clock_type: ClockType,
    pub high_resolution: bool,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            clock_type: ClockType::System,
            high_resolution: false,
        }
    }
}

#[derive(Clone)]
pub struct ClockContext {
    manager: Arc<clock::ClockManager>,
}

impl ClockContext {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(clock::ClockManager::new()),
        }
    }

    pub fn create_clock(&self, config: &ClockConfig) -> Result<u32, ClockError> {
        self.manager.create_clock(config)
            .map_err(|e| ClockError::OperationFailed(e))
    }

    pub fn destroy_clock(&self, handle: u32) -> Result<(), ClockError> {
        self.manager.destroy_clock(handle)
            .map_err(|e| ClockError::OperationFailed(e))
    }

    pub fn get_time(&self, handle: u32) -> Result<u64, ClockError> {
        self.manager.get_time(handle)
            .map_err(|e| ClockError::OperationFailed(e))
    }

    pub fn get_resolution(&self, handle: u32) -> Result<u64, ClockError> {
        self.manager.get_resolution(handle)
            .map_err(|e| ClockError::OperationFailed(e))
    }

    pub async fn sleep(&self, handle: u32, duration: u64) -> Result<(), ClockError> {
        self.manager.sleep(handle, duration).await
            .map_err(|e| ClockError::OperationFailed(e))
    }

    pub fn get_elapsed(&self, handle: u32) -> Result<u64, ClockError> {
        self.manager.get_elapsed(handle)
            .map_err(|e| ClockError::OperationFailed(e))
    }
} 