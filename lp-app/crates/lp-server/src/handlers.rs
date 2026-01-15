//! Message handlers for LpServer

extern crate alloc;

use crate::error::ServerError;
use crate::project_manager::ProjectManager;
use alloc::{format, rc::Rc, string::ToString, vec::Vec};
use core::cell::RefCell;
use lp_model::{
    server::{FsRequest, FsResponse, ServerRequest, ServerResponse},
    ClientMessage, ServerMessage,
};
use lp_shared::fs::LpFs;
use lp_shared::output::OutputProvider;

/// Handle a client message and generate a server response
pub fn handle_client_message(
    _project_manager: &mut ProjectManager,
    base_fs: &mut dyn LpFs,
    _output_provider: &Rc<RefCell<dyn OutputProvider>>,
    client_msg: ClientMessage,
) -> Result<ServerMessage, ServerError> {
    let ClientMessage { id, msg } = client_msg;

    let response = match msg {
        lp_model::ClientRequest::Filesystem(fs_request) => {
            ServerResponse::Filesystem(handle_fs_request(base_fs, fs_request)?)
        }
        // Future: Handle project management requests here
        // For now, return an error if non-filesystem requests are received
    };

    Ok(ServerMessage { id, msg: response })
}

/// Handle a filesystem request
fn handle_fs_request(
    fs: &mut dyn LpFs,
    request: FsRequest,
) -> Result<FsResponse, ServerError> {
    match request {
        FsRequest::Read { path } => {
            match fs.read_file(&path) {
                Ok(data) => Ok(FsResponse::Read {
                    path,
                    data: Some(data),
                    error: None,
                }),
                Err(e) => Ok(FsResponse::Read {
                    path,
                    data: None,
                    error: Some(format!("{}", e)),
                }),
            }
        }
        FsRequest::Write { path, data } => {
            match fs.write_file(&path, &data) {
                Ok(()) => Ok(FsResponse::Write {
                    path,
                    error: None,
                }),
                Err(e) => Ok(FsResponse::Write {
                    path,
                    error: Some(format!("{}", e)),
                }),
            }
        }
        FsRequest::DeleteFile { path } => {
            match fs.delete_file(&path) {
                Ok(()) => Ok(FsResponse::DeleteFile {
                    path,
                    error: None,
                }),
                Err(e) => Ok(FsResponse::DeleteFile {
                    path,
                    error: Some(format!("{}", e)),
                }),
            }
        }
        FsRequest::DeleteDir { path } => {
            match fs.delete_dir(&path) {
                Ok(()) => Ok(FsResponse::DeleteDir {
                    path,
                    error: None,
                }),
                Err(e) => Ok(FsResponse::DeleteDir {
                    path,
                    error: Some(format!("{}", e)),
                }),
            }
        }
        FsRequest::ListDir { path, recursive } => {
            match fs.list_dir(&path, recursive) {
                Ok(entries) => Ok(FsResponse::ListDir {
                    path,
                    entries,
                    error: None,
                }),
                Err(e) => Ok(FsResponse::ListDir {
                    path,
                    entries: Vec::new(),
                    error: Some(format!("{}", e)),
                }),
            }
        }
    }
}

/// Handle a project management request
///
/// TODO: Implement project management handlers
/// This will handle ServerRequest variants like LoadProject, UnloadProject, etc.
///
/// Currently returns an error indicating project management is not yet implemented.
/// This function is not called in the current implementation (only filesystem requests
/// are handled), but exists for future use.
pub fn handle_project_request(
    _project_manager: &mut ProjectManager,
    _output_provider: &Rc<RefCell<dyn OutputProvider>>,
    _request: ServerRequest,
) -> Result<ServerResponse, ServerError> {
    // TODO: Implement project management handlers
    // For now, return an error instead of panicking
    Err(ServerError::Core(
        "Project management requests are not yet implemented".to_string(),
    ))
}
