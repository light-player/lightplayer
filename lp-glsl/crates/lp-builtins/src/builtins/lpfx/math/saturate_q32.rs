//! Saturate function - clamp values between 0 and 1.
//!
//! This function clamps values to the [0, 1] range, which is commonly used in color
//! space conversions and other graphics operations.

use crate::util::q32::Q32;
use crate::util::vec3_q32::Vec3Q32;
use crate::util::vec4_q32::Vec4Q32;

/// Saturate a single Q32 value (clamp to [0, 1]).
///
/// # Arguments
/// * `value` - Value to saturate
///
/// # Returns
/// Value clamped between 0 and 1
#[inline(always)]
pub fn lpfx_saturate_q32(value: Q32) -> Q32 {
    value.clamp(Q32::ZERO, Q32::ONE)
}

/// Saturate each component of a Vec3Q32 (clamp to [0, 1]).
///
/// # Arguments
/// * `v` - Vector to saturate
///
/// # Returns
/// Vector with each component clamped between 0 and 1
#[inline(always)]
pub fn lpfx_saturate_vec3_q32(v: Vec3Q32) -> Vec3Q32 {
    Vec3Q32::new(
        lpfx_saturate_q32(v.x),
        lpfx_saturate_q32(v.y),
        lpfx_saturate_q32(v.z),
    )
}

/// Saturate each component of a Vec4Q32 (clamp to [0, 1]).
///
/// # Arguments
/// * `v` - Vector to saturate
///
/// # Returns
/// Vector with each component clamped between 0 and 1
#[inline(always)]
pub fn lpfx_saturate_vec4_q32(v: Vec4Q32) -> Vec4Q32 {
    Vec4Q32::new(
        lpfx_saturate_q32(v.x),
        lpfx_saturate_q32(v.y),
        lpfx_saturate_q32(v.z),
        lpfx_saturate_q32(v.w),
    )
}

/// Saturate function for Q32 (extern C wrapper for compiler).
///
/// # Arguments
/// * `value` - Value to saturate as i32 (Q32 fixed-point)
///
/// # Returns
/// Value clamped between 0 and 1 as i32 (Q32 fixed-point)
#[lpfx_impl_macro::lpfx_impl(q32, "float lpfx_saturate(float x)")]
#[unsafe(no_mangle)]
pub extern "C" fn __lpfx_saturate_q32(value: i32) -> i32 {
    lpfx_saturate_q32(Q32::from_fixed(value)).to_fixed()
}

/// Saturate function for vec3 (extern C wrapper for compiler).
///
/// # Arguments
/// * `x` - X component as i32 (Q32 fixed-point)
/// * `y` - Y component as i32 (Q32 fixed-point)
/// * `z` - Z component as i32 (Q32 fixed-point)
///
/// # Returns
/// X component as i32 (Q32 fixed-point)
#[lpfx_impl_macro::lpfx_impl(q32, "vec3 lpfx_saturate(vec3 v)")]
#[unsafe(no_mangle)]
pub extern "C" fn __lpfx_saturate_vec3_q32(x: i32, y: i32, z: i32) -> i32 {
    let v = Vec3Q32::new(Q32::from_fixed(x), Q32::from_fixed(y), Q32::from_fixed(z));
    let result = lpfx_saturate_vec3_q32(v);
    result.x.to_fixed()
}

/// Saturate function for vec4 (extern C wrapper for compiler).
///
/// # Arguments
/// * `x` - X component as i32 (Q32 fixed-point)
/// * `y` - Y component as i32 (Q32 fixed-point)
/// * `z` - Z component as i32 (Q32 fixed-point)
/// * `w` - W component as i32 (Q32 fixed-point)
///
/// # Returns
/// X component as i32 (Q32 fixed-point)
#[lpfx_impl_macro::lpfx_impl(q32, "vec4 lpfx_saturate(vec4 v)")]
#[unsafe(no_mangle)]
pub extern "C" fn __lpfx_saturate_vec4_q32(x: i32, y: i32, z: i32, w: i32) -> i32 {
    let v = Vec4Q32::new(
        Q32::from_fixed(x),
        Q32::from_fixed(y),
        Q32::from_fixed(z),
        Q32::from_fixed(w),
    );
    let result = lpfx_saturate_vec4_q32(v);
    result.x.to_fixed()
}
