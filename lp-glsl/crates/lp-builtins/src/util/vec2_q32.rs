use core::ops::{Add, Div, Mul, Neg, Sub};

use super::q32::Q32;
use crate::builtins::q32::__lp_q32_sqrt;

/// 2D vector for Q32 fixed-point arithmetic
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vec2Q32 {
    pub x: Q32,
    pub y: Q32,
}

impl Vec2Q32 {
    #[inline(always)]
    pub const fn new(x: Q32, y: Q32) -> Self {
        Vec2Q32 { x, y }
    }

    #[inline(always)]
    pub fn from_f32(x: f32, y: f32) -> Self {
        Vec2Q32 {
            x: Q32::from_f32(x),
            y: Q32::from_f32(y),
        }
    }

    #[inline(always)]
    pub fn from_i32(x: i32, y: i32) -> Self {
        Vec2Q32 {
            x: Q32::from_i32(x),
            y: Q32::from_i32(y),
        }
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Vec2Q32::new(Q32::ZERO, Q32::ZERO)
    }

    #[inline(always)]
    pub const fn one() -> Self {
        Vec2Q32::new(Q32::ONE, Q32::ONE)
    }

    /// Dot product
    #[inline(always)]
    pub fn dot(self, rhs: Self) -> Q32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    /// Cross product (returns scalar in 2D, representing z-component of 3D cross product)
    #[inline(always)]
    pub fn cross(self, rhs: Self) -> Q32 {
        (self.x * rhs.y) - (self.y * rhs.x)
    }

    /// Length squared (avoids sqrt)
    #[inline(always)]
    pub fn length_squared(self) -> Q32 {
        self.dot(self)
    }

    /// Length (magnitude)
    #[inline(always)]
    pub fn length(self) -> Q32 {
        let len_sq = self.length_squared();
        Q32::from_fixed(__lp_q32_sqrt(len_sq.to_fixed()))
    }

    /// Distance to another vector
    #[inline(always)]
    pub fn distance(self, other: Self) -> Q32 {
        (self - other).length()
    }

    /// Normalize to unit vector
    #[inline(always)]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len.to_fixed() == 0 {
            return Vec2Q32::zero();
        }
        self / len
    }

    // Swizzle accessors (GLSL-style)
    #[inline(always)]
    pub fn x(self) -> Q32 {
        self.x
    }

    #[inline(always)]
    pub fn y(self) -> Q32 {
        self.y
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
    pub fn s(self) -> Q32 {
        self.x
    }

    #[inline(always)]
    pub fn t(self) -> Q32 {
        self.y
    }

    // 2-component swizzles (most common)
    #[inline(always)]
    pub fn xx(self) -> Vec2Q32 {
        Vec2Q32::new(self.x, self.x)
    }

    #[inline(always)]
    pub fn xy(self) -> Vec2Q32 {
        self
    }

    // identity
    #[inline(always)]
    pub fn yx(self) -> Vec2Q32 {
        Vec2Q32::new(self.y, self.x)
    }

    #[inline(always)]
    pub fn yy(self) -> Vec2Q32 {
        Vec2Q32::new(self.y, self.y)
    }

    // RGBA variants
    #[inline(always)]
    pub fn rr(self) -> Vec2Q32 {
        self.xx()
    }

    #[inline(always)]
    pub fn rg(self) -> Vec2Q32 {
        self.xy()
    }

    #[inline(always)]
    pub fn gr(self) -> Vec2Q32 {
        self.yx()
    }

    #[inline(always)]
    pub fn gg(self) -> Vec2Q32 {
        self.yy()
    }

    // STPQ variants
    #[inline(always)]
    pub fn ss(self) -> Vec2Q32 {
        self.xx()
    }

    #[inline(always)]
    pub fn st(self) -> Vec2Q32 {
        self.xy()
    }

    #[inline(always)]
    pub fn ts(self) -> Vec2Q32 {
        self.yx()
    }

    #[inline(always)]
    pub fn tt(self) -> Vec2Q32 {
        self.yy()
    }

    /// Component-wise multiply
    #[inline(always)]
    pub fn mul_comp(self, rhs: Self) -> Self {
        Vec2Q32::new(self.x * rhs.x, self.y * rhs.y)
    }

    /// Component-wise divide
    #[inline(always)]
    pub fn div_comp(self, rhs: Self) -> Self {
        Vec2Q32::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Add for Vec2Q32 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Vec2Q32::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2Q32 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Vec2Q32::new(self.x - rhs.x, self.y - rhs.y)
    }
}

// Vector * Scalar
impl Mul<Q32> for Vec2Q32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Q32) -> Self {
        Vec2Q32::new(self.x * rhs, self.y * rhs)
    }
}

// Vector / Scalar
impl Div<Q32> for Vec2Q32 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Q32) -> Self {
        Vec2Q32::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2Q32 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Vec2Q32::new(-self.x, -self.y)
    }
}
