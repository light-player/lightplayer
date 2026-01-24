use anyhow::Result;
use cranelift_jit::JITModule;
use lp_glsl_compiler::GlslCompiler;
use lp_glsl_compiler::backend::module::gl_module::GlModule;
use lp_glsl_compiler::backend::target::Target;
use lp_glsl_compiler::backend::transform::fixed32::{Fixed32Transform, FixedPointFormat};

pub fn compile_and_transform(
    glsl_source: &str,
    format: FixedPointFormat,
) -> Result<(GlModule<JITModule>, GlModule<JITModule>)> {
    let target =
        Target::host_jit().map_err(|e| anyhow::anyhow!("Failed to create target: {}", e))?;

    // Compile twice: once for before transform, once for after transform
    // We need separate modules because apply_transform consumes the module

    // Compile to module (before transform)
    let mut compiler_before = GlslCompiler::new();
    let module_before = compiler_before
        .compile_to_gl_module_jit(glsl_source, target.clone())
        .map_err(|e| anyhow::anyhow!("Failed to compile GLSL: {}", e))?;

    // Compile again for after transform
    let mut compiler_after = GlslCompiler::new();
    let module_for_transform = compiler_after
        .compile_to_gl_module_jit(glsl_source, target)
        .map_err(|e| anyhow::anyhow!("Failed to compile GLSL (for transform): {}", e))?;

    // Apply fixed32 transform (consumes module_for_transform)
    let transform = Fixed32Transform::new(format);
    let module_after = module_for_transform
        .apply_transform(transform)
        .map_err(|e| anyhow::anyhow!("Failed to apply fixed32 transform: {}", e))?;

    Ok((module_before, module_after))
}
