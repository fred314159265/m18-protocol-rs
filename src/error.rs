use thiserror::Error;

pub type Result<T> = std::result::Result<T, M18Error>;

#[derive(Error, Debug)]
pub enum M18Error {
    #[error("Serial port error: {0}")]
    SerialPort(#[from] serialport::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Communication timeout")]
    Timeout,
    
    #[error("Invalid response: expected {expected}, got {actual}")]
    InvalidResponse { expected: String, actual: String },
    
    #[error("Empty response")]
    EmptyResponse,
    
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    
    #[error("Invalid data type: {0}")]
    InvalidDataType(String),
    
    #[error("Register not found: {address:#06x}")]
    RegisterNotFound { address: u16 },
    
    #[error("Message too long: {length} bytes (max 20)")]
    MessageTooLong { length: usize },
    
    #[cfg(feature = "form-submission")]
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
}