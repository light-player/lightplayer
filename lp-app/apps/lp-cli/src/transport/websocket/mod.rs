//! WebSocket transport implementations
//!
//! Provides WebSocket-based transport for both client and server.

pub mod client;
pub mod server;

pub use client::WebSocketClientTransport;
pub use server::WebSocketServerTransport;
