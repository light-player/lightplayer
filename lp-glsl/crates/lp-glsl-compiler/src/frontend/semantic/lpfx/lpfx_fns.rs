//! LPFX function registry
//!
//! This module contains the const array of all LPFX functions.
//! This will be codegen output in the future, but for now is manually maintained.

use super::lpfx_fn::{LpfxFn, LpfxFnImpl};
use crate::DecimalFormat;
use crate::semantic::functions::{FunctionSignature, ParamQualifier, Parameter};
use crate::semantic::types::Type;
use alloc::string::String;
use alloc::vec;

/// Registry of all LPFX functions
///
/// This is the single source of truth for all LPFX function definitions.
/// Functions are looked up by name from this array.
pub const LPFX_FNS: &[LpfxFn] = &[
    // Hash functions
    LpfxFn {
        glsl_sig: FunctionSignature {
            name: String::from("lpfx_hash1"),
            return_type: Type::UInt,
            parameters: vec![
                Parameter {
                    name: String::from("x"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("seed"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
            ],
        },
        impls: vec![LpfxFnImpl {
            decimal_format: None,
            builtin_module: "builtins::lpfx::hash",
            rust_fn_name: "__lpfx_hash_1",
        }],
    },
    LpfxFn {
        glsl_sig: FunctionSignature {
            name: String::from("lpfx_hash2"),
            return_type: Type::UInt,
            parameters: vec![
                Parameter {
                    name: String::from("x"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("y"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("seed"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
            ],
        },
        impls: vec![LpfxFnImpl {
            decimal_format: None,
            builtin_module: "builtins::lpfx::hash",
            rust_fn_name: "__lpfx_hash_2",
        }],
    },
    LpfxFn {
        glsl_sig: FunctionSignature {
            name: String::from("lpfx_hash3"),
            return_type: Type::UInt,
            parameters: vec![
                Parameter {
                    name: String::from("x"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("y"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("z"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("seed"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
            ],
        },
        impls: vec![LpfxFnImpl {
            decimal_format: None,
            builtin_module: "builtins::lpfx::hash",
            rust_fn_name: "__lpfx_hash_3",
        }],
    },
    // Simplex noise functions
    LpfxFn {
        glsl_sig: FunctionSignature {
            name: String::from("lpfx_simplex1"),
            return_type: Type::Float,
            parameters: vec![
                Parameter {
                    name: String::from("x"),
                    ty: Type::Float,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("seed"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
            ],
        },
        impls: vec![LpfxFnImpl {
            decimal_format: Some(DecimalFormat::Fixed32),
            builtin_module: "builtins::lpfx::simplex",
            rust_fn_name: "__lpfx_simplex1_q32",
        }],
    },
    LpfxFn {
        glsl_sig: FunctionSignature {
            name: String::from("lpfx_simplex2"),
            return_type: Type::Float,
            parameters: vec![
                Parameter {
                    name: String::from("p"),
                    ty: Type::Vec2,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("seed"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
            ],
        },
        impls: vec![LpfxFnImpl {
            decimal_format: Some(DecimalFormat::Fixed32),
            builtin_module: "builtins::lpfx::simplex",
            rust_fn_name: "__lpfx_simplex2_q32",
        }],
    },
    LpfxFn {
        glsl_sig: FunctionSignature {
            name: String::from("lpfx_simplex3"),
            return_type: Type::Float,
            parameters: vec![
                Parameter {
                    name: String::from("p"),
                    ty: Type::Vec3,
                    qualifier: ParamQualifier::In,
                },
                Parameter {
                    name: String::from("seed"),
                    ty: Type::UInt,
                    qualifier: ParamQualifier::In,
                },
            ],
        },
        impls: vec![LpfxFnImpl {
            decimal_format: Some(DecimalFormat::Fixed32),
            builtin_module: "builtins::lpfx::simplex",
            rust_fn_name: "__lpfx_simplex3_q32",
        }],
    },
];
