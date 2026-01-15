#![no_std]

extern crate alloc;

pub mod client;
pub mod error;

pub use client::LpClient;
pub use error::ClientError;
// Re-export LocalMemoryTransport from lp-shared for convenience
pub use lp_shared::LocalTransport;
