//! Serve command handler
//!
//! Orchestrates the serve command execution.

use anyhow::Result;
use std::path::PathBuf;

use super::args::ServeArgs;
use super::init::{create_filesystem, initialize_server};
use super::server_loop::run_server_loop;
use lp_server::LpServer;
use lp_shared::output::MemoryOutputProvider;
use std::cell::RefCell;
use std::rc::Rc;

use crate::transport::WebSocketServerTransport;

/// Handle the serve command
///
/// Initializes server, creates filesystem, starts LpServer, and runs the main loop.
pub fn handle_serve(args: ServeArgs) -> Result<()> {
    // Determine server directory (default to current directory)
    let server_dir = args.dir.unwrap_or_else(|| PathBuf::from("."));

    // Initialize server (check/create server.json)
    initialize_server(&server_dir, args.init)?;

    // Create filesystem
    let base_fs = create_filesystem(Some(&server_dir), args.memory)?;

    // Create output provider
    let output_provider = Rc::new(RefCell::new(MemoryOutputProvider::new()));

    // Create LpServer
    let server = LpServer::new(output_provider, base_fs, "projects/".to_string());

    // Create websocket server transport on port 2812
    let transport = WebSocketServerTransport::new(2812)
        .map_err(|e| anyhow::anyhow!("Failed to start websocket server: {}", e))?;

    println!("Server started on ws://localhost:2812/");
    println!("Press Ctrl+C to stop");

    // Run server loop (blocks until interrupted)
    run_server_loop(server, transport)?;

    Ok(())
}
