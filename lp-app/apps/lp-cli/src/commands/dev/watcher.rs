//! File watcher for detecting local filesystem changes
//!
//! Wraps the `notify` crate to watch for file changes in the project directory
//! and converts them to `FsChange` events for syncing to the server.

use anyhow::{Context, Result};
use lp_shared::fs::fs_event::{ChangeType, FsChange};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

/// File watcher that monitors a project directory for changes
pub struct FileWatcher {
    /// The underlying notify watcher
    watcher: RecommendedWatcher,
    /// Receiver for file change events from notify
    receiver: mpsc::Receiver<Result<Event, notify::Error>>,
    /// Project root directory (for converting absolute paths to relative paths)
    project_root: PathBuf,
}

impl FileWatcher {
    /// Create a new file watcher for the given project directory
    ///
    /// # Arguments
    ///
    /// * `project_dir` - Path to the project root directory (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// * `Ok(FileWatcher)` - If watcher was created successfully
    /// * `Err` - If watcher creation failed
    pub fn new(project_dir: PathBuf) -> Result<Self> {
        // Convert to absolute path (canonicalize resolves symlinks and makes it absolute)
        let project_root = project_dir
            .canonicalize()
            .with_context(|| {
                format!(
                    "Failed to canonicalize project directory: {}",
                    project_dir.display()
                )
            })?;

        // Create channel for receiving events
        let (tx, rx) = mpsc::channel();

        // Create watcher with event sender
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                // Send event to receiver (ignore send errors - receiver may be dropped)
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )
        .context("Failed to create file watcher")?;

        // Watch the project directory recursively
        watcher
            .watch(&project_root, RecursiveMode::Recursive)
            .with_context(|| format!("Failed to watch directory: {}", project_root.display()))?;

        Ok(Self {
            watcher,
            receiver: rx,
            project_root,
        })
    }

    /// Collect pending file changes (non-blocking)
    ///
    /// Reads all available events from the watcher and converts them to `FsChange` events.
    /// Returns immediately if no events are available.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FsChange>)` - List of file changes (may be empty)
    /// * `Err` - If an error occurred while collecting changes
    pub fn collect_changes(&mut self) -> Result<Vec<FsChange>> {
        let mut changes = Vec::new();

        // Collect all available events (non-blocking)
        loop {
            match self.receiver.try_recv() {
                Ok(Ok(event)) => {
                    // Convert notify event to FsChange events
                    let fs_changes = self.convert_event(&event)?;
                    changes.extend(fs_changes);
                }
                Ok(Err(e)) => {
                    // Watcher error - log but continue
                    eprintln!("File watcher error: {}", e);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No more events available
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Watcher was stopped
                    return Err(anyhow::anyhow!("File watcher disconnected"));
                }
            }
        }

        Ok(changes)
    }

    /// Convert a notify Event to FsChange events
    ///
    /// Handles path normalization and filters out irrelevant events.
    fn convert_event(&self, event: &Event) -> Result<Vec<FsChange>> {
        let mut changes = Vec::new();

        // Filter out events for irrelevant paths (e.g., .git, temporary files)
        for path in &event.paths {
            if self.should_ignore_path(path) {
                continue;
            }

            // Convert absolute path to project-relative path
            let relative_path = self.absolute_to_relative(path)?;

            // Determine change type from event kind
            let change_type = match event.kind {
                EventKind::Create(_) => ChangeType::Create,
                EventKind::Modify(_) => {
                    // For modify events, check if file exists to distinguish create vs modify
                    // Since we're watching, a modify usually means the file already existed
                    // We'll treat it as Modify, but the sync logic can handle both cases
                    ChangeType::Modify
                }
                EventKind::Remove(_) => ChangeType::Delete,
                EventKind::Any => {
                    // For "Any" events, try to determine the actual type
                    // If file doesn't exist, it's a delete; otherwise it's a modify
                    if path.exists() {
                        ChangeType::Modify
                    } else {
                        ChangeType::Delete
                    }
                }
                _ => {
                    // Ignore other event types (e.g., access, other)
                    continue;
                }
            };

            changes.push(FsChange {
                path: relative_path,
                change_type,
            });
        }

        Ok(changes)
    }

    /// Check if a path should be ignored
    ///
    /// Filters out temporary files, hidden files/directories, and build artifacts.
    fn should_ignore_path(&self, path: &Path) -> bool {
        // Get path components
        let components: Vec<_> = path.components().collect();

        // Check for hidden files/directories (starting with .)
        for component in &components {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') {
                    // Ignore .git, .DS_Store, etc., but allow files like .gitignore
                    // For now, ignore all hidden directories
                    if name_str == ".git" || name_str == ".DS_Store" {
                        return true;
                    }
                }
            }
        }

        // Check for common temporary/build directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/.cargo/")
            || path_str.contains("/node_modules/")
        {
            return true;
        }

        // Check for temporary/backup files (common editor backup patterns)
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            // Filter out files ending with ~ (common backup pattern)
            // Filter out files starting with .# (emacs lock files)
            // Filter out files ending with .swp, .swo (vim swap files)
            if file_name_str.ends_with('~')
                || file_name_str.starts_with(".#")
                || file_name_str.ends_with(".swp")
                || file_name_str.ends_with(".swo")
            {
                return true;
            }
        }

        false
    }

    /// Convert an absolute path to a project-relative path
    ///
    /// Converts `/Users/.../project/src/file.glsl` to `/src/file.glsl`
    fn absolute_to_relative(&self, absolute_path: &Path) -> Result<String> {
        // Strip the project root prefix
        let relative = absolute_path
            .strip_prefix(&self.project_root)
            .with_context(|| {
                format!(
                    "Path {} is not under project root {}",
                    absolute_path.display(),
                    self.project_root.display()
                )
            })?;

        // Convert to string and ensure it starts with /
        let mut path_str = relative.to_string_lossy().to_string();
        if !path_str.starts_with('/') {
            path_str = format!("/{}", path_str);
        }

        // Normalize path separators (Windows uses \, Unix uses /)
        // For cross-platform compatibility, normalize to /
        path_str = path_str.replace('\\', "/");

        Ok(path_str)
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        // Watcher will stop automatically when dropped
        // The notify crate handles cleanup
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_absolute_to_relative() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Create a dummy watcher just for testing the path conversion
        // We can't easily test the full watcher without actual file system events
        let watcher = FileWatcher::new(project_root.clone()).unwrap();

        // Test path conversion
        let test_file = project_root.join("src").join("file.glsl");
        let relative = watcher.absolute_to_relative(&test_file).unwrap();
        assert_eq!(relative, "/src/file.glsl");

        // Test root file
        let root_file = project_root.join("project.json");
        let relative = watcher.absolute_to_relative(&root_file).unwrap();
        assert_eq!(relative, "/project.json");
    }

    #[test]
    fn test_should_ignore_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        let watcher = FileWatcher::new(project_root.clone()).unwrap();

        // Should ignore .git directory
        assert!(watcher.should_ignore_path(&project_root.join(".git/config")));

        // Should ignore .DS_Store
        assert!(watcher.should_ignore_path(&project_root.join(".DS_Store")));

        // Should ignore target directory
        assert!(watcher.should_ignore_path(&project_root.join("target/debug/file")));

        // Should not ignore normal files
        assert!(!watcher.should_ignore_path(&project_root.join("src/file.glsl")));
        assert!(!watcher.should_ignore_path(&project_root.join("project.json")));
    }

    #[test]
    fn test_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Create watcher
        let mut watcher = FileWatcher::new(project_root.clone()).unwrap();

        // Create a test file to trigger an event
        let test_file = project_root.join("test.txt");
        fs::write(&test_file, b"test content").unwrap();

        // Give watcher a moment to detect the change
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Collect changes (may or may not have detected the change yet, depending on timing)
        let changes = watcher.collect_changes().unwrap();
        // We can't assert specific changes due to timing, but we can verify it doesn't crash
        // In a real scenario, we'd wait longer or use a more sophisticated test setup
    }
}
