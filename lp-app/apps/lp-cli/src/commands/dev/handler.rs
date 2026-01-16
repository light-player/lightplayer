//! Dev command handler
//!
//! Orchestrates the dev command execution.

use anyhow::Result;

use super::args::DevArgs;
use super::async_client::AsyncLpClient;
use super::push::{
    load_project, load_project_async, push_project, push_project_async, validate_local_project,
};
use super::sync::sync_changes;
use super::watcher::FileWatcher;
use crate::debug_ui::DebugUiState;
use crate::messages;
use crate::server::{create_server, run_server_loop_async};
use crate::transport::HostSpecifier;
use crate::transport::WebSocketClientTransport;
use crate::transport::local::create_local_transport_pair;
use lp_client::LpClient;
use lp_engine_client::project::ClientProjectView;
use lp_model::TransportError;
use lp_shared::fs::LpFsStd;
use lp_shared::transport::ClientTransport;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

/// Handle the dev command
///
/// Validates local project, connects to server, and optionally pushes project files.
/// When no --push is specified, uses local in-memory server and automatically pushes.
/// Supports both in-memory server and remote servers (via --push HOST).
pub fn handle_dev(args: DevArgs) -> Result<()> {
    // Validate local project
    let (project_uid, project_name) = validate_local_project(&args.dir)?;

    // Determine if we should push and to which host
    match &args.push_host {
        Some(Some(host_str)) => {
            // Push to specified remote host
            let host_spec = HostSpecifier::parse(host_str)
                .map_err(|e| anyhow::anyhow!("Invalid host specifier '{}': {}", host_str, e))?;
            match host_spec {
                HostSpecifier::Local => handle_dev_local(args, project_uid, project_name, true),
                HostSpecifier::WebSocket { url } => {
                    handle_dev_websocket(args, project_uid, project_name, &url, true)
                }
                HostSpecifier::Serial { .. } => {
                    anyhow::bail!("Serial transport not yet implemented");
                }
            }
        }
        Some(None) => {
            // Push to local in-memory server
            handle_dev_local(args, project_uid, project_name, true)
        }
        None => {
            // No --push specified: use local in-memory server and automatically push
            handle_dev_local(args, project_uid, project_name, true)
        }
    }
}

/// Handle dev command with local in-memory server
fn handle_dev_local(
    args: DevArgs,
    project_uid: String,
    project_name: String,
    should_push: bool,
) -> Result<()> {
    // Create local transport pair
    let (client_transport, server_transport) = create_local_transport_pair();

    // Spawn server on separate thread with its own tokio runtime
    // Create server inside the thread since LpServer is not Send
    let _server_handle = std::thread::Builder::new()
        .name("lp-server".to_string())
        .spawn(move || {
            let runtime = Runtime::new().expect("Failed to create tokio runtime for server");
            runtime.block_on(async {
                // Create in-memory server (inside thread since LpServer is not Send)
                let (server, _base_fs) =
                    create_server(None, true, None).expect("Failed to create server");

                // Create LocalSet for spawn_local (needed because LpServer is not Send)
                let local_set = tokio::task::LocalSet::new();
                let _ = local_set
                    .run_until(run_server_loop_async(server, server_transport))
                    .await;
            });
        })
        .expect("Failed to spawn server thread");

    // Create tokio runtime for client
    let runtime = Runtime::new()?;

    // Wrap transport in Arc<Mutex<>> for sharing between loader and UI
    let shared_transport: Arc<Mutex<Box<dyn ClientTransport + Send>>> =
        Arc::new(Mutex::new(Box::new(client_transport)));

    // Create local filesystem view of project directory (needed for both push and watch)
    let local_fs = LpFsStd::new(args.dir.clone());

    // Run async client code to load project
    let (handle, async_client_for_ui) = runtime.block_on(async {
        // Create async client with shared transport
        let mut async_client = AsyncLpClient::new(Arc::clone(&shared_transport));

        // Push project if requested
        if should_push {
            println!(
                "Pushing project '{}' (uid: {}) to server...",
                project_name, project_uid
            );

            // Push project files using async client
            push_project_async(&mut async_client, &local_fs, &project_uid)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to push project '{}': {}", project_name, e))?;

            println!("Project files pushed successfully");
        }

        // Load project on server
        println!("Loading project on server...");
        let handle = load_project_async(&mut async_client, &project_uid)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load project '{}': {}", project_name, e))?;

        messages::print_success(
            &format!(
                "Project '{}' (uid: {}) loaded successfully",
                project_name, project_uid
            ),
            &[
                &format!("Project handle: {:?}", handle),
                "Project is now running on the server",
                if args.headless {
                    "Press Ctrl+C to stop"
                } else {
                    "Debug UI will open shortly"
                },
            ],
        );

        // Return handle and client for UI (transport is shared via Arc)
        Ok::<(lp_model::project::handle::ProjectHandle, AsyncLpClient), anyhow::Error>((
            handle,
            async_client,
        ))
    })?;

    // Start file watcher for automatic sync
    // Note: We need to clone the local_fs for the watcher task, but LpFsStd is not Send.
    // For now, we'll create a new LpFsStd instance in the watcher task since it's just a path wrapper.
    let project_dir = args.dir.clone();
    let project_uid_clone = project_uid.clone();
    let shared_transport_clone = Arc::clone(&shared_transport);

    // Spawn background task for file watching and syncing
    // Use spawn_blocking since FileWatcher uses blocking I/O
    let _watcher_handle = runtime.spawn(async move {
        // Create file watcher
        let mut watcher = match FileWatcher::new(project_dir.clone()) {
            Ok(w) => {
                println!("File watcher started for {}", project_dir.display());
                w
            }
            Err(e) => {
                eprintln!("Failed to start file watcher: {}", e);
                return;
            }
        };

        // Create async client for syncing (using shared transport)
        let mut sync_client = AsyncLpClient::new(shared_transport_clone);

        // Debouncing: collect changes and wait for inactivity before syncing
        let debounce_duration = Duration::from_millis(500); // Wait 500ms after last change
        let mut pending_changes: Vec<lp_shared::fs::fs_event::FsChange> = Vec::new();
        let mut last_change_time: Option<tokio::time::Instant> = None;

        // Watch loop: collect changes and sync them with debouncing
        loop {
            // Collect pending changes (non-blocking)
            match watcher.collect_changes() {
                Ok(new_changes) => {
                    if !new_changes.is_empty() {
                        // Add new changes to pending list (deduplicate by path)
                        for change in new_changes {
                            // Remove any existing change for the same path
                            pending_changes.retain(|c| c.path != change.path);
                            // Add the new change (most recent wins)
                            pending_changes.push(change);
                        }
                        // Update last change time
                        last_change_time = Some(tokio::time::Instant::now());
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {}", e);
                    // Watcher disconnected, exit loop
                    break;
                }
            }

            // Check if we should sync (debounce period has passed since last change)
            if let Some(last_time) = last_change_time {
                if last_time.elapsed() >= debounce_duration && !pending_changes.is_empty() {
                    // Time to sync - take ownership of pending changes
                    let changes_to_sync = std::mem::take(&mut pending_changes);
                    last_change_time = None;

                    // Sync changes to server (sync_changes creates its own LpFsStd instance)
                    if let Err(e) = sync_changes(
                        &mut sync_client,
                        changes_to_sync,
                        &project_uid_clone,
                        &project_dir,
                    )
                    .await
                    {
                        eprintln!("Error syncing file changes: {}", e);
                    }
                }
            }

            // Sleep briefly to avoid busy-waiting
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    // If not headless, spawn UI
    if !args.headless {
        // Create ClientProjectView (use std::sync::Mutex for sync UI context)
        let project_view = Arc::new(std::sync::Mutex::new(ClientProjectView::new()));

        // Get runtime handle for UI
        let runtime_handle = runtime.handle().clone();

        // Create UI state (transport is shared via Arc, so this works correctly)
        let ui_state = DebugUiState::new(project_view, handle, async_client_for_ui, runtime_handle);

        // Run UI (blocks until window closes)
        // This runs outside the async context
        let native_options = eframe::NativeOptions {
            // On macOS, eframe uses WGPU (Metal) by default, but the underlying
            // windowing library may still probe OpenGL first, causing harmless warnings.
            // These warnings can be safely ignored - the app will use Metal for rendering.
            ..Default::default()
        };
        
        eframe::run_native(
            "LP Debug UI",
            native_options,
            Box::new(|_cc| Box::new(ui_state)),
        )
        .map_err(|e| anyhow::anyhow!("UI error: {}", e))?;

        // UI closed - drop the shared transport to signal server shutdown
        drop(shared_transport);
    } else {
        // Enter client loop with Ctrl+C handling
        let result: Result<()> = runtime.block_on(async {
            // Recreate async client for headless mode
            // Note: This won't work because client_transport was moved
            // We need to restructure this
            // For now, just sleep
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("\nShutting down...");
                }
                _ = tokio::time::sleep(Duration::from_secs(3600)) => {
                    // Sleep for 1 hour (effectively forever)
                }
            }
            Ok(())
        });
        result?;

        // Headless mode closed - drop the shared transport to signal server shutdown
        drop(shared_transport);
    }

    // Don't wait for server thread - it will exit when transport closes or process exits
    // The server is a background thread and will be cleaned up automatically

    Ok(())
}

/// Run the client loop (async version)
///
/// Continuously polls the transport for incoming messages and processes them
/// via the async client. Runs until an error occurs or the transport is closed.
async fn run_client_loop_async(_client: &mut AsyncLpClient) -> Result<()> {
    // For now, just sleep indefinitely since the async client handles message processing
    // internally when making requests. In the future, we might want to add a background
    // task to process incoming messages, but for now the client only processes messages
    // when making requests.
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Run the client loop (sync version for websocket)
///
/// Continuously polls the transport for incoming messages and processes them
/// via the client. Runs until an error occurs or the transport is closed.
async fn run_client_loop(client: &mut LpClient, transport: &mut dyn ClientTransport) -> Result<()> {
    loop {
        // Collect incoming messages
        let mut incoming_messages = Vec::new();

        // Poll transport for messages (non-blocking)
        loop {
            match transport.receive() {
                Ok(Some(server_msg)) => {
                    // Wrap in Message envelope for client.tick()
                    incoming_messages.push(lp_model::Message::Server(server_msg));
                }
                Ok(None) => {
                    // No more messages available
                    break;
                }
                Err(e) => {
                    // Connection lost is expected during shutdown - exit gracefully
                    if matches!(e, TransportError::ConnectionLost) {
                        return Ok(());
                    }
                    // Other transport errors are unexpected - log and return
                    eprintln!("Transport error: {}", e);
                    return Err(anyhow::anyhow!("Transport error: {}", e));
                }
            }
        }

        // Process messages if any
        if !incoming_messages.is_empty() {
            if let Err(e) = client.tick(incoming_messages) {
                eprintln!("Client error: {}", e);
                return Err(anyhow::anyhow!("Client error: {}", e));
            }
        }

        // Small sleep to avoid busy-waiting
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Handle dev command with WebSocket server
fn handle_dev_websocket(
    args: DevArgs,
    project_uid: String,
    project_name: String,
    url: &str,
    should_push: bool,
) -> Result<()> {
    // Create WebSocket transport
    let mut transport: Box<dyn ClientTransport> = Box::new(
        WebSocketClientTransport::new(url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", url, e))?,
    );

    // Create client
    let mut client = LpClient::new();

    // Push project if requested
    if should_push {
        // Create local filesystem view of project directory
        let local_fs = LpFsStd::new(args.dir.clone());

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
            "Press Ctrl+C to stop",
        ],
    );

    // Create tokio runtime for async operations
    let runtime = Runtime::new()?;

    // Run async code with Ctrl+C handling
    runtime.block_on(async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down...");
            }
            result = run_client_loop(&mut client, transport.as_mut()) => {
                result?;
            }
        }

        Ok(())
    })
}
