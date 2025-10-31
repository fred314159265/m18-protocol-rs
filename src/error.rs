//! Error types for M18 protocol operations.

use thiserror::Error;

/// Result type alias for M18 operations.
pub type Result<T> = std::result::Result<T, M18Error>;

/// Error types for M18 battery communication.
#[derive(Error, Debug)]
pub enum M18Error {
    /// Serial port communication error
    #[error("Serial port error: {0}")]
    SerialPort(#[from] serialport::Error),

    /// General I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Communication timeout (no response from battery)
    #[error("Communication timeout")]
    Timeout,

    /// Response didn't match expected format
    #[error("Invalid response: expected {expected}, got {actual}")]
    InvalidResponse {
        /// Expected response format
        expected: String,
        /// Actual response received
        actual: String,
    },

    /// Battery returned empty response
    #[error("Empty response")]
    EmptyResponse,

    /// Response checksum validation failed
    #[error("Checksum mismatch")]
    ChecksumMismatch,

    /// Unknown data type string
    #[error("Invalid data type: {0}")]
    InvalidDataType(String),

    /// Requested register address not found
    #[error("Register not found: {address:#06x}")]
    RegisterNotFound {
        /// Register address that was not found
        address: u16,
    },

    /// Message exceeds maximum length for battery memory
    #[error("Message too long: {length} bytes (max 20)")]
    MessageTooLong {
        /// Length of message that was too long
        length: usize,
    },

    /// Data parsing error
    #[error("Parse error: {0}")]
    Parse(String),
}
