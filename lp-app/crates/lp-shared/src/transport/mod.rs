//! Transport traits for client-server communication
//!
//! This module defines traits for pluggable transport implementations.
//! Messages are consumed (moved) on send, and receive is non-blocking.

pub mod client;
pub mod server;

pub use client::ClientTransport;
pub use server::ServerTransport;
pub use crate::error::TransportError;

/// Message type placeholder
///
/// This is a placeholder type that will be replaced with `lp_model::Message`
/// when the message protocol is defined in Phase 4.
///
/// For now, this is a simple type that can be used for testing.
/// Implementations should serialize/deserialize messages as needed.
#[derive(Debug, Clone)]
pub struct Message {
    /// Message payload (will be replaced with proper message types)
    pub payload: alloc::vec::Vec<u8>,
}

impl Message {
    /// Create a new message with the given payload
    pub fn new(payload: alloc::vec::Vec<u8>) -> Self {
        Self { payload }
    }
}
