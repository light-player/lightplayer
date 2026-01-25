//! LPFX function registry
//!
//! This module contains the const array of all LPFX functions.
//! This will be codegen output in the future, but for now is manually maintained.

use super::lpfx_fn::{LpfxFn, LpfxFnImpl};

/// Registry of all LPFX functions
///
/// This is the single source of truth for all LPFX function definitions.
/// Functions are looked up by name from this array.
pub const LPFX_FNS: &[LpfxFn] = &[];
