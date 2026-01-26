//! Tests for GLSL signature parsing

use lp_builtin_gen::lpfx::glsl_parse::parse_glsl_signature;

#[test]
fn test_parse_simple_signature() {
    let sig = "u32 lpfx_hash1(u32 x, u32 seed)";
    let result = parse_glsl_signature(sig, "test_func", "test.rs");
    assert!(result.is_ok());

    let func_sig = result.unwrap();
    assert_eq!(func_sig.name, "lpfx_hash1");
    assert_eq!(func_sig.parameters.len(), 2);
}

#[test]
fn test_parse_vector_signature() {
    let sig = "float lpfx_simplex3(vec3 p, u32 seed)";
    let result = parse_glsl_signature(sig, "test_func", "test.rs");
    assert!(result.is_ok());

    let func_sig = result.unwrap();
    assert_eq!(func_sig.name, "lpfx_simplex3");
    assert_eq!(func_sig.parameters.len(), 2);
}

#[test]
fn test_parse_invalid_signature() {
    let sig = "invalid syntax here";
    let result = parse_glsl_signature(sig, "test_func", "test.rs");
    assert!(result.is_err());
}
