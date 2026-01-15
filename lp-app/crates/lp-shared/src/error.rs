//! Error types for lp-shared

use alloc::string::String;
use core::fmt;

/// Filesystem error type
#[derive(Debug, Clone)]
pub enum FsError {
    /// Filesystem operation failed
    Filesystem(String),
    /// File not found
    NotFound(String),
    /// Invalid path
    InvalidPath(String),
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::Filesystem(msg) => write!(f, "Filesystem error: {}", msg),
            FsError::NotFound(msg) => write!(f, "File not found: {}", msg),
            FsError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
        }
    }
}

/// Texture error type
#[derive(Debug, Clone)]
pub enum TextureError {
    /// Invalid texture format
    InvalidFormat(String),
    /// Texture dimensions too large
    DimensionsTooLarge { width: u32, height: u32 },
}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextureError::InvalidFormat(format) => {
                write!(f, "Invalid texture format: {}", format)
            }
            TextureError::DimensionsTooLarge { width, height } => {
                write!(f, "Texture dimensions too large: {}x{}", width, height)
            }
        }
    }
}

/// Transport error type
#[derive(Debug, Clone)]
pub enum TransportError {
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Connection lost
    ConnectionLost,
    /// Other transport error
    Other(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            TransportError::Deserialization(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            TransportError::ConnectionLost => write!(f, "Connection lost"),
            TransportError::Other(msg) => write!(f, "Transport error: {}", msg),
        }
    }
}

/// Output provider error type
#[derive(Debug, Clone)]
pub enum OutputError {
    /// Pin is already open
    PinAlreadyOpen { pin: u32 },
    /// Invalid handle
    InvalidHandle { handle: i32 },
    /// Invalid configuration
    InvalidConfig { reason: String },
    /// Data length mismatch
    DataLengthMismatch { expected: u32, actual: usize },
    /// Other error
    Other { message: String },
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputError::PinAlreadyOpen { pin } => {
                write!(f, "Pin {} is already open", pin)
            }
            OutputError::InvalidHandle { handle } => {
                write!(f, "Invalid handle: {}", handle)
            }
            OutputError::InvalidConfig { reason } => {
                write!(f, "Invalid config: {}", reason)
            }
            OutputError::DataLengthMismatch { expected, actual } => {
                write!(
                    f,
                    "Data length {} doesn't match expected byte_count {}",
                    actual, expected
                )
            }
            OutputError::Other { message } => {
                write!(f, "Error: {}", message)
            }
        }
    }
}
