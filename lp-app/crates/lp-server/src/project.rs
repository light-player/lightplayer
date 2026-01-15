//! Project wrapper for managing a single project instance

extern crate alloc;

use crate::error::ServerError;
use alloc::{boxed::Box, format, rc::Rc, string::String, sync::Arc};
use core::cell::RefCell;
use lp_engine::ProjectRuntime;
use lp_shared::fs::LpFs;
use lp_shared::output::{OutputChannelHandle, OutputFormat, OutputProvider};

/// Wrapper to convert Rc<RefCell<dyn OutputProvider>> to Arc<dyn OutputProvider>
///
/// This allows ProjectRuntime (which needs Arc for multi-threading) to work with
/// LpServer's Rc<RefCell> (for no_std single-threaded environments).
struct OutputProviderWrapper {
    inner: Rc<RefCell<dyn OutputProvider>>,
}

impl OutputProvider for OutputProviderWrapper {
    fn open(
        &self,
        pin: u32,
        byte_count: u32,
        format: OutputFormat,
    ) -> Result<OutputChannelHandle, lp_shared::OutputError> {
        self.inner.borrow().open(pin, byte_count, format)
    }

    fn write(
        &self,
        handle: OutputChannelHandle,
        data: &[u8],
    ) -> Result<(), lp_shared::OutputError> {
        self.inner.borrow().write(handle, data)
    }

    fn close(&self, handle: OutputChannelHandle) -> Result<(), lp_shared::OutputError> {
        self.inner.borrow().close(handle)
    }
}

/// A project instance wrapping a ProjectRuntime
pub struct Project {
    /// Project name/identifier
    name: String,
    /// Project filesystem path
    path: String,
    /// The underlying ProjectRuntime instance
    runtime: ProjectRuntime,
    /// Wrapped output provider (kept alive for the lifetime of Project)
    _output_provider: Arc<dyn OutputProvider>,
}

impl Project {
    /// Create a new project instance
    ///
    /// The project must already exist on the filesystem.
    /// Takes an OutputProvider from the server as Rc<RefCell> (for no_std compatibility)
    /// and converts it to Arc for ProjectRuntime.
    pub fn new(
        name: String,
        path: String,
        fs: Box<dyn LpFs>,
        output_provider: Rc<RefCell<dyn OutputProvider>>,
    ) -> Result<Self, ServerError> {
        // Wrap Rc<RefCell> in a type that implements OutputProvider, then wrap in Arc
        let wrapper = OutputProviderWrapper {
            inner: output_provider,
        };
        let arc_provider: Arc<dyn OutputProvider> = Arc::new(wrapper);

        let runtime = ProjectRuntime::new(fs, arc_provider.clone())
            .map_err(|e| ServerError::Core(format!("{}", e)))?;

        Ok(Self {
            name,
            path,
            runtime,
            _output_provider: arc_provider,
        })
    }

    /// Get the project name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the project path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get mutable access to the underlying ProjectRuntime
    pub fn runtime_mut(&mut self) -> &mut ProjectRuntime {
        &mut self.runtime
    }

    /// Get immutable access to the underlying ProjectRuntime
    pub fn runtime(&self) -> &ProjectRuntime {
        &self.runtime
    }

    /// Reload the project from the filesystem
    pub fn reload(&mut self) -> Result<(), ServerError> {
        // todo!("Implement project reload")
        Ok(())
    }
}
