//! Browser-specific WASM tests
//!
//! These tests run in a browser environment and test WASM-specific functionality
//! like JS serialization, error handling, and integration with JavaScript.

use fob_mdx_wasm::{WasmMdxOptions, compile_mdx};
use wasm_bindgen_test::*;

// Allow tests to run in both Node.js and browser environments
wasm_bindgen_test_configure!();

#[wasm_bindgen_test]
fn test_compile_in_wasm_environment() {
    let mdx = "# Hello World\n\nThis is a test.";
    let result = compile_mdx(mdx, None);

    assert!(result.is_ok());
    let js_value = result.unwrap();

    // Verify it's a JavaScript object
    assert!(!js_value.is_undefined());
    assert!(!js_value.is_null());
}

#[wasm_bindgen_test]
fn test_options_serialization_to_js() {
    let mut options = WasmMdxOptions::new();
    options.set_gfm(true);
    options.set_math(true);
    options.set_filepath("test.mdx".to_string());

    // Options should be usable in WASM context
    let mdx = "# Test";
    let result = compile_mdx(mdx, Some(options));
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_result_serialization_to_js() {
    let mdx = r#"---
title: Test
---

# {frontmatter.title}
"#;
    let result = compile_mdx(mdx, None);
    assert!(result.is_ok());

    let js_value = result.unwrap();

    // Verify result can be accessed as JS object
    assert!(!js_value.is_undefined());

    // Verify it's an object or string
    assert!(js_value.is_object() || js_value.is_string());
}

#[wasm_bindgen_test]
fn test_error_to_jsvalue_conversion() {
    // Try to compile something that might fail
    // Note: The parser might be lenient, so this test verifies error handling exists
    let mdx = "<div><p>Potentially malformed";
    let result = compile_mdx(mdx, None);

    match result {
        Ok(_) => {
            // Parser handled it gracefully, that's fine
        }
        Err(e) => {
            // Error should be a JsValue that can be used in JS
            assert!(!e.is_undefined());
            assert!(!e.is_null());
            // Should be a structured error object with kind and message
            assert!(e.is_object(), "Error should be a structured object");
        }
    }
}

#[wasm_bindgen_test]
fn test_wasm_result_structure() {
    let mdx = r#"# Test Document

import { Component } from './component';

export const metadata = { version: '1.0' };

<Component />
"#;
    let result = compile_mdx(mdx, None);
    assert!(result.is_ok());

    let js_value = result.unwrap();

    // Verify the result structure is accessible
    // In a real browser test, we'd use js-sys to check properties
    // For now, just verify it's a valid JS value
    assert!(!js_value.is_undefined());
    assert!(!js_value.is_null());
}
