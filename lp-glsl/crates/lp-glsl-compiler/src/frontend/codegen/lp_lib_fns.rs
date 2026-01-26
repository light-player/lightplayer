//! Code generation for LPFX Functions
//!
//! Maps user-facing `lpfx_*` function names to internal implementations
//! and handles vector argument flattening.

use crate::DecimalFormat;
use crate::error::{ErrorCode, GlslError};
use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::semantic::lpfx::lpfx_fn_registry::{find_lpfx_fn, get_builtin_id_for_format};
use crate::frontend::semantic::lpfx::lpfx_sig::{build_call_signature, expand_vector_args};
use crate::semantic::types::Type;
use alloc::{format, vec, vec::Vec};
use cranelift_codegen::ir::{ExtFuncData, ExternalName, FuncRef, InstBuilder, Value, types};

impl<'a, M: cranelift_module::Module> CodegenContext<'a, M> {
    /// Emit code for an LPFX function call.
    ///
    /// # Arguments
    /// * `name` - Function name (e.g., "lpfx_hash1", "lpfx_simplex2")
    /// * `args` - Vector of (value, type) pairs for each argument
    ///
    /// # Returns
    /// Tuple of (result_values, return_type)
    pub fn emit_lp_lib_fn_call(
        &mut self,
        name: &str,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        // Look up function in registry
        let func = find_lpfx_fn(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0400, format!("Unknown LPFX function: {name}"))
        })?;

        // Collect parameter types before flattening (needed for signature)
        let param_types: Vec<Type> = args.iter().map(|(_, ty)| ty.clone()).collect();

        // Flatten vector arguments to individual components
        let mut flat_values = Vec::new();
        let mut flat_types = Vec::new(); // Track types for each flattened value
        for ((vals, _ty), param_ty) in args.iter().zip(&param_types) {
            let base_ty = if param_ty.is_vector() {
                param_ty.vector_base_type().unwrap()
            } else {
                param_ty.clone()
            };
            for _val in vals {
                flat_types.push(base_ty.clone());
            }
            flat_values.extend(vals);
        }
        let flat_args = expand_vector_args(&param_types, &flat_values);

        // Determine decimal format - use Fixed32 for functions that need it, otherwise doesn't matter
        // (hash functions don't use decimal formats)
        let format = match &func.impls {
            crate::frontend::semantic::lpfx::lpfx_fn::LpfxFnImpl::NonDecimal(_) => {
                DecimalFormat::Fixed32 // Won't actually be used
            }
            crate::frontend::semantic::lpfx::lpfx_fn::LpfxFnImpl::Decimal(_) => {
                DecimalFormat::Fixed32
            }
        };

        // Get BuiltinId for this format
        let builtin_id = get_builtin_id_for_format(func, format).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("No implementation found for LPFX function {name} with format {format:?}"),
            )
        })?;

        // Check if this function needs fixed32 mapping (Decimal variant)
        match &func.impls {
            crate::frontend::semantic::lpfx::lpfx_fn::LpfxFnImpl::Decimal(_) => {
                // Emit TestCase call - transform will convert to fixed32 builtin
                // Note: The TestCase call signature expects i32 arguments (for Fixed32), but
                // the frontend works with f32. We need to convert f32 -> i32 (fixed32) for arguments
                // and i32 (fixed32) -> f32 for the return value.
                let func_ref =
                    self.get_lpfx_testcase_call(func, builtin_id, &param_types, format)?;

                // Convert arguments from f32 to i32 (fixed32) if needed
                // The TestCase call signature expects i32 for Fixed32 format
                let mut converted_args = Vec::new();
                for (arg_val, arg_ty) in flat_args.iter().zip(&flat_types) {
                    let converted_arg = if matches!(arg_ty, Type::Float) {
                        // Convert f32 to i32 (fixed32): multiply by 2^16 (65536) and convert to i32
                        let scale = self.builder.ins().f32const(65536.0);
                        let scaled = self.builder.ins().fmul(*arg_val, scale);
                        self.builder.ins().fcvt_to_sint(types::I32, scaled)
                    } else {
                        // Int/UInt types are already i32
                        *arg_val
                    };
                    converted_args.push(converted_arg);
                }

                // Emit call instruction with converted arguments
                let call_inst = self.builder.ins().call(func_ref, &converted_args);

                // Extract return value(s)
                let results = self.builder.inst_results(call_inst);
                if results.len() != 1 {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "Expected 1 return value from LPFX function, got {}",
                            results.len()
                        ),
                    ));
                }

                // Copy the value out to avoid borrow checker issues
                let i32_result = results[0];

                // Convert i32 (fixed32) result to f32 for frontend
                // The TestCase call returns i32, but frontend expects f32 for Float types
                let result_value = if matches!(func.glsl_sig.return_type, Type::Float) {
                    // Convert i32 fixed-point to f32: divide by 2^16 (65536)
                    // Note: This conversion happens in the frontend, but the transform will
                    // convert it back to i32 when processing the TestCase call
                    let scale = self.builder.ins().iconst(types::I32, 65536);
                    let scaled = self.builder.ins().sdiv(i32_result, scale);
                    // Convert i32 to f32
                    self.builder.ins().fcvt_from_sint(types::F32, scaled)
                } else {
                    i32_result
                };

                Ok((vec![result_value], func.glsl_sig.return_type.clone()))
            }
            crate::frontend::semantic::lpfx::lpfx_fn::LpfxFnImpl::NonDecimal(_) => {
                // Direct builtin call (hash functions don't need conversion)
                let func_ref = self
                    .gl_module
                    .get_builtin_func_ref(builtin_id, self.builder.func)?;

                // Emit call instruction
                let call_inst = self.builder.ins().call(func_ref, &flat_args);

                // Extract return value(s)
                let results = self.builder.inst_results(call_inst);
                if results.len() != 1 {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "Expected 1 return value from LPFX function, got {}",
                            results.len()
                        ),
                    ));
                }

                Ok((vec![results[0]], func.glsl_sig.return_type.clone()))
            }
        }
    }

    /// Helper to declare and get FuncRef for LPFX function TestCase call.
    ///
    /// Creates external function calls using TestCase names (e.g., "__lpfx_simplex3").
    /// These are converted to fixed32 builtins by the transform.
    ///
    /// Note: The signature always uses f32 for Float return types, even though the
    /// actual builtin returns i32 for Fixed32 format. The transform will handle
    /// the conversion from f32 to i32 when processing the TestCase call.
    fn get_lpfx_testcase_call(
        &mut self,
        func: &'static crate::frontend::semantic::lpfx::lpfx_fn::LpfxFn,
        builtin_id: crate::backend::builtins::registry::BuiltinId,
        _param_types: &[Type],
        format: DecimalFormat,
    ) -> Result<FuncRef, GlslError> {
        // TestCase name is the GLSL function name with __ prefix
        let testcase_name = format!("__{}", func.glsl_sig.name);

        // Build signature dynamically
        // Note: The signature returns i32 for Fixed32 format, but we'll convert it to f32
        // in emit_lp_lib_fn_call before returning to the frontend
        let sig = build_call_signature(func, builtin_id, format);

        // Create TestCase name for external function call
        let sig_ref = self.builder.func.import_signature(sig);
        let ext_name = ExternalName::testcase(testcase_name.as_bytes());
        let ext_func = ExtFuncData {
            name: ext_name,
            signature: sig_ref,
            colocated: false,
        };
        Ok(self.builder.func.import_function(ext_func))
    }
}
