use crate::DecimalFormat;
use alloc::vec::Vec;
use crate::semantic::functions::FunctionSignature;

pub struct LpfxFn {
    /// GLSL name of the function, example: lpfx_hash
    pub glsl_sig: FunctionSignature,

    pub impls: Vec<LpfxFnImpl>,
}


/// Info about one implementation of a function, for a particular decimal format
pub struct LpfxFnImpl {
    /// Decimal format for the function, if applicable.
    /// None signifies that the function applies to all formats and doesn't use decimal numbers.
    pub decimal_format: Option<DecimalFormat>,

    /// Name of the builtin module in which this function is defined
    /// in lp_builtins, example: builtins::lpfx::hash
    pub builtin_module: &'static str,

    /// Name of the Rust function that implements this function.
    /// Example:  __lpfx_hash
    pub rust_fn_name: &'static str,
}

pub enum LpfxImplType {
    F32,
    I32,
    U32
}