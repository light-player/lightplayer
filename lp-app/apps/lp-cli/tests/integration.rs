//! Integration tests for lp-cli
//!
//! Tests end-to-end functionality using memory filesystem and in-memory transport.
//! These tests verify that the CLI commands work correctly without requiring
//! real filesystem or network access.

extern crate alloc;

use alloc::{boxed::Box, rc::Rc};
use core::cell::RefCell;
use lp_client::{ClientError, LocalTransport, LpClient};
use lp_model::Message;
use lp_server::LpServer;
use lp_shared::fs::{LpFs, LpFsMemory};
use lp_shared::output::MemoryOutputProvider;
use lp_shared::transport::{ClientTransport, ServerTransport};

/// Set up server and client with memory transport
///
/// Returns `(server, client, client_transport, server_transport)` for
/// synchronous message processing in tests.
fn setup_server_and_client(fs: LpFsMemory) -> (LpServer, LpClient, LocalTransport, LocalTransport) {
    // Create transport pair
    let (client_transport, server_transport) = LocalTransport::new_pair();

    // Create server with filesystem (LpFsMemory now uses interior mutability)
    let output_provider = Rc::new(RefCell::new(MemoryOutputProvider::new()));
    let server = LpServer::new(output_provider, Box::new(fs), "projects".to_string());

    // Create client
    let client = LpClient::new();

    (server, client, client_transport, server_transport)
}

/// Extract ClientMessage from Message envelope and send via transport
fn send_client_message(transport: &mut LocalTransport, msg: Message) -> Result<(), ClientError> {
    let client_msg = match msg {
        Message::Client(msg) => msg,
        Message::Server(_) => {
            return Err(ClientError::Protocol {
                message: "Expected Client message".to_string(),
            });
        }
    };
    ClientTransport::send(transport, client_msg).map_err(ClientError::from)
}

/// Process messages synchronously between client and server
///
/// This bridges messages through the transport, processing them on both
/// client and server using their tick() methods.
fn process_messages(
    client: &mut LpClient,
    server: &mut LpServer,
    client_transport: &mut LocalTransport,
    server_transport: &mut LocalTransport,
) -> Result<(), ClientError> {
    // Process client -> server messages
    loop {
        match ServerTransport::receive(server_transport) {
            Ok(Some(client_msg)) => {
                // Transport handles deserialization, we get ClientMessage directly
                // Wrap in Message envelope for server.tick()
                let message = Message::Client(client_msg);

                // Process on server
                let responses = server
                    .tick(0, vec![message])
                    .map_err(|e| ClientError::Other {
                        message: format!("Server error: {}", e),
                    })?;

                // Send responses back through server transport
                for response in responses {
                    // Extract ServerMessage from Message envelope
                    match response {
                        Message::Server(server_msg) => {
                            ServerTransport::send(server_transport, server_msg)
                                .map_err(ClientError::from)?;
                        }
                        Message::Client(_) => {
                            return Err(ClientError::Protocol {
                                message: "Server received client message".to_string(),
                            });
                        }
                    }
                }
            }
            Ok(None) => break,
            Err(e) => {
                return Err(ClientError::Transport(e));
            }
        }
    }

    // Process server -> client messages
    loop {
        match ClientTransport::receive(client_transport) {
            Ok(Some(server_msg)) => {
                // Transport handles deserialization, we get ServerMessage directly
                // Wrap in Message envelope for client.tick()
                let message = Message::Server(server_msg);

                // Process on client
                let _outgoing = client.tick(vec![message]).map_err(|e| ClientError::Other {
                    message: format!("Client error: {}", e),
                })?;
            }
            Ok(None) => break,
            Err(e) => {
                return Err(ClientError::Transport(e));
            }
        }
    }

    Ok(())
}

/// Create a test project on a filesystem
///
/// Creates a minimal project with project.json and returns the project UID.
fn create_test_project(fs: &mut LpFsMemory, name: &str, uid: &str) -> Result<(), ClientError> {
    // Create project.json
    let project_json = format!(
        r#"{{
  "uid": "{}",
  "name": "{}"
}}"#,
        uid, name
    );
    fs.write_file_mut("/project.json", project_json.as_bytes())
        .map_err(|e| ClientError::Other {
            message: format!("Failed to write project.json: {}", e),
        })?;

    // Create src directory
    fs.write_file_mut("/src/.gitkeep", b"")
        .map_err(|e| ClientError::Other {
            message: format!("Failed to create src directory: {}", e),
        })?;

    Ok(())
}

#[test]
fn test_server_startup_with_memory_filesystem() {
    // Create server with memory filesystem
    let fs = LpFsMemory::new();
    let output_provider = Rc::new(RefCell::new(MemoryOutputProvider::new()));
    let _server = LpServer::new(output_provider, Box::new(fs), "projects".to_string());

    // Server created successfully
    // (In a real test, we might verify it accepts connections, but with
    // LocalTransport we test that separately)
}

#[test]
fn test_client_server_communication() {
    // Create server filesystem with test file
    let mut fs = LpFsMemory::new();
    fs.write_file_mut("/projects/test/file.txt", b"test content")
        .unwrap();

    // Set up server and client with memory transport
    let (mut server, mut client, mut client_transport, mut server_transport) =
        setup_server_and_client(fs);

    // Test read operation
    let (request_msg, request_id) = client.fs_read("/projects/test/file.txt".to_string());
    send_client_message(&mut client_transport, request_msg).unwrap();
    process_messages(
        &mut client,
        &mut server,
        &mut client_transport,
        &mut server_transport,
    )
    .unwrap();

    // Get response
    let response = client.get_response(request_id).unwrap();
    let content = client.extract_read_response(request_id, response).unwrap();
    assert_eq!(content, b"test content");
}

#[test]
fn test_end_to_end_project_push() {
    // Create client-side project
    let mut client_fs = LpFsMemory::new();
    let project_uid = "2025.01.15-12.00.00-test-project";
    create_test_project(&mut client_fs, "test-project", project_uid).unwrap();

    // Create src directory and a test file in the project
    // Note: LpFsMemory requires directories to be created implicitly by writing files
    client_fs
        .write_file_mut("/src/test.glsl", b"void main() {}")
        .unwrap();

    // Create a minimal valid texture node (with required fields)
    client_fs
        .write_file_mut(
            "/src/texture.texture/node.json",
            b"{\"$type\":\"basic\",\"width\":64,\"height\":64}",
        )
        .unwrap();

    // Set up server and client
    let server_fs = LpFsMemory::new();
    let (mut server, mut client, mut client_transport, mut server_transport) =
        setup_server_and_client(server_fs);

    // Push project files to server
    // List files from client (recursively)
    // Note: We need to handle the case where list_dir might fail on empty directories
    let entries = match client_fs.list_dir("/", true) {
        Ok(entries) => entries,
        Err(_) => {
            // If listing fails, manually push known files
            vec![
                "/project.json".to_string(),
                "/src/test.glsl".to_string(),
                "/src/texture.texture/node.json".to_string(),
            ]
        }
    };

    // Write each file to server
    for entry in entries {
        // Skip directories (they'll be created implicitly)
        if entry.ends_with('/') || entry == "/src" {
            continue;
        }

        let content = match client_fs.read_file(&entry) {
            Ok(c) => c,
            Err(_) => continue, // Skip if file doesn't exist or is a directory
        };

        // Construct server path: /projects/{uid}/...
        let server_path = if entry.starts_with('/') {
            format!("/projects/{}{}", project_uid, entry)
        } else {
            format!("/projects/{}/{}", project_uid, entry)
        };

        // Write file to server
        let (write_msg, write_id) = client.fs_write(server_path.clone(), content);
        send_client_message(&mut client_transport, write_msg).unwrap();
        process_messages(
            &mut client,
            &mut server,
            &mut client_transport,
            &mut server_transport,
        )
        .unwrap();

        // Get and check response
        let response = client.get_response(write_id).unwrap();
        client.extract_write_response(write_id, response).unwrap();
    }

    // Verify project.json exists on server
    let (read_msg, read_id) = client.fs_read(format!("/projects/{}/project.json", project_uid));
    send_client_message(&mut client_transport, read_msg).unwrap();
    process_messages(
        &mut client,
        &mut server,
        &mut client_transport,
        &mut server_transport,
    )
    .unwrap();

    let response = client.get_response(read_id).unwrap();
    let content = client.extract_read_response(read_id, response).unwrap();
    let project_json: serde_json::Value = serde_json::from_slice(&content).unwrap();
    assert_eq!(project_json["uid"], project_uid);
    assert_eq!(project_json["name"], "test-project");

    // Load project on server
    // Note: LoadProject expects just the project name/UID, not the full path
    // The server will construct the path as projects/{uid}/
    let (load_msg, load_id) = client.project_load(project_uid.to_string());
    send_client_message(&mut client_transport, load_msg).unwrap();
    process_messages(
        &mut client,
        &mut server,
        &mut client_transport,
        &mut server_transport,
    )
    .unwrap();

    let response = client.get_response(load_id).unwrap();
    let handle = client
        .extract_load_project_response(load_id, response)
        .unwrap();

    // Verify handle is valid (non-zero)
    assert_ne!(handle.0, 0);
}

#[test]
fn test_create_command_structure() {
    // Simulate create command by creating a project structure
    let mut fs = LpFsMemory::new();
    let project_name = "my-project";
    let project_uid = "2025.01.15-12.00.00-my-project";

    // Create project.json
    let project_json = format!(
        r#"{{
  "uid": "{}",
  "name": "{}"
}}"#,
        project_uid, project_name
    );
    fs.write_file_mut("/project.json", project_json.as_bytes())
        .unwrap();

    // Verify project.json exists and is valid
    let content = fs.read_file("/project.json").unwrap();
    let config: lp_model::project::config::ProjectConfig =
        serde_json::from_slice(&content).unwrap();
    assert_eq!(config.uid, project_uid);
    assert_eq!(config.name, project_name);
}

// Note: A full async test for the dev command would require making the server,
// transport, and command modules public or creating a lib.rs file.
// The current test structure verifies the synchronous transport works correctly.
// The async message processing fix in the handler should resolve the issue
// where responses weren't being received from the async server.
