//! Variable expression resolution

use crate::error::{GlslError, extract_span_from_identifier, source_span_to_location};
use crate::frontend::codegen::context::CodegenContext;
use alloc::vec::Vec;

use super::super::types::LValue;

/// Resolve a variable expression to an LValue
pub fn resolve_variable_lvalue<M: cranelift_module::Module>(
    ctx: &mut CodegenContext<'_, M>,
    ident: &glsl::syntax::Identifier,
) -> Result<LValue, GlslError> {
    let span = extract_span_from_identifier(ident);

    // Get variable type first to check if it's an array
    let ty = ctx
        .lookup_variable_type(&ident.name)
        .ok_or_else(|| {
            let error = GlslError::undefined_variable(&ident.name)
                .with_location(source_span_to_location(&span));
            ctx.add_span_to_error(error, &span)
        })?
        .clone();

    // Check if this is an out/inout parameter
    let var_info = ctx.lookup_var_info(&ident.name);
    let is_out_inout = var_info.and_then(|info| info.out_inout_ptr).is_some();

    // For arrays, return LValue::Variable with empty vars (arrays use pointer-based storage)
    if ty.is_array() {
        return Ok(LValue::Variable {
            vars: Vec::new(),
            ty,
            name: Some(ident.name.clone()),
        });
    }

    let vars = ctx
        .lookup_variables(&ident.name)
        .ok_or_else(|| {
            let error = GlslError::undefined_variable(&ident.name)
                .with_location(source_span_to_location(&span));
            ctx.add_span_to_error(error, &span)
        })?
        .to_vec();

    Ok(LValue::Variable {
        vars,
        ty,
        name: if is_out_inout {
            Some(ident.name.clone())
        } else {
            None
        },
    })
}
