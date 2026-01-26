//! This file is AUTO-GENERATED. Do not edit manually.
//!
//! To regenerate this file, run:
//!     cargo run --bin lp-builtin-gen --manifest-path lp-glsl/apps/lp-builtin-gen/Cargo.toml
//!
//! Or use the build script:
//!     scripts/build-builtins.sh

use lp_builtins::builtins::lpfx::hash::{__lpfx_hash_1, __lpfx_hash_2, __lpfx_hash_3};
use lp_builtins::builtins::lpfx::simplex::{
    simplex1_f32::__lpfx_simplex1_f32, simplex1_q32::__lpfx_simplex1_q32,
    simplex2_f32::__lpfx_simplex2_f32, simplex2_q32::__lpfx_simplex2_q32,
    simplex3_f32::__lpfx_simplex3_f32, simplex3_q32::__lpfx_simplex3_q32,
};
use lp_builtins::builtins::q32::{
    __lp_q32_acos, __lp_q32_acosh, __lp_q32_add, __lp_q32_asin, __lp_q32_asinh, __lp_q32_atan,
    __lp_q32_atan2, __lp_q32_atanh, __lp_q32_cos, __lp_q32_cosh, __lp_q32_div, __lp_q32_exp,
    __lp_q32_exp2, __lp_q32_fma, __lp_q32_inversesqrt, __lp_q32_ldexp, __lp_q32_log, __lp_q32_log2,
    __lp_q32_mod, __lp_q32_mul, __lp_q32_pow, __lp_q32_round, __lp_q32_roundeven, __lp_q32_sin,
    __lp_q32_sinh, __lp_q32_sqrt, __lp_q32_sub, __lp_q32_tan, __lp_q32_tanh,
};

/// Reference all builtin functions to prevent dead code elimination.
///
/// This function ensures all builtin functions are included in the executable
/// by creating function pointers and reading them with volatile operations.
pub fn ensure_builtins_referenced() {
    unsafe {
        let _q32_acos_fn: extern "C" fn(i32) -> i32 = __lp_q32_acos;
        let _q32_acosh_fn: extern "C" fn(i32) -> i32 = __lp_q32_acosh;
        let _q32_add_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_add;
        let _q32_asin_fn: extern "C" fn(i32) -> i32 = __lp_q32_asin;
        let _q32_asinh_fn: extern "C" fn(i32) -> i32 = __lp_q32_asinh;
        let _q32_atan_fn: extern "C" fn(i32) -> i32 = __lp_q32_atan;
        let _q32_atan2_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_atan2;
        let _q32_atanh_fn: extern "C" fn(i32) -> i32 = __lp_q32_atanh;
        let _q32_cos_fn: extern "C" fn(i32) -> i32 = __lp_q32_cos;
        let _q32_cosh_fn: extern "C" fn(i32) -> i32 = __lp_q32_cosh;
        let _q32_div_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_div;
        let _q32_exp_fn: extern "C" fn(i32) -> i32 = __lp_q32_exp;
        let _q32_exp2_fn: extern "C" fn(i32) -> i32 = __lp_q32_exp2;
        let _q32_fma_fn: extern "C" fn(i32, i32, i32) -> i32 = __lp_q32_fma;
        let _q32_inversesqrt_fn: extern "C" fn(i32) -> i32 = __lp_q32_inversesqrt;
        let _q32_ldexp_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_ldexp;
        let _q32_log_fn: extern "C" fn(i32) -> i32 = __lp_q32_log;
        let _q32_log2_fn: extern "C" fn(i32) -> i32 = __lp_q32_log2;
        let _q32_mod_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_mod;
        let _q32_mul_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_mul;
        let _q32_pow_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_pow;
        let _q32_round_fn: extern "C" fn(i32) -> i32 = __lp_q32_round;
        let _q32_roundeven_fn: extern "C" fn(i32) -> i32 = __lp_q32_roundeven;
        let _q32_sin_fn: extern "C" fn(i32) -> i32 = __lp_q32_sin;
        let _q32_sinh_fn: extern "C" fn(i32) -> i32 = __lp_q32_sinh;
        let _q32_sqrt_fn: extern "C" fn(i32) -> i32 = __lp_q32_sqrt;
        let _q32_sub_fn: extern "C" fn(i32, i32) -> i32 = __lp_q32_sub;
        let _q32_tan_fn: extern "C" fn(i32) -> i32 = __lp_q32_tan;
        let _q32_tanh_fn: extern "C" fn(i32) -> i32 = __lp_q32_tanh;
        let __lpfx_hash_1_fn: extern "C" fn(u32, u32) -> u32 = __lpfx_hash_1;
        let __lpfx_hash_2_fn: extern "C" fn(u32, u32, u32) -> u32 = __lpfx_hash_2;
        let __lpfx_hash_3_fn: extern "C" fn(u32, u32, u32, u32) -> u32 = __lpfx_hash_3;
        let __lpfx_simplex1_f32_fn: extern "C" fn(f32, u32) -> f32 = __lpfx_simplex1_f32;
        let __lpfx_simplex1_q32_fn: extern "C" fn(i32, u32) -> i32 = __lpfx_simplex1_q32;
        let __lpfx_simplex2_f32_fn: extern "C" fn(f32, f32, u32) -> f32 = __lpfx_simplex2_f32;
        let __lpfx_simplex2_q32_fn: extern "C" fn(i32, i32, u32) -> i32 = __lpfx_simplex2_q32;
        let __lpfx_simplex3_f32_fn: extern "C" fn(f32, f32, f32, u32) -> f32 = __lpfx_simplex3_f32;
        let __lpfx_simplex3_q32_fn: extern "C" fn(i32, i32, i32, u32) -> i32 = __lpfx_simplex3_q32;

        // Force these to be included by using them in a way that can't be optimized away
        // We'll use volatile reads to prevent optimization
        let _ = core::ptr::read_volatile(&_q32_acos_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_acosh_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_add_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_asin_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_asinh_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_atan_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_atan2_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_atanh_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_cos_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_cosh_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_div_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_exp_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_exp2_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_fma_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_inversesqrt_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_ldexp_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_log_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_log2_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_mod_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_mul_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_pow_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_round_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_roundeven_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_sin_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_sinh_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_sqrt_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_sub_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_tan_fn as *const _);
        let _ = core::ptr::read_volatile(&_q32_tanh_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_hash_1_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_hash_2_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_hash_3_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_simplex1_f32_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_simplex1_q32_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_simplex2_f32_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_simplex2_q32_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_simplex3_f32_fn as *const _);
        let _ = core::ptr::read_volatile(&__lpfx_simplex3_q32_fn as *const _);
    }
}
