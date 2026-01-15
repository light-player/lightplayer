//! Project push logic
//!
//! Functions for validating local projects and pushing them to the server.

use anyhow::{Context, Result};
use std::path::Path;

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
pub fn push_project(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
    local_fs: &dyn LpFs,
    project_uid: &str,
) -> Result<()> {
    // List all files recursively from project root
    let entries = local_fs
        .list_dir("/", true)
        .map_err(|e| anyhow::anyhow!("Failed to list local project files: {}", e))?;

    // Write each file to server (path: /projects/{uid}/...)
    for entry in entries {
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

        // Write file to server
        let (write_msg, write_id) = client.fs_write(server_path.clone(), content);
        send_and_process(client, transport, write_msg)
            .with_context(|| format!("Failed to send write request for {}", server_path))?;

        // Get and check response
        let response = client.get_response(write_id).ok_or_else(|| {
            anyhow::anyhow!("No response received for write request {}", write_id)
        })?;

        client
            .extract_write_response(write_id, response)
            .map_err(|e| anyhow::anyhow!("Server error writing {}: {}", server_path, e))?;
    }

    Ok(())
}

/// Load project on server
///
/// Sends LoadProject request and waits for response.
/// Returns the project handle.
pub fn load_project(
    client: &mut LpClient,
    transport: &mut dyn ClientTransport,
    project_uid: &str,
) -> Result<lp_model::project::handle::ProjectHandle> {
    let project_path = format!("/projects/{}/project.json", project_uid);

    // Send load request
    let (load_msg, load_id) = client.project_load(project_path.clone());
    send_and_process(client, transport, load_msg)
        .with_context(|| format!("Failed to send load project request for {}", project_path))?;

    // Get and extract response
    let response = client
        .get_response(load_id)
        .ok_or_else(|| anyhow::anyhow!("No response received for load request {}", load_id))?;

    client
        .extract_load_project_response(load_id, response)
        .map_err(|e| anyhow::anyhow!("Server error loading project {}: {}", project_path, e))
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
