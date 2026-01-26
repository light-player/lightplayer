use core::ops::{Add, Div, Mul, Neg, Sub};

use super::q32::Q32;
use super::vec2_q32::Vec2Q32;
use super::vec3_q32::Vec3Q32;
use crate::builtins::q32::__lp_q32_sqrt;

/// 4D vector for Q32 fixed-point arithmetic (useful for RGBA colors and homogeneous coordinates)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vec4Q32 {
    pub x: Q32,
    pub y: Q32,
    pub z: Q32,
    pub w: Q32,
}

impl Vec4Q32 {
    #[inline(always)]
    pub const fn new(x: Q32, y: Q32, z: Q32, w: Q32) -> Self {
        Vec4Q32 { x, y, z, w }
    }

    #[inline(always)]
    pub fn from_f32(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4Q32 {
            x: Q32::from_f32(x),
            y: Q32::from_f32(y),
            z: Q32::from_f32(z),
            w: Q32::from_f32(w),
        }
    }

    #[inline(always)]
    pub fn from_i32(x: i32, y: i32, z: i32, w: i32) -> Self {
        Vec4Q32 {
            x: Q32::from_i32(x),
            y: Q32::from_i32(y),
            z: Q32::from_i32(z),
            w: Q32::from_i32(w),
        }
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Vec4Q32::new(Q32::ZERO, Q32::ZERO, Q32::ZERO, Q32::ZERO)
    }

    #[inline(always)]
    pub const fn one() -> Self {
        Vec4Q32::new(Q32::ONE, Q32::ONE, Q32::ONE, Q32::ONE)
    }

    /// Dot product
    #[inline(always)]
    pub fn dot(self, rhs: Self) -> Q32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    /// Length squared (avoids sqrt)
    #[inline(always)]
    pub fn length_squared(self) -> Q32 {
        self.dot(self)
    }

    /// Length
    #[inline(always)]
    pub fn length(self) -> Q32 {
        let len_sq = self.length_squared();
        Q32::from_fixed(__lp_q32_sqrt(len_sq.to_fixed()))
    }

    /// Distance between two vectors
    #[inline(always)]
    pub fn distance(self, other: Self) -> Q32 {
        (self - other).length()
    }

    /// Normalize (returns zero vector if length is zero)
    #[inline(always)]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len.to_fixed() == 0 {
            return Vec4Q32::zero();
        }
        self / len
    }

    // Swizzle accessors (GLSL-style) - scalar
    #[inline(always)]
    pub fn x(self) -> Q32 {
        self.x
    }

    #[inline(always)]
    pub fn y(self) -> Q32 {
        self.y
    }

    #[inline(always)]
    pub fn z(self) -> Q32 {
        self.z
    }

    #[inline(always)]
    pub fn w(self) -> Q32 {
        self.w
    }

    #[inline(always)]
    pub fn r(self) -> Q32 {
        self.x
    }

    #[inline(always)]
    pub fn g(self) -> Q32 {
        self.y
    }

    #[inline(always)]
    pub fn b(self) -> Q32 {
        self.z
    }

    #[inline(always)]
    pub fn a(self) -> Q32 {
        self.w
    }

    // 2-component swizzles (most common)
    #[inline(always)]
    pub fn xy(self) -> Vec2Q32 {
        Vec2Q32::new(self.x, self.y)
    }

    #[inline(always)]
    pub fn xz(self) -> Vec2Q32 {
        Vec2Q32::new(self.x, self.z)
    }

    #[inline(always)]
    pub fn xw(self) -> Vec2Q32 {
        Vec2Q32::new(self.x, self.w)
    }

    #[inline(always)]
    pub fn yz(self) -> Vec2Q32 {
        Vec2Q32::new(self.y, self.z)
    }

    #[inline(always)]
    pub fn yw(self) -> Vec2Q32 {
        Vec2Q32::new(self.y, self.w)
    }

    #[inline(always)]
    pub fn zw(self) -> Vec2Q32 {
        Vec2Q32::new(self.z, self.w)
    }

    // 3-component swizzles (most common)
    #[inline(always)]
    pub fn xyz(self) -> Vec3Q32 {
        Vec3Q32::new(self.x, self.y, self.z)
    }

    #[inline(always)]
    pub fn xyw(self) -> Vec3Q32 {
        Vec3Q32::new(self.x, self.y, self.w)
    }

    #[inline(always)]
    pub fn xzw(self) -> Vec3Q32 {
        Vec3Q32::new(self.x, self.z, self.w)
    }

    #[inline(always)]
    pub fn yzw(self) -> Vec3Q32 {
        Vec3Q32::new(self.y, self.z, self.w)
    }

    // 4-component swizzle (identity)
    #[inline(always)]
    pub fn xyzw(self) -> Vec4Q32 {
        self
    }

    // RGBA variants
    #[inline(always)]
    pub fn rg(self) -> Vec2Q32 {
        self.xy()
    }

    #[inline(always)]
    pub fn rb(self) -> Vec2Q32 {
        self.xz()
    }

    #[inline(always)]
    pub fn rgb(self) -> Vec3Q32 {
        self.xyz()
    }

    #[inline(always)]
    pub fn rgba(self) -> Vec4Q32 {
        self
    }

    /// Component-wise multiply
    #[inline(always)]
    pub fn mul_comp(self, rhs: Self) -> Self {
        Vec4Q32::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
            self.w * rhs.w,
        )
    }

    /// Component-wise divide
    #[inline(always)]
    pub fn div_comp(self, rhs: Self) -> Self {
        Vec4Q32::new(
            self.x / rhs.x,
            self.y / rhs.y,
            self.z / rhs.z,
            self.w / rhs.w,
        )
    }

    /// Clamp components between min and max
    #[inline(always)]
    pub fn clamp(self, min: Q32, max: Q32) -> Self {
        Vec4Q32::new(
            self.x.clamp(min, max),
            self.y.clamp(min, max),
            self.z.clamp(min, max),
            self.w.clamp(min, max),
        )
    }
}

// Vector + Vector
impl Add for Vec4Q32 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Vec4Q32::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

// Vector - Vector
impl Sub for Vec4Q32 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Vec4Q32::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

// Vector * Scalar
impl Mul<Q32> for Vec4Q32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Q32) -> Self {
        Vec4Q32::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

// Vector / Scalar
impl Div<Q32> for Vec4Q32 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Q32) -> Self {
        Vec4Q32::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl Neg for Vec4Q32 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Vec4Q32::new(-self.x, -self.y, -self.z, -self.w)
    }
}
