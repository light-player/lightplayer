//! Client-side transport trait
//!
//! Defines the interface for client-side transport implementations.
//! Messages are consumed (moved) on send, and receive is non-blocking.
//!
//! The transport handles serialization/deserialization internally.

extern crate alloc;

use alloc::vec::Vec;
use lp_model::{ClientMessage, ServerMessage, TransportError};

/// Trait for client-side transport implementations
///
/// This trait provides a simple polling-based interface for sending and receiving
/// messages. Messages are consumed (moved) on send, and receive is non-blocking
/// (returns `None` if no message is available).
///
/// The transport handles serialization/deserialization internally.
///
/// # Examples
///
/// ```rust,no_run
/// use lp_shared::transport::ClientTransport;
/// use lp_model::{ClientMessage, ServerMessage, TransportError};
///
/// struct MyTransport;
///
/// impl ClientTransport for MyTransport {
///     fn send(&mut self, msg: ClientMessage) -> Result<(), TransportError> {
///         // Send message (transport handles serialization)
///         Ok(())
///     }
///
///     fn receive(&mut self) -> Result<Option<ServerMessage>, TransportError> {
///         // Receive message (transport handles deserialization)
///         Ok(None)
///     }
/// }
/// ```
pub trait ClientTransport {
    /// Send a client message (consumes the message)
    ///
    /// The transport handles serialization internally.
    ///
    /// # Arguments
    ///
    /// * `msg` - The client message to send (consumed/moved)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message was sent successfully
    /// * `Err(TransportError)` if sending failed
    fn send(&mut self, msg: ClientMessage) -> Result<(), TransportError>;

    /// Receive a server message (non-blocking)
    ///
    /// The transport handles deserialization internally.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ServerMessage))` if a message is available
    /// * `Ok(None)` if no message is available (non-blocking)
    /// * `Err(TransportError)` if receiving failed
    fn receive(&mut self) -> Result<Option<ServerMessage>, TransportError>;

    /// Receive all available server messages (non-blocking)
    ///
    /// Drains all available messages from the transport in a single call.
    /// This is more efficient than calling `receive()` in a loop.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ServerMessage>)` - Vector of all available messages (may be empty)
    /// * `Err(TransportError)` if receiving failed
    fn receive_all(&mut self) -> Result<Vec<ServerMessage>, TransportError> {
        let mut messages = Vec::new();
        loop {
            match self.receive()? {
                Some(msg) => messages.push(msg),
                None => break,
            }
        }
        Ok(messages)
    }

    /// Close the transport connection
    ///
    /// Explicitly closes the transport connection. This method is idempotent -
    /// calling it multiple times is safe and will return `Ok(())` if already closed.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the transport was closed successfully (or already closed)
    /// * `Err(TransportError)` if closing failed
    fn close(&mut self) -> Result<(), TransportError>;
}
