//! Watch and sync functionality
//!
//! Functions for syncing file changes detected by the file watcher to the server.

use anyhow::{Context, Result};
use lp_shared::fs::{fs_event::FsChange, LpFs, LpFsStd};
use std::path::Path;

use super::async_client::AsyncLpClient;

/// Sync a single file change to the server
///
/// Reads the file from the local filesystem (for Create/Modify) and writes it to the server.
/// For Delete operations, logs a warning since server may not support delete yet.
///
/// # Arguments
///
/// * `client` - The AsyncLpClient instance
/// * `change` - The file change event
/// * `project_uid` - Project UID (used for server path)
/// * `_project_dir` - Local project directory path (unused, kept for API consistency)
/// * `local_fs` - Local filesystem implementation (must not be held across await points)
///
/// # Returns
///
/// * `Ok(())` - If sync succeeded
/// * `Err` - If sync failed
///
/// # Note
///
/// This function reads the file content before any await points to ensure Send safety
/// when used in spawned tasks.
#[cfg_attr(test, allow(dead_code))]
pub async fn sync_file_change(
    client: &mut AsyncLpClient,
    change: &FsChange,
    project_uid: &str,
    _project_dir: &Path,
    local_fs: &dyn LpFs,
) -> Result<()> {
    // Construct server path: /projects/{uid}/{relative_path}
    let server_path = if change.path.starts_with('/') {
        format!("/projects/{}{}", project_uid, change.path)
    } else {
        format!("/projects/{}/{}", project_uid, change.path)
    };

    match change.change_type {
        lp_shared::fs::fs_event::ChangeType::Create | lp_shared::fs::fs_event::ChangeType::Modify => {
            // Read file from local filesystem (before any await to avoid Send issues)
            let content = local_fs
                .read_file(&change.path)
                .map_err(|e| anyhow::anyhow!("Failed to read local file {}: {}", change.path, e))?;

            // Write to server (content is owned, so this is Send-safe)
            client
                .fs_write(server_path.clone(), content)
                .await
                .with_context(|| format!("Failed to sync file {} to server", change.path))?;

            println!("Synced {} -> {}", change.path, server_path);
        }
        lp_shared::fs::fs_event::ChangeType::Delete => {
            // Server may not support delete operations yet
            // For now, log a warning and skip
            eprintln!(
                "Warning: File deletion detected for {}, but delete sync is not yet implemented",
                change.path
            );
            // TODO: Implement delete sync when server supports it
        }
    }

    Ok(())
}

/// Sync multiple file changes to the server
///
/// Processes changes sequentially, syncing each one to the server.
/// Continues processing even if individual changes fail (logs errors).
///
/// # Arguments
///
/// * `client` - The AsyncLpClient instance
/// * `changes` - Vector of file change events
/// * `project_uid` - Project UID (used for server path)
/// * `project_dir` - Local project directory path (used to create LpFsStd instance)
///
/// # Returns
///
/// * `Ok(())` - Always returns Ok (errors are logged but don't stop processing)
///
/// # Note
///
/// This function creates a new LpFsStd instance internally to avoid Send/Sync issues
/// when used in spawned tasks.
#[cfg_attr(test, allow(dead_code))]
pub async fn sync_changes(
    client: &mut AsyncLpClient,
    changes: Vec<FsChange>,
    project_uid: &str,
    project_dir: &Path,
) -> Result<()> {
    // Canonicalize project directory to ensure it's absolute and normalized
    // This is important because LpFsStd needs an absolute path for proper validation
    let canonical_project_dir = project_dir
        .canonicalize()
        .with_context(|| {
            format!(
                "Failed to canonicalize project directory: {}",
                project_dir.display()
            )
        })?;

    // Create local filesystem instance (LpFsStd is just a path wrapper, so this is Send-safe)
    let local_fs = LpFsStd::new(canonical_project_dir);

    // First pass: read all file contents before any await (ensures Send safety)
    let mut sync_operations = Vec::new();

    for change in changes {
        match change.change_type {
            lp_shared::fs::fs_event::ChangeType::Create
            | lp_shared::fs::fs_event::ChangeType::Modify => {
                // Read file content before await
                // Retry reading the file a few times in case it's still being written
                let mut content_result = Err(lp_shared::error::FsError::NotFound(change.path.clone()));
                for attempt in 0..3 {
                    content_result = local_fs.read_file(&change.path);
                    if content_result.is_ok() {
                        break;
                    }
                    // If file doesn't exist yet, wait a bit and retry (editor might still be writing)
                    if attempt < 2 {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
                
                match content_result {
                    Ok(content) => {
                        // Construct server path
                        let server_path = if change.path.starts_with('/') {
                            format!("/projects/{}{}", project_uid, change.path)
                        } else {
                            format!("/projects/{}/{}", project_uid, change.path)
                        };

                        sync_operations.push((change.path.clone(), server_path, Some(content)));
                    }
                    Err(e) => {
                        eprintln!("Failed to read local file {} after retries: {}", change.path, e);
                    }
                }
            }
            lp_shared::fs::fs_event::ChangeType::Delete => {
                // Delete sync not yet implemented
                eprintln!(
                    "Warning: File deletion detected for {}, but delete sync is not yet implemented",
                    change.path
                );
            }
        }
    }

    // Second pass: sync all files (now safe to await since we don't hold local_fs)
    for (change_path, server_path, content) in sync_operations {
        if let Some(content) = content {
            match client
                .fs_write(server_path.clone(), content)
                .await
                .with_context(|| format!("Failed to sync file {} to server", change_path))
            {
                Ok(_) => {
                    println!("Synced {} -> {}", change_path, server_path);
                }
                Err(e) => {
                    eprintln!("Failed to sync change {}: {}", change_path, e);
                }
            }
        }
    }

    Ok(())
}
