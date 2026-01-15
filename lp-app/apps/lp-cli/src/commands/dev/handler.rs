//! Dev command handler
//!
//! Orchestrates the dev command execution.

use anyhow::Result;
use std::path::PathBuf;

use super::args::DevArgs;
use super::push::{load_project, push_project, validate_local_project};
use crate::messages;
use crate::transport::HostSpecifier;
use crate::transport::WebSocketClientTransport;
use lp_client::LpClient;
use lp_shared::fs::LpFsStd;
use lp_shared::transport::ClientTransport;

/// Handle the dev command
///
/// Validates local project, connects to server, and pushes project files.
pub fn handle_dev(args: DevArgs) -> Result<()> {
    // Determine project directory (default to current directory)
    let project_dir = args.dir.unwrap_or_else(|| PathBuf::from("."));

    // Validate local project
    let (project_uid, project_name) = validate_local_project(&project_dir)?;

    // Parse host specifier
    let host_spec = HostSpecifier::parse(&args.host)
        .map_err(|e| anyhow::anyhow!("Invalid host specifier '{}': {}", args.host, e))?;

    // Create transport based on host specifier
    // For now, only websocket is supported
    let mut transport: Box<dyn ClientTransport> = match host_spec {
        HostSpecifier::WebSocket { url } => Box::new(
            WebSocketClientTransport::new(&url)
                .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", url, e))?,
        ),
        HostSpecifier::Serial { .. } => {
            anyhow::bail!("Serial transport not yet implemented");
        }
        HostSpecifier::Local => {
            anyhow::bail!("Local transport not yet implemented");
        }
    };

    // Create client
    let mut client = LpClient::new();

    // Create local filesystem view of project directory
    let local_fs = LpFsStd::new(project_dir.clone());

    // Push project if requested (default: true)
    if args.push {
        println!(
            "Pushing project '{}' (uid: {}) to server...",
            project_name, project_uid
        );

        push_project(&mut client, transport.as_mut(), &local_fs, &project_uid)
            .map_err(|e| anyhow::anyhow!("Failed to push project '{}': {}", project_name, e))?;

        println!("Project files pushed successfully");
    }

    // Load project on server
    println!("Loading project on server...");
    let handle = load_project(&mut client, transport.as_mut(), &project_uid)
        .map_err(|e| anyhow::anyhow!("Failed to load project '{}': {}", project_name, e))?;

    messages::print_success(
        &format!(
            "Project '{}' (uid: {}) loaded successfully",
            project_name, project_uid
        ),
        &[
            &format!("Project handle: {:?}", handle),
            "Project is now running on the server",
        ],
    );

    Ok(())
}
