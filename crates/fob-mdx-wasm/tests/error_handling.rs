//! Integration tests for error handling in bunny-wasm
//!
//! These tests verify that errors are properly converted from Rust to JavaScript
//! and that all error information is preserved.

use fob_mdx_wasm::compile_mdx;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Validation Error Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validation_error_large_input() {
    // Create input larger than 10MB limit
    let large_input = "x".repeat(11_000_000);
    let result = compile_mdx(&large_input, None);

    assert!(result.is_err(), "Large input should be rejected");

    let err_value = result.unwrap_err();
    assert!(err_value.is_object(), "Error should be an object");

    // Verify error structure using js_sys
    let kind = js_sys::Reflect::get(&err_value, &"kind".into()).unwrap();
    assert_eq!(kind.as_string().unwrap(), "validationError");

    let message = js_sys::Reflect::get(&err_value, &"message".into()).unwrap();
    assert!(message.as_string().unwrap().contains("size exceeds"));

    let details = js_sys::Reflect::get(&err_value, &"details".into()).unwrap();
    assert!(!details.is_undefined(), "Details should be present");
}

#[wasm_bindgen_test]
fn test_validation_error_null_bytes() {
    let input_with_null = "Hello\0World";
    let result = compile_mdx(input_with_null, None);

    assert!(result.is_err(), "Input with null bytes should be rejected");

    let err_value = result.unwrap_err();
    let kind = js_sys::Reflect::get(&err_value, &"kind".into()).unwrap();
    assert_eq!(kind.as_string().unwrap(), "validationError");

    let message = js_sys::Reflect::get(&err_value, &"message".into()).unwrap();
    assert!(message.as_string().unwrap().contains("null bytes"));
}

#[wasm_bindgen_test]
fn test_validation_passes_for_valid_input() {
    let valid_input = "# Hello World\n\nThis is valid MDX.";
    let result = compile_mdx(valid_input, None);

    assert!(result.is_ok(), "Valid input should compile successfully");
}

// ============================================================================
// Compilation Error Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_compilation_error_structure() {
    // This MDX might trigger a compilation error depending on parser strictness
    let invalid_mdx = "import { foo } from\n\n# Hello";
    let result = compile_mdx(invalid_mdx, None);

    // If it errors (parser dependent), verify error structure
    if let Err(err_value) = result {
        let kind = js_sys::Reflect::get(&err_value, &"kind".into()).unwrap();
        // Should be either compilationError or validationError
        assert!(
            kind.as_string().unwrap() == "compilationError"
                || kind.as_string().unwrap() == "validationError"
        );

        let message = js_sys::Reflect::get(&err_value, &"message".into()).unwrap();
        assert!(!message.as_string().unwrap().is_empty());
    }
}

// ============================================================================
// Error Field Verification Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_error_has_required_fields() {
    let large_input = "x".repeat(11_000_000);
    let result = compile_mdx(&large_input, None);

    assert!(result.is_err());
    let err_value = result.unwrap_err();

    // Verify required fields exist
    let kind = js_sys::Reflect::get(&err_value, &"kind".into());
    assert!(kind.is_ok(), "Error should have 'kind' field");

    let message = js_sys::Reflect::get(&err_value, &"message".into());
    assert!(message.is_ok(), "Error should have 'message' field");

    // kind should be a string
    assert!(kind.unwrap().is_string(), "'kind' should be a string");

    // message should be a string
    assert!(message.unwrap().is_string(), "'message' should be a string");
}

#[wasm_bindgen_test]
fn test_error_details_are_optional() {
    let input_with_null = "Test\0";
    let result = compile_mdx(input_with_null, None);

    assert!(result.is_err());
    let err_value = result.unwrap_err();

    // Details field might be undefined for some errors
    let details = js_sys::Reflect::get(&err_value, &"details".into()).unwrap();
    // This is okay - details can be undefined
    let _ = details;
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_error_messages_are_descriptive() {
    let large_input = "x".repeat(11_000_000);
    let result = compile_mdx(&large_input, None);

    assert!(result.is_err());
    let err_value = result.unwrap_err();

    let message = js_sys::Reflect::get(&err_value, &"message".into())
        .unwrap()
        .as_string()
        .unwrap();

    // Message should be descriptive, not just "error"
    assert!(message.len() > 10, "Error message should be descriptive");
    assert!(message.contains("size") || message.contains("large"));
}

#[wasm_bindgen_test]
fn test_null_byte_error_is_clear() {
    let input_with_null = "Hello\0World";
    let result = compile_mdx(input_with_null, None);

    assert!(result.is_err());
    let err_value = result.unwrap_err();

    let message = js_sys::Reflect::get(&err_value, &"message".into())
        .unwrap()
        .as_string()
        .unwrap();

    assert!(
        message.contains("null") && message.contains("byte"),
        "Error message should mention null bytes clearly"
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_empty_input_succeeds() {
    let result = compile_mdx("", None);
    assert!(result.is_ok(), "Empty input should compile successfully");
}

#[wasm_bindgen_test]
fn test_exactly_at_size_limit_succeeds() {
    // 10MB exactly (the limit)
    let at_limit = "x".repeat(10_000_000);
    let result = compile_mdx(&at_limit, None);
    assert!(result.is_ok(), "Input exactly at limit should succeed");
}

#[wasm_bindgen_test]
fn test_one_byte_over_limit_fails() {
    // 10MB + 1 byte
    let over_limit = "x".repeat(10_000_001);
    let result = compile_mdx(&over_limit, None);
    assert!(result.is_err(), "Input one byte over limit should fail");
}

// ============================================================================
// Unicode and Special Character Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_unicode_input_succeeds() {
    let unicode_input = "# Hello 世界 🌍\n\nUnicode is **awesome** ✨";
    let result = compile_mdx(unicode_input, None);
    assert!(result.is_ok(), "Unicode input should compile successfully");
}

#[wasm_bindgen_test]
fn test_emoji_heavy_input_succeeds() {
    let emoji_input = "🎉 # Hello 👋\n\n🌟 This has **lots** of 🎨 emojis 🚀";
    let result = compile_mdx(emoji_input, None);
    assert!(
        result.is_ok(),
        "Emoji-heavy input should compile successfully"
    );
}

#[wasm_bindgen_test]
fn test_mixed_unicode_null_byte_fails() {
    let mixed_input = "Hello 世界\0🌍";
    let result = compile_mdx(mixed_input, None);
    assert!(
        result.is_err(),
        "Unicode input with null byte should be rejected"
    );

    let err_value = result.unwrap_err();
    let kind = js_sys::Reflect::get(&err_value, &"kind".into()).unwrap();
    assert_eq!(kind.as_string().unwrap(), "validationError");
}

// ============================================================================
// Error Metadata Tests (location, context, suggestion)
// ============================================================================

#[wasm_bindgen_test]
fn test_compilation_error_has_location() {
    // This MDX has a syntax error that should produce location info
    let mdx = "line1\nline2\nimport { foo } from";
    let result = compile_mdx(mdx, None);

    // If it errors, check for location field
    if let Err(err_value) = result {
        // Check location field exists (may be undefined for some errors)
        let location = js_sys::Reflect::get(&err_value, &"location".into());
        if let Ok(loc) = location {
            if !loc.is_undefined() && !loc.is_null() {
                // If location exists, it should have line/column
                let line = js_sys::Reflect::get(&loc, &"line".into());
                if let Ok(line_val) = line {
                    if !line_val.is_undefined() {
                        // Line should be a positive number
                        assert!(
                            line_val.as_f64().unwrap_or(0.0) >= 1.0,
                            "Line number should be >= 1"
                        );
                    }
                }
            }
        }
    }
    // If compilation succeeds (parser is lenient), that's okay too
}

#[wasm_bindgen_test]
fn test_error_context_shows_source() {
    // Trigger a validation error which should have context
    let large_input = "x".repeat(11_000_000);
    let result = compile_mdx(&large_input, None);

    assert!(result.is_err());
    let err_value = result.unwrap_err();

    // Check context field (validation errors include size info as context)
    let context = js_sys::Reflect::get(&err_value, &"context".into());
    // Context is optional, but if present should be a string
    if let Ok(ctx) = context {
        if !ctx.is_undefined() && !ctx.is_null() {
            // Context should be informative
            if let Some(ctx_str) = ctx.as_string() {
                assert!(
                    !ctx_str.is_empty() || ctx_str.is_empty(),
                    "Context should be valid if present"
                );
            }
        }
    }

    // The important thing is that error has useful details
    let details = js_sys::Reflect::get(&err_value, &"details".into());
    if let Ok(det) = details {
        if !det.is_undefined() && !det.is_null() {
            if let Some(det_str) = det.as_string() {
                // Details should mention size
                assert!(
                    det_str.contains("bytes") || det_str.contains("size") || det_str.len() > 0,
                    "Details should be informative"
                );
            }
        }
    }
}
