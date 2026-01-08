//! Project wrapper for managing a single project instance

extern crate alloc;

use crate::error::ServerError;
use alloc::{format, string::String};
use lp_engine::app::{EngineEnv, LpEngine};
use lp_engine::error::Error as CoreError;

/// A project instance wrapping an LpEngine
pub struct Project {
    /// Project name/identifier
    name: String,
    /// Project filesystem path
    path: String,
    /// The underlying LpEngine instance
    engine: LpEngine,
}

impl Project {
    /// Create a new project instance
    ///
    /// The project must already exist on the filesystem.
    pub fn new(name: String, path: String, engine_env: EngineEnv) -> Result<Self, ServerError> {
        let mut engine = LpEngine::new(engine_env);

        // Load the project
        engine.load_project().map_err(|e| match e {
            CoreError::Filesystem(msg) => ServerError::Filesystem(msg),
            e => ServerError::Core(format!("{}", e)),
        })?;

        Ok(Self { name, path, engine })
    }

    /// Get the project name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the project path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get mutable access to the underlying LpEngine
    pub fn engine_mut(&mut self) -> &mut LpEngine {
        &mut self.engine
    }

    /// Get immutable access to the underlying LpEngine
    pub fn engine(&self) -> &LpEngine {
        &self.engine
    }

    /// Reload the project from the filesystem
    pub fn reload(&mut self) -> Result<(), ServerError> {
        self.engine.load_project().map_err(|e| match e {
            CoreError::Filesystem(msg) => ServerError::Filesystem(msg),
            e => ServerError::Core(format!("{}", e)),
        })
    }
}
