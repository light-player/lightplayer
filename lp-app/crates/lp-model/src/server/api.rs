use crate::project::{ProjectHandle, ProjectRequest};
use crate::server::fs_api::{FsRequest, FsResponse};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "requestType", rename_all = "camelCase")]
pub enum ServerRequest {
    /// Filesystem operation request
    Filesystem(FsRequest),
    /// Load a project
    LoadProject {
        path: String,
    },
    /// Unload a project
    UnloadProject {
        handle: ProjectHandle,
    },
    /// Project-specific request
    ProjectRequest {
        handle: ProjectHandle,
        request: ProjectRequest,
    },
    /// List available projects
    ListAvailableProjects,
    /// List loaded projects
    ListLoadedProjects,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "responseType", rename_all = "camelCase")]
pub enum ServerResponse {
    /// Filesystem operation response
    Filesystem(FsResponse),
    /// Response to LoadProject
    LoadProject { handle: ProjectHandle },
    /// Response to UnloadProject
    UnloadProject,
    /// Response to ProjectRequest
    ///
    /// TODO: ProjectResponse serialization is disabled because ProjectResponse contains
    /// `NodeDetail` which includes `Box<dyn NodeConfig>` (a trait object) that cannot be
    /// serialized directly with serde.
    ///
    /// Options for future implementation:
    /// 1. Create a serializable wrapper type that converts trait objects to concrete types
    /// 2. Implement custom Serialize/Deserialize for ProjectResponse
    /// 3. Refactor NodeDetail to use an enum instead of trait objects
    ///
    /// See: `lp-model/src/project/api.rs` for ProjectResponse definition
    /// See: `lp-model/src/project/api.rs::NodeDetail` for the problematic type
    // ProjectRequest { response: ProjectResponse },
    /// Response to ListAvailableProjects
    ListAvailableProjects { projects: Vec<AvailableProject> },
    /// Response to ListLoadedProjects
    ListLoadedProjects { projects: Vec<LoadedProject> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableProject {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedProject {
    pub handle: ProjectHandle,
    pub path: String,
}
