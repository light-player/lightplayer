//! LPFX function definitions
//!
//! Core data structures for representing LPFX functions and their implementations.

use crate::DecimalFormat;
use crate::semantic::functions::FunctionSignature;
use alloc::vec::Vec;

/// LPFX function definition
///
/// Contains the GLSL signature and all available implementations for different decimal formats.
pub struct LpfxFn {
    /// GLSL signature of the function (name, parameters, return type)
    pub glsl_sig: FunctionSignature,

    /// Available implementations for different decimal formats
    pub impls: Vec<LpfxFnImpl>,
}

/// Implementation of an LPFX function for a specific decimal format
///
/// Each function may have multiple implementations (e.g., one for q32 fixed-point,
/// one format-agnostic for hash functions).
pub struct LpfxFnImpl {
    /// Decimal format for this implementation.
    ///
    /// `None` signifies that the function applies to all formats and doesn't use decimal numbers
    /// (e.g., hash functions).
    pub decimal_format: Option<DecimalFormat>,

    /// Name of the builtin module in which this function is defined in lp_builtins.
    ///
    /// Example: `"builtins::lpfx::hash"`
    pub builtin_module: &'static str,

    /// Name of the Rust function that implements this function.
    ///
    /// Example: `"__lpfx_hash_1"`
    pub rust_fn_name: &'static str,
}
