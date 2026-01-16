//! Local in-memory transport for testing
//!
//! Provides a simple single-threaded in-memory transport that serializes/deserializes messages
//! to/from JSON to ensure the message protocol works correctly.
//!
//! Uses Rc<RefCell> for single-threaded interior mutability. This transport is explicitly
//! single-threaded and does not require std.

extern crate alloc;

use crate::transport::{ClientTransport, ServerTransport};
use alloc::{collections::VecDeque, format, rc::Rc, string::ToString, vec::Vec};
use core::cell::RefCell;
use lp_model::{ClientMessage, ServerMessage, TransportError};
use serde_json;

/// Shared state for bidirectional communication
struct SharedState {
    /// Queue of serialized client messages (client -> server)
    client_to_server: VecDeque<Vec<u8>>,
    /// Queue of serialized server messages (server -> client)
    server_to_client: VecDeque<Vec<u8>>,
    /// Whether the transport is closed
    closed: bool,
}

/// Local in-memory transport implementation
///
/// Uses shared queues for bidirectional communication. Messages are serialized
/// to/from JSON to ensure the message protocol serializes correctly.
///
/// This transport is explicitly single-threaded and uses Rc<RefCell> for
/// interior mutability. It does not require std.
pub struct LocalTransport {
    /// Shared state (wrapped in RefCell for interior mutability)
    state: Rc<RefCell<SharedState>>,
    /// Whether this is the client side (true) or server side (false)
    is_client: bool,
}

impl LocalTransport {
    /// Create a pair of transports (client and server)
    ///
    /// Returns `(client_transport, server_transport)` that can communicate
    /// with each other through in-memory queues.
    pub fn new_pair() -> (Self, Self) {
        let state = Rc::new(RefCell::new(SharedState {
            client_to_server: VecDeque::new(),
            server_to_client: VecDeque::new(),
            closed: false,
        }));

        let client = LocalTransport {
            state: Rc::clone(&state),
            is_client: true,
        };

        let server = LocalTransport {
            state,
            is_client: false,
        };

        (client, server)
    }
}

impl ClientTransport for LocalTransport {
    fn send(&mut self, msg: ClientMessage) -> Result<(), TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        // Serialize the message
        let payload = serde_json::to_vec(&msg).map_err(|e| {
            TransportError::Serialization(format!("Failed to serialize ClientMessage: {}", e))
        })?;

        let mut state = self.state.borrow_mut();
        state.client_to_server.push_back(payload);
        Ok(())
    }

    fn receive(&mut self) -> Result<Option<ServerMessage>, TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(TransportError::ConnectionLost);
        }

        Ok(state
            .server_to_client
            .pop_front()
            .map(|payload| {
                serde_json::from_slice(&payload).map_err(|e| {
                    TransportError::Deserialization(format!(
                        "Failed to deserialize ServerMessage: {}",
                        e
                    ))
                })
            })
            .transpose()?)
    }

    fn receive_all(&mut self) -> Result<Vec<ServerMessage>, TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(TransportError::ConnectionLost);
        }

        let mut messages = Vec::new();
        while let Some(payload) = state.server_to_client.pop_front() {
            let msg = serde_json::from_slice(&payload).map_err(|e| {
                TransportError::Deserialization(format!(
                    "Failed to deserialize ServerMessage: {}",
                    e
                ))
            })?;
            messages.push(msg);
        }
        Ok(messages)
    }

    fn close(&mut self) -> Result<(), TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        state.closed = true;
        Ok(())
    }
}

impl Drop for LocalTransport {
    fn drop(&mut self) {
        // Mark as closed (best-effort, ignore errors)
        // We can't call close() because LocalTransport implements both ClientTransport and ServerTransport
        // so we just mark the shared state as closed directly
        let mut state = self.state.borrow_mut();
        state.closed = true;
    }
}

impl ServerTransport for LocalTransport {
    fn send(&mut self, msg: ServerMessage) -> Result<(), TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        // Serialize the message
        let payload = serde_json::to_vec(&msg).map_err(|e| {
            TransportError::Serialization(format!("Failed to serialize ServerMessage: {}", e))
        })?;

        let mut state = self.state.borrow_mut();
        state.server_to_client.push_back(payload);
        Ok(())
    }

    fn receive(&mut self) -> Result<Option<ClientMessage>, TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(TransportError::ConnectionLost);
        }

        Ok(state
            .client_to_server
            .pop_front()
            .map(|payload| {
                serde_json::from_slice(&payload).map_err(|e| {
                    TransportError::Deserialization(format!(
                        "Failed to deserialize ClientMessage: {}",
                        e
                    ))
                })
            })
            .transpose()?)
    }

    fn receive_all(&mut self) -> Result<Vec<ClientMessage>, TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(TransportError::ConnectionLost);
        }

        let mut messages = Vec::new();
        while let Some(payload) = state.client_to_server.pop_front() {
            let msg = serde_json::from_slice(&payload).map_err(|e| {
                TransportError::Deserialization(format!(
                    "Failed to deserialize ClientMessage: {}",
                    e
                ))
            })?;
            messages.push(msg);
        }
        Ok(messages)
    }

    fn close(&mut self) -> Result<(), TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        state.closed = true;
        Ok(())
    }
}
