//! Filesystem API message types
//!
//! Defines request and response types for filesystem operations.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Filesystem operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "fsType", rename_all = "camelCase")]
pub enum FsRequest {
    /// Read a file
    Read { path: String },
    /// Write a file
    Write { path: String, data: Vec<u8> },
    /// Delete a file
    DeleteFile { path: String },
    /// Delete a directory (always recursive)
    DeleteDir { path: String },
    /// List directory contents
    ListDir { path: String, recursive: bool },
}

/// Filesystem operation response
///
/// All response variants include an optional error field.
/// If `error` is `Some`, the operation failed and other fields may be empty/default.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "fsType", rename_all = "camelCase")]
pub enum FsResponse {
    /// Response to Read request
    Read {
        path: String,
        data: Option<Vec<u8>>,
        error: Option<String>,
    },
    /// Response to Write request
    Write { path: String, error: Option<String> },
    /// Response to DeleteFile request
    DeleteFile { path: String, error: Option<String> },
    /// Response to DeleteDir request
    DeleteDir { path: String, error: Option<String> },
    /// Response to ListDir request
    ListDir {
        path: String,
        entries: Vec<String>,
        error: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_fs_request_serialization() {
        let req = FsRequest::Read {
            path: "/project.json".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        // Verify round-trip serialization
        let deserialized: FsRequest = serde_json::from_str(&json).unwrap();
        match deserialized {
            FsRequest::Read { path } => assert_eq!(path, "/project.json"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_fs_response_serialization() {
        let resp = FsResponse::Read {
            path: "/project.json".to_string(),
            data: Some(b"{}".to_vec()),
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        // With tag="type" and rename_all="camelCase", JSON uses lowercase "read"
        assert!(json.contains("read") || json.contains("Read"));
        assert!(json.contains("/project.json"));

        let deserialized: FsResponse = serde_json::from_str(&json).unwrap();
        match deserialized {
            FsResponse::Read { path, data, error } => {
                assert_eq!(path, "/project.json");
                assert_eq!(data, Some(b"{}".to_vec()));
                assert_eq!(error, None);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_fs_response_with_error() {
        let resp = FsResponse::Write {
            path: "/test.txt".to_string(),
            error: Some("Permission denied".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        // Verify round-trip serialization
        let deserialized: FsResponse = serde_json::from_str(&json).unwrap();
        match deserialized {
            FsResponse::Write { path, error } => {
                assert_eq!(path, "/test.txt");
                assert_eq!(error, Some("Permission denied".to_string()));
            }
            _ => panic!("Wrong variant"),
        }
    }
}
