//! Error types for lp-client

extern crate alloc;

use alloc::string::String;
use core::fmt;
use lp_shared::TransportError;

/// Error type for lp-client operations
#[derive(Debug, Clone)]
pub enum ClientError {
    /// Transport error
    Transport(TransportError),
    /// Request timed out
    Timeout { request_id: u64 },
    /// Protocol error (e.g., unexpected response, error in response)
    Protocol { message: String },
    /// Other error
    Other { message: String },
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Transport(err) => write!(f, "Transport error: {}", err),
            ClientError::Timeout { request_id } => {
                write!(f, "Request {} timed out", request_id)
            }
            ClientError::Protocol { message } => write!(f, "Protocol error: {}", message),
            ClientError::Other { message } => write!(f, "Error: {}", message),
        }
    }
}

impl From<TransportError> for ClientError {
    fn from(err: TransportError) -> Self {
        ClientError::Transport(err)
    }
}
