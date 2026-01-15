//! In-memory transport for testing
//!
//! Provides a simple in-memory transport that serializes/deserializes messages
//! to/from JSON to ensure the message protocol works correctly.
//!
//! Uses Arc<Mutex> for thread-safety to allow concurrent server processing in tests.

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

use lp_shared::transport::{ClientTransport, Message as TransportMessage, ServerTransport, TransportError};

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "std")]
use std::collections::VecDeque;
#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::string::ToString;

#[cfg(not(feature = "std"))]
use alloc::{collections::VecDeque, rc::Rc, vec::Vec, string::ToString};
#[cfg(not(feature = "std"))]
use core::cell::RefCell;

/// Shared state for bidirectional communication
struct SharedState {
    /// Queue of messages from client to server
    client_to_server: VecDeque<Vec<u8>>,
    /// Queue of messages from server to client
    server_to_client: VecDeque<Vec<u8>>,
}

/// In-memory transport implementation
///
/// Uses shared queues for bidirectional communication. Messages are serialized
/// to/from JSON to ensure the message protocol serializes correctly.
///
/// Uses Arc<Mutex> when std feature is enabled (for thread-safety in tests),
/// otherwise uses Rc<RefCell> for no_std compatibility.
#[cfg(feature = "std")]
pub struct MemoryTransport {
    /// Shared state (wrapped in Mutex for thread-safety)
    state: Arc<Mutex<SharedState>>,
    /// Whether this is the client side (true) or server side (false)
    is_client: bool,
}

#[cfg(not(feature = "std"))]
pub struct MemoryTransport {
    /// Shared state (wrapped in RefCell for interior mutability)
    state: Rc<RefCell<SharedState>>,
    /// Whether this is the client side (true) or server side (false)
    is_client: bool,
}

impl MemoryTransport {
    /// Create a pair of transports (client and server)
    ///
    /// Returns `(client_transport, server_transport)` that can communicate
    /// with each other through in-memory queues.
    #[cfg(feature = "std")]
    pub fn new_pair() -> (Self, Self) {
        let state = Arc::new(Mutex::new(SharedState {
            client_to_server: VecDeque::new(),
            server_to_client: VecDeque::new(),
        }));

        let client = MemoryTransport {
            state: Arc::clone(&state),
            is_client: true,
        };

        let server = MemoryTransport {
            state,
            is_client: false,
        };

        (client, server)
    }

    #[cfg(not(feature = "std"))]
    pub fn new_pair() -> (Self, Self) {
        let state = Rc::new(RefCell::new(SharedState {
            client_to_server: VecDeque::new(),
            server_to_client: VecDeque::new(),
        }));

        let client = MemoryTransport {
            state: Rc::clone(&state),
            is_client: true,
        };

        let server = MemoryTransport {
            state,
            is_client: false,
        };

        (client, server)
    }
}

impl ClientTransport for MemoryTransport {
    #[cfg(feature = "std")]
    fn send(&mut self, msg: TransportMessage) -> Result<(), TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.lock().unwrap();
        state.client_to_server.push_back(msg.payload);
        Ok(())
    }

    #[cfg(not(feature = "std"))]
    fn send(&mut self, msg: TransportMessage) -> Result<(), TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        state.client_to_server.push_back(msg.payload);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn receive(&mut self) -> Result<Option<TransportMessage>, TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.lock().unwrap();
        Ok(state.server_to_client.pop_front().map(|payload| TransportMessage { payload }))
    }

    #[cfg(not(feature = "std"))]
    fn receive(&mut self) -> Result<Option<TransportMessage>, TransportError> {
        if !self.is_client {
            return Err(TransportError::Other(
                "Cannot use server transport as client transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        Ok(state.server_to_client.pop_front().map(|payload| TransportMessage { payload }))
    }
}

impl ServerTransport for MemoryTransport {
    #[cfg(feature = "std")]
    fn send(&mut self, msg: TransportMessage) -> Result<(), TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.lock().unwrap();
        state.server_to_client.push_back(msg.payload);
        Ok(())
    }

    #[cfg(not(feature = "std"))]
    fn send(&mut self, msg: TransportMessage) -> Result<(), TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        state.server_to_client.push_back(msg.payload);
        Ok(())
    }

    #[cfg(feature = "std")]
    fn receive(&mut self) -> Result<Option<TransportMessage>, TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.lock().unwrap();
        Ok(state.client_to_server.pop_front().map(|payload| TransportMessage { payload }))
    }

    #[cfg(not(feature = "std"))]
    fn receive(&mut self) -> Result<Option<TransportMessage>, TransportError> {
        if self.is_client {
            return Err(TransportError::Other(
                "Cannot use client transport as server transport".to_string(),
            ));
        }

        let mut state = self.state.borrow_mut();
        Ok(state.client_to_server.pop_front().map(|payload| TransportMessage { payload }))
    }
}
