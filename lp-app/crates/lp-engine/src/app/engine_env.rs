//! Platform-specific trait implementations wrapper

use crate::traits::{LpFs, OutputProvider};

/// Environment for the LightPlayer Engine, providing platform-specific trait
/// implementations.
pub struct EngineEnv {
    /// View of the project filesystem, rooted at the project root
    pub fs: alloc::boxed::Box<dyn LpFs>,

    /// Output provider for creating LED outputs
    pub output: alloc::boxed::Box<dyn OutputProvider>,
}

impl EngineEnv {
    pub fn new(
        fs: alloc::boxed::Box<dyn LpFs>,
        output: alloc::boxed::Box<dyn OutputProvider>,
    ) -> Self {
        Self { fs, output }
    }
}
