use crate::error::Error;
use crate::nodes::NodeRuntime;
use crate::runtime::contexts::{NodeInitContext, RenderContext, TextureHandle};
use alloc::{boxed::Box, format, string::String};
use lp_glsl_compiler::{glsl_jit, GlslExecutable, GlslOptions, DecimalFormat, RunMode};
use lp_model::{NodeHandle, nodes::shader::{ShaderConfig, ShaderState}};

/// Shader node runtime
pub struct ShaderRuntime {
    config: Option<ShaderConfig>,
    glsl_source: Option<String>,           // Stored for state extraction
    executable: Option<Box<dyn GlslExecutable + Send + Sync>>,  // Compiled shader (must be Send + Sync for NodeRuntime)
    texture_handle: Option<TextureHandle>,  // Resolved texture handle
    compilation_error: Option<String>,      // Compilation error if any
    node_handle: NodeHandle,
}

impl ShaderRuntime {
    pub fn new(node_handle: NodeHandle) -> Self {
        Self {
            config: None,
            glsl_source: None,
            executable: None,
            texture_handle: None,
            compilation_error: None,
            node_handle,
        }
    }

    pub fn set_config(&mut self, config: ShaderConfig) {
        self.config = Some(config);
    }

    pub fn get_state(&self) -> ShaderState {
        ShaderState {
            glsl_code: self.glsl_source.clone().unwrap_or_default(),
            error: self.compilation_error.clone(),
        }
    }
}

impl NodeRuntime for ShaderRuntime {
    fn init(&mut self, ctx: &dyn NodeInitContext) -> Result<(), Error> {
        let config = self.config.as_ref().ok_or_else(|| Error::InvalidConfig {
            node_path: format!("shader-{}", self.node_handle.as_i32()),
            reason: alloc::string::String::from("Config not set"),
        })?;

        // Load GLSL source from filesystem
        let glsl_path = &config.glsl_path;
        let fs = ctx.get_node_fs();
        let source_bytes = fs.read_file(glsl_path).map_err(|e| Error::Io {
            path: glsl_path.clone(),
            details: format!("Failed to read GLSL file: {:?}", e),
        })?;
        
        let glsl_source = alloc::string::String::from_utf8(source_bytes).map_err(|e| Error::Parse {
            file: glsl_path.clone(),
            error: format!("Invalid UTF-8 in GLSL file: {}", e),
        })?;
        
        // Store source for state extraction
        self.glsl_source = Some(glsl_source.clone());

        // Resolve texture handle
        let texture_handle = ctx.resolve_texture(&config.texture_spec).map_err(|e| {
            self.compilation_error = Some(format!("Failed to resolve texture: {}", e));
            e
        })?;
        self.texture_handle = Some(texture_handle);

        // Compile GLSL shader
        // Use Fixed32 format (Float format not yet supported)
        let options = GlslOptions {
            run_mode: RunMode::HostJit,
            decimal_format: DecimalFormat::Fixed32,
        };

        match glsl_jit(&glsl_source, options) {
            Ok(executable) => {
                // Cast to add Send + Sync bounds (GlslJitModule is safe to send/sync)
                // The function pointers are stable and don't change after compilation
                let executable_with_bounds: Box<dyn GlslExecutable + Send + Sync> = unsafe {
                    core::mem::transmute(executable)
                };
                self.executable = Some(executable_with_bounds);
                self.compilation_error = None;
                Ok(())
            }
            Err(e) => {
                // Store compilation error but don't fail initialization
                // This allows the shader to be in an error state but still report the error
                self.compilation_error = Some(format!("{}", e));
                self.executable = None;
                // Return error so node status is set to InitError
                Err(Error::InvalidConfig {
                    node_path: format!("shader-{}", self.node_handle.as_i32()),
                    reason: format!("GLSL compilation failed: {}", e),
                })
            }
        }
    }

    fn render(&mut self, _ctx: &mut dyn RenderContext) -> Result<(), Error> {
        // todo!("Shader rendering - execute GLSL")
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_runtime_creation() {
        let handle = lp_model::NodeHandle::new(0);
        let runtime = ShaderRuntime::new(handle);
        let _boxed: alloc::boxed::Box<dyn NodeRuntime> = alloc::boxed::Box::new(runtime);
    }
}
