//! LightPlayer Effects (LPFX) library native implementation
//!
//! Native implementation of the lpfx library used when running shaders on the cpu, providing
//! optimized implementations which are linked against.
//!
//! Writing these in rust provides two main advantages:
//! - rust compiler produces more optimized code
//! - lower memory footprint when compiling
//!

pub mod lpfx;
pub mod q32;
