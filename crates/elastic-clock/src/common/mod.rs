use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Resource not found")]
    NotFound,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub trait ClockOperations {
    fn create_clock(&self, config: &ClockConfig) -> Result<u32, CommonError>;
    fn destroy_clock(&self, handle: u32) -> Result<(), CommonError>;
    fn get_time(&self, handle: u32) -> Result<u64, CommonError>;
    fn get_resolution(&self, handle: u32) -> Result<u64, CommonError>;
    async fn sleep(&self, handle: u32, duration: u64) -> Result<(), CommonError>;
    fn get_elapsed(&self, handle: u32) -> Result<u64, CommonError>;
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