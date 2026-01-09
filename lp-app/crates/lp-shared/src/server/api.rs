use crate::project::api::{ProjectRequest, ProjectResponse};
use crate::project::handle::ProjectHandle;
use alloc::string::String;
use alloc::vec::Vec;

pub enum ServerRequest {
    LoadProject {
        path: String,
    },
    UnloadProject {
        handle: ProjectHandle,
    },
    ProjectRequest {
        handle: ProjectHandle,
        request: ProjectRequest,
    },
    ListAvailableProjects,
    ListLoadedProjects,
}

pub enum ServerResponse {
    LoadProject { handle: ProjectHandle },
    UnloadProject,
    ProjectRequest { response: ProjectResponse },
    ListAvailableProjects { projects: Vec<AvailableProject> },
    ListLoadedProjects { projects: Vec<LoadedProject> },
}

pub struct AvailableProject {
    path: String,
}

pub struct LoadedProject {
    handle: ProjectHandle,
    path: String,
}
