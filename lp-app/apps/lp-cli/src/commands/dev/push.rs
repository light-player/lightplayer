//! Project push logic
//!
//! Functions for validating local projects and pushing them to the server.

use anyhow::{Context, Result};
use std::path::Path;

use crate::commands::dev::async_client::AsyncLpClient;
use crate::messages::{format_command, print_error_and_return};
use lp_client::LpClient;
use lp_model::Message;
use lp_shared::fs::LpFs;
use lp_shared::transport::ClientTransport;

/// Validate local project directory
///
/// Checks for project.json and extracts uid and name.
/// Returns (uid, name) if valid, or an error with helpful message.
pub fn validate_local_project(dir: &Path) -> Result<(String, String)> {
    let project_json_path = dir.join("project.json");

    if !project_json_path.exists() {
        let cmd = format_command(&format!("lp-cli create {}", dir.display()));
        return Err(print_error_and_return(
            &format!("No project.json found in {}", dir.display()),
            &[&format!("To create a new project, run: {}", cmd)],
        ));
    }

    let project_json = std::fs::read_to_string(&project_json_path)
        .with_context(|| format!("Failed to read project.json from {}", dir.display()))?;

    let config: lp_model::project::config::ProjectConfig = serde_json::from_str(&project_json)
        .with_context(|| format!("Failed to parse project.json from {}", dir.display()))?;

    Ok((config.uid, config.name))
}

/// Process messages between client and server
///
/// Receives messages from server, processes them on client, and sends responses back.
/// This is a helper function for synchronous message processing.
fn process_messages(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
) -> Result<(), anyhow::Error> {
    // Process all available messages from server
    loop {
        match transport.receive() {
            Ok(Some(server_msg)) => {
                // Wrap in Message envelope for client.tick()
                let message = Message::Server(server_msg);

                // Process on client (matches responses to pending requests)
                client
                    .tick(vec![message])
                    .map_err(|e| anyhow::anyhow!("Client error: {}", e))?;
            }
            Ok(None) => {
                // No more messages available
                break;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Transport error: {}", e));
            }
        }
    }

    Ok(())
}

/// Send a client message and wait for response
///
/// Sends the message via transport, processes responses, and returns the request ID.
fn send_and_process(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
    msg: Message,
) -> Result<u64, anyhow::Error> {
    // Extract ClientMessage from Message envelope
    let client_msg = match msg {
        Message::Client(msg) => msg,
        Message::Server(_) => {
            return Err(anyhow::anyhow!("Expected Client message"));
        }
    };

    // Send via transport
    transport
        .send(client_msg.clone())
        .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;

    // Process responses
    process_messages(client, transport)?;

    Ok(client_msg.id)
}

/// Push project files to server
///
/// Recursively reads all files from local project and writes them to server.
/// Accepts `LpClient` and filesystem as parameters for testability.
///
/// # Arguments
///
/// * `client` - The LpClient instance
/// * `transport` - The client transport
/// * `local_fs` - Local filesystem (project root)
/// * `project_uid` - Project UID (used for server path)
/// Send all project files to server (first phase of push)
///
/// Sends write requests for all files but doesn't wait for responses.
/// Returns a list of (request_id, server_path) tuples for verification.
pub fn send_push_requests(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
    local_fs: &dyn LpFs,
    project_uid: &str,
) -> Result<Vec<(u64, String)>> {
    // List all files recursively from project root
    let entries = local_fs
        .list_dir("/", true)
        .map_err(|e| anyhow::anyhow!("Failed to list local project files: {}", e))?;

    let mut write_requests = Vec::new();

    // Send all write requests
    for entry in entries {
        // Skip directories (directories are created implicitly when files are written)
        if let Ok(true) = local_fs.is_dir(&entry) {
            continue;
        }

        // Read file from local filesystem
        let content = local_fs
            .read_file(&entry)
            .map_err(|e| anyhow::anyhow!("Failed to read local file {}: {}", entry, e))?;

        // Construct server path: /projects/{uid}/...
        let server_path = if entry.starts_with('/') {
            format!("/projects/{}{}", project_uid, entry)
        } else {
            format!("/projects/{}/{}", project_uid, entry)
        };

        // Create write request and send
        let (write_msg, write_id) = client.fs_write(server_path.clone(), content);

        // Extract ClientMessage and send
        let client_msg = match write_msg {
            Message::Client(msg) => msg,
            Message::Server(_) => {
                return Err(anyhow::anyhow!("Expected Client message"));
            }
        };
        transport.send(client_msg.clone()).map_err(|e| {
            anyhow::anyhow!("Failed to send write request for {}: {}", server_path, e)
        })?;

        write_requests.push((write_id, server_path));
    }

    Ok(write_requests)
}

/// Verify all write responses are available and valid (second phase of push)
pub fn verify_push_responses(
    client: &mut LpClient,
    write_requests: &[(u64, String)],
) -> Result<()> {
    for (write_id, server_path) in write_requests {
        let response = client.get_response(*write_id).ok_or_else(|| {
            anyhow::anyhow!(
                "No response received for write request {} (file: {})",
                write_id,
                server_path
            )
        })?;

        client
            .extract_write_response(*write_id, response)
            .map_err(|e| anyhow::anyhow!("Server error writing {}: {}", server_path, e))?;
    }

    Ok(())
}

pub fn push_project(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
    local_fs: &dyn LpFs,
    project_uid: &str,
) -> Result<()> {
    // Send all write requests
    let write_requests = send_push_requests(client, transport, local_fs, project_uid)?;

    // Process messages once (responses may not be immediately available with async servers)
    process_messages(client, transport)?;

    // Verify all responses
    verify_push_responses(client, &write_requests)?;

    Ok(())
}

/// Load project on server
///
/// Sends LoadProject request and waits for response.
/// Returns the project handle.
///
/// # Arguments
///
/// * `client` - The LpClient instance
/// * `transport` - The client transport
/// * `project_uid` - Project UID (the server will look for it in projects/{uid}/)
pub fn load_project(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
    project_uid: &str,
) -> Result<lp_model::project::handle::ProjectHandle> {
    // Send load request with just the project UID
    // The server will construct the path as projects/{uid}/
    let (load_msg, load_id) = client.project_load(project_uid.to_string());
    send_and_process(client, transport, load_msg)
        .with_context(|| format!("Failed to send load project request for {}", project_uid))?;

    // Get and extract response
    let response = client
        .get_response(load_id)
        .ok_or_else(|| anyhow::anyhow!("No response received for load request {}", load_id))?;

    client
        .extract_load_project_response(load_id, response)
        .map_err(|e| anyhow::anyhow!("Server error loading project {}: {}", project_uid, e))
}

/// Push project files to server (async version)
///
/// Recursively reads all files from local project and writes them to server using AsyncLpClient.
///
/// # Arguments
///
/// * `client` - The AsyncLpClient instance
/// * `local_fs` - Local filesystem (project root)
/// * `project_uid` - Project UID (used for server path)
pub async fn push_project_async(
    client: &mut AsyncLpClient,
    local_fs: &dyn LpFs,
    project_uid: &str,
) -> Result<()> {
    // List all files recursively from project root
    let entries = local_fs
        .list_dir("/", true)
        .map_err(|e| anyhow::anyhow!("Failed to list local project files: {}", e))?;

    // Write all files using async client
    for entry in entries {
        // Skip directories (directories are created implicitly when files are written)
        if let Ok(true) = local_fs.is_dir(&entry) {
            continue;
        }

        // Read file from local filesystem
        let content = local_fs
            .read_file(&entry)
            .map_err(|e| anyhow::anyhow!("Failed to read local file {}: {}", entry, e))?;

        // Construct server path: /projects/{uid}/...
        let server_path = if entry.starts_with('/') {
            format!("/projects/{}{}", project_uid, entry)
        } else {
            format!("/projects/{}/{}", project_uid, entry)
        };

        // Write file using async client (awaits response)
        client
            .fs_write(server_path.clone(), content)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", server_path, e))?;
    }

    Ok(())
}

/// Load project on server (async version)
///
/// Sends LoadProject request and waits for response using AsyncLpClient.
/// Returns the project handle.
///
/// # Arguments
///
/// * `client` - The AsyncLpClient instance
/// * `project_uid` - Project UID (the server will look for it in projects/{uid}/)
pub async fn load_project_async(
    client: &mut AsyncLpClient,
    project_uid: &str,
) -> Result<lp_model::project::handle::ProjectHandle> {
    // Load project using async client (awaits response)
    client
        .project_load(project_uid.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load project {}: {}", project_uid, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lp_model::project::config::ProjectConfig;
    use tempfile::TempDir;

    #[test]
    fn test_validate_local_project_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        // Create project.json
        let config = ProjectConfig {
            uid: "test-uid".to_string(),
            name: "Test Project".to_string(),
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(dir.join("project.json"), json).unwrap();

        // Should succeed
        let (uid, name) = validate_local_project(dir).unwrap();
        assert_eq!(uid, "test-uid");
        assert_eq!(name, "Test Project");
    }

    #[test]
    fn test_validate_local_project_missing() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        // Should fail with helpful error
        let result = validate_local_project(dir);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No project.json found"));
        // Note: The suggestions are printed separately via print_error(),
        // so they won't be in the error message string itself
    }

    #[test]
    fn test_validate_local_project_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        // Create invalid project.json
        std::fs::write(dir.join("project.json"), "invalid json").unwrap();

        // Should fail with parse error
        let result = validate_local_project(dir);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse project.json"));
    }
}
