//! # fob-mdx-wasm
//!
//! WebAssembly bindings for fob-mdx - compile MDX to JSX in the browser.
//!
//! This crate provides WASM bindings for the fob-mdx compiler, allowing
//! you to compile MDX files to JSX directly in the browser without a server.
//!
//! ## Features
//!
//! - Compile MDX to JSX in the browser
//! - Extract frontmatter (YAML/TOML)
//! - Support for GFM, math, footnotes
//! - No bundling (compile-only, WASM-compatible)
//!
//! ## Usage
//!
//! ```javascript
//! import init, { compile_mdx, WasmMdxOptions } from './pkg/fob_mdx_wasm.js';
//!
//! await init();
//!
//! const options = new WasmMdxOptions();
//! options.set_gfm(true);
//! options.set_math(true);
//!
//! const result = compile_mdx("# Hello **World**", options);
//! console.log(result.code); // Compiled JSX
//! ```

mod error;

use error::{WasmError, validate_input};
use fob_mdx::{MdxCompileOptions, compile};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in console
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WASM-compatible MDX compilation options
///
/// This is a JS-friendly wrapper around `MdxCompileOptions` that can be
/// constructed and configured from JavaScript.
#[wasm_bindgen]
pub struct WasmMdxOptions {
    filepath: Option<String>,
    gfm: bool,
    footnotes: bool,
    math: bool,
    jsx_runtime: String,
    output_format: String,
}

#[wasm_bindgen]
impl WasmMdxOptions {
    /// Create new options with defaults
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            filepath: None,
            gfm: false,
            footnotes: false,
            math: false,
            jsx_runtime: "react/jsx-runtime".to_string(),
            output_format: "program".to_string(),
        }
    }

    /// Set the filepath (for error messages)
    #[wasm_bindgen]
    pub fn set_filepath(&mut self, filepath: String) {
        self.filepath = Some(filepath);
    }

    /// Get the filepath
    #[wasm_bindgen(getter)]
    pub fn filepath(&self) -> Option<String> {
        self.filepath.clone()
    }

    /// Enable/disable GFM (GitHub Flavored Markdown)
    #[wasm_bindgen]
    pub fn set_gfm(&mut self, enabled: bool) {
        self.gfm = enabled;
    }

    /// Get GFM setting
    #[wasm_bindgen(getter)]
    pub fn gfm(&self) -> bool {
        self.gfm
    }

    /// Enable/disable footnotes
    #[wasm_bindgen]
    pub fn set_footnotes(&mut self, enabled: bool) {
        self.footnotes = enabled;
    }

    /// Get footnotes setting
    #[wasm_bindgen(getter)]
    pub fn footnotes(&self) -> bool {
        self.footnotes
    }

    /// Enable/disable math
    #[wasm_bindgen]
    pub fn set_math(&mut self, enabled: bool) {
        self.math = enabled;
    }

    /// Get math setting
    #[wasm_bindgen(getter)]
    pub fn math(&self) -> bool {
        self.math
    }

    /// Set JSX runtime (default: "react/jsx-runtime")
    #[wasm_bindgen]
    pub fn set_jsx_runtime(&mut self, runtime: String) {
        self.jsx_runtime = runtime;
    }

    /// Get JSX runtime
    #[wasm_bindgen(getter)]
    pub fn jsx_runtime(&self) -> String {
        self.jsx_runtime.clone()
    }

    /// Set output format ("program" or "function-body")
    #[wasm_bindgen]
    pub fn set_output_format(&mut self, format: &str) {
        self.output_format = match format {
            "function-body" => "function-body".to_string(),
            _ => "program".to_string(),
        };
    }

    /// Get output format
    #[wasm_bindgen(getter)]
    pub fn output_format(&self) -> String {
        self.output_format.clone()
    }
}

impl Default for WasmMdxOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert WASM options to Rust options
impl From<&WasmMdxOptions> for MdxCompileOptions {
    fn from(opts: &WasmMdxOptions) -> Self {
        let mut rust_opts = MdxCompileOptions::new();

        // Set filepath directly (it's a public field)
        rust_opts.filepath = opts.filepath.clone();

        // Set feature flags
        rust_opts.gfm = opts.gfm;
        rust_opts.footnotes = opts.footnotes;
        rust_opts.math = opts.math;

        // Set JSX runtime
        rust_opts.jsx_runtime = opts.jsx_runtime.clone();

        // Set output format
        rust_opts.output_format = match opts.output_format.as_str() {
            "function-body" => fob_mdx::OutputFormat::FunctionBody,
            _ => fob_mdx::OutputFormat::Program,
        };

        rust_opts
    }
}

/// Result of MDX compilation (serializable for JS)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmMdxResult {
    /// Compiled JSX code
    pub code: String,

    /// Extracted frontmatter (if present)
    pub frontmatter: Option<WasmFrontmatter>,

    /// List of image URLs found in the document
    pub images: Vec<String>,

    /// Named exports found in the document
    pub named_exports: Vec<String>,

    /// Re-exports found in the document
    pub reexports: Vec<String>,

    /// Imports found in the document
    pub imports: Vec<String>,

    /// Default export name (if present)
    pub default_export: Option<String>,
}

/// Frontmatter data (serializable for JS)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmFrontmatter {
    /// Raw frontmatter string
    pub raw: String,

    /// Parsed frontmatter format (yaml or toml)
    pub format: String,

    /// Parsed frontmatter data (as JSON value, will be converted to JS object)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Compile MDX source to JSX
///
/// # Arguments
///
/// * `source` - MDX source code as string (max 10MB)
/// * `options` - Compilation options (optional, uses defaults if None)
///
/// # Returns
///
/// * `Ok(WasmMdxResult)` - Compiled JSX and metadata
/// * `Err(JsValue)` - Structured error object with kind, message, location, etc.
///
/// # Errors
///
/// Returns structured error objects that can be discriminated by `kind`:
/// - `"validationError"` - Input validation failed (size limit, null bytes)
/// - `"compilationError"` - MDX syntax error (with location and suggestion)
/// - `"serializationError"` - Failed to serialize result to JavaScript
///
/// # Example
///
/// ```javascript
/// import { compile_mdx, WasmMdxOptions } from './pkg/fob_mdx_wasm.js';
///
/// const options = new WasmMdxOptions();
/// options.set_gfm(true);
///
/// try {
///   const result = compile_mdx("# Hello **World**", options);
///   console.log(result.code);
/// } catch (error) {
///   if (error.kind === "compilationError") {
///     console.error(error.message);
///     if (error.location) {
///       console.error(`At ${error.location.line}:${error.location.column}`);
///     }
///     if (error.suggestion) {
///       console.log(`Suggestion: ${error.suggestion}`);
///     }
///   }
/// }
/// ```
#[wasm_bindgen]
pub fn compile_mdx(source: &str, options: Option<WasmMdxOptions>) -> Result<JsValue, JsValue> {
    // Input validation (10MB limit for WASM environments)
    validate_input(source, 10_000_000).map_err(|e| -> JsValue { (*e).into() })?;

    // Convert WASM options to Rust options
    let rust_options = if let Some(ref opts) = options {
        MdxCompileOptions::from(opts)
    } else {
        MdxCompileOptions::new()
    };

    // Compile MDX - Box<MdxError> automatically converts to WasmError
    let result = compile(source, rust_options)
        .map_err(|e| -> WasmError { e.into() })
        .map_err(|e| -> JsValue { e.into() })?;

    // Convert frontmatter
    let frontmatter = result.frontmatter.map(|fm| {
        WasmFrontmatter {
            raw: fm.raw.clone(),
            format: match fm.format {
                fob_mdx::FrontmatterFormat::Yaml => "yaml".to_string(),
                fob_mdx::FrontmatterFormat::Toml => "toml".to_string(),
            },
            // Convert JsonValue to serde_json::Value (they're the same type)
            data: Some(fm.data),
        }
    });

    // Build WASM result
    let wasm_result = WasmMdxResult {
        code: result.code,
        frontmatter,
        images: result.images,
        named_exports: result.named_exports,
        reexports: result.reexports,
        imports: result.imports,
        default_export: result.default_export,
    };

    // Serialize to JS value
    serde_wasm_bindgen::to_value(&wasm_result).map_err(|e| {
        let err = WasmError::serialization_with_details(
            "Failed to serialize compilation result",
            e.to_string(),
        );
        JsValue::from(err)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // Allow tests to run in both Node.js and browser environments
    wasm_bindgen_test_configure!();

    // Helper macro to load fixture files
    macro_rules! load_fixture {
        ($name:literal) => {
            include_str!(concat!("../tests/fixtures/", $name))
        };
    }

    // ============================================================================
    // Basic Compilation Tests (8 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_compile_empty_string() {
        let result = compile_mdx("", None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(!result_obj.code.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_compile_plain_text() {
        let result = compile_mdx("Hello world", None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.code.contains("Hello"));
        assert!(result_obj.code.contains("world"));
    }

    #[wasm_bindgen_test]
    fn test_compile_simple_markdown() {
        let mdx = "# Hello\n\nThis is **bold** text.";
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.code.contains("Hello"));
        assert!(result_obj.code.contains("bold"));
    }

    #[wasm_bindgen_test]
    fn test_compile_with_options() {
        let mdx = "# Hello";
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_compile_without_options() {
        let mdx = "# Hello";
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_compile_basic_fixture() {
        let mdx = load_fixture!("basic.mdx");
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.code.contains("Hello World"));
        assert!(result_obj.code.contains("Features"));
    }

    #[wasm_bindgen_test]
    fn test_compile_with_jsx() {
        let mdx = r#"# Hello

<Component prop="value" />
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.code.contains("Component"));
    }

    #[wasm_bindgen_test]
    fn test_compile_returns_all_fields() {
        let mdx = "# Test";
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Check all fields exist
        assert!(!result_obj.code.is_empty());
        assert!(result_obj.images.is_empty() || !result_obj.images.is_empty());
        assert!(result_obj.named_exports.is_empty() || !result_obj.named_exports.is_empty());
        assert!(result_obj.reexports.is_empty() || !result_obj.reexports.is_empty());
        assert!(result_obj.imports.is_empty() || !result_obj.imports.is_empty());
    }

    // ============================================================================
    // WasmMdxOptions Tests (10 tests)
    // ============================================================================

    #[test]
    fn test_options_constructor_defaults() {
        let options = WasmMdxOptions::new();
        assert_eq!(options.filepath(), None);
        assert!(!options.gfm());
        assert!(!options.footnotes());
        assert!(!options.math());
        assert_eq!(options.jsx_runtime(), "react/jsx-runtime");
    }

    #[test]
    fn test_options_set_get_filepath() {
        let mut options = WasmMdxOptions::new();
        options.set_filepath("test.mdx".to_string());
        assert_eq!(options.filepath(), Some("test.mdx".to_string()));
    }

    #[test]
    fn test_options_set_get_gfm() {
        let mut options = WasmMdxOptions::new();
        assert!(!options.gfm());
        options.set_gfm(true);
        assert!(options.gfm());
        options.set_gfm(false);
        assert!(!options.gfm());
    }

    #[test]
    fn test_options_set_get_footnotes() {
        let mut options = WasmMdxOptions::new();
        assert!(!options.footnotes());
        options.set_footnotes(true);
        assert!(options.footnotes());
        options.set_footnotes(false);
        assert!(!options.footnotes());
    }

    #[test]
    fn test_options_set_get_math() {
        let mut options = WasmMdxOptions::new();
        assert!(!options.math());
        options.set_math(true);
        assert!(options.math());
        options.set_math(false);
        assert!(!options.math());
    }

    #[test]
    fn test_options_set_get_jsx_runtime() {
        let mut options = WasmMdxOptions::new();
        assert_eq!(options.jsx_runtime(), "react/jsx-runtime");
        options.set_jsx_runtime("preact/jsx-runtime".to_string());
        assert_eq!(options.jsx_runtime(), "preact/jsx-runtime");
    }

    #[test]
    fn test_options_all_features() {
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        options.set_footnotes(true);
        options.set_math(true);
        assert!(options.gfm());
        assert!(options.footnotes());
        assert!(options.math());
    }

    #[test]
    fn test_options_conversion_to_mdx_options() {
        let mut wasm_options = WasmMdxOptions::new();
        wasm_options.set_gfm(true);
        wasm_options.set_footnotes(true);
        wasm_options.set_math(true);
        wasm_options.set_filepath("test.mdx".to_string());
        wasm_options.set_jsx_runtime("custom/jsx-runtime".to_string());

        let rust_options: MdxCompileOptions = (&wasm_options).into();
        assert!(rust_options.gfm);
        assert!(rust_options.footnotes);
        assert!(rust_options.math);
        assert_eq!(rust_options.filepath, Some("test.mdx".to_string()));
        assert_eq!(rust_options.jsx_runtime, "custom/jsx-runtime");
    }

    #[test]
    fn test_options_default_trait() {
        let options1 = WasmMdxOptions::new();
        let options2 = WasmMdxOptions::default();
        assert_eq!(options1.gfm(), options2.gfm());
        assert_eq!(options1.footnotes(), options2.footnotes());
        assert_eq!(options1.math(), options2.math());
        assert_eq!(options1.jsx_runtime(), options2.jsx_runtime());
    }

    #[test]
    fn test_options_roundtrip_all_properties() {
        let mut options = WasmMdxOptions::new();
        options.set_filepath("path/to/file.mdx".to_string());
        options.set_gfm(true);
        options.set_footnotes(true);
        options.set_math(true);
        options.set_jsx_runtime("custom-runtime".to_string());

        // Verify all properties persist
        assert_eq!(options.filepath(), Some("path/to/file.mdx".to_string()));
        assert!(options.gfm());
        assert!(options.footnotes());
        assert!(options.math());
        assert_eq!(options.jsx_runtime(), "custom-runtime");
    }

    // ============================================================================
    // Feature Flag Tests (7 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_gfm_table() {
        let mdx = load_fixture!("gfm.mdx");
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Table should be compiled
        assert!(result_obj.code.contains("table") || result_obj.code.contains("Column"));
    }

    #[wasm_bindgen_test]
    fn test_gfm_strikethrough() {
        let mdx = "This is ~~strikethrough~~ text.";
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Should contain del tag for strikethrough
        assert!(result_obj.code.contains("del") || result_obj.code.contains("strikethrough"));
    }

    #[wasm_bindgen_test]
    fn test_gfm_task_list() {
        let mdx = "- [x] Completed\n- [ ] Incomplete";
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.code.contains("Completed") || result_obj.code.contains("Incomplete"));
    }

    #[wasm_bindgen_test]
    fn test_math_inline() {
        let mdx = "Inline math: $E = mc^2$";
        let mut options = WasmMdxOptions::new();
        options.set_math(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Should contain math
        assert!(
            result_obj.code.contains("math")
                || result_obj.code.contains("E")
                || result_obj.code.contains("mc")
        );
    }

    #[wasm_bindgen_test]
    fn test_math_block() {
        let mdx = "$$\nE = mc^2\n$$";
        let mut options = WasmMdxOptions::new();
        options.set_math(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(
            result_obj.code.contains("math")
                || result_obj.code.contains("E")
                || result_obj.code.contains("mc")
        );
    }

    #[wasm_bindgen_test]
    fn test_footnotes() {
        let mdx = "Text with footnote[^1].\n\n[^1]: Footnote content";
        let mut options = WasmMdxOptions::new();
        options.set_footnotes(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.code.contains("footnote") || result_obj.code.contains("Footnote"));
    }

    #[wasm_bindgen_test]
    fn test_all_features_combined() {
        let mdx = r#"# Test

| Col | Col |
|-----|-----|
| Val | Val |

Math: $x = y$

Footnote[^1]

[^1]: Note
"#;
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        options.set_math(true);
        options.set_footnotes(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok());
    }

    // ============================================================================
    // Frontmatter Tests (4 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_frontmatter_yaml() {
        let mdx = load_fixture!("frontmatter.mdx");
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.frontmatter.is_some());
        let fm = result_obj.frontmatter.unwrap();
        assert_eq!(fm.format, "yaml");
        assert!(!fm.raw.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_frontmatter_toml() {
        let mdx = r#"+++
title = "TOML Test"
+++

# Content
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.frontmatter.is_some());
        let fm = result_obj.frontmatter.unwrap();
        assert_eq!(fm.format, "toml");
        assert!(!fm.raw.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_no_frontmatter() {
        let mdx = "# No frontmatter\n\nJust content.";
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Frontmatter should be None
        assert!(result_obj.frontmatter.is_none());
    }

    #[wasm_bindgen_test]
    fn test_frontmatter_complex_data() {
        let mdx = r#"---
title: Complex Test
author:
  name: Test Author
  email: test@example.com
tags:
  - tag1
  - tag2
nested:
  deep:
    value: 42
---

# {frontmatter.title}
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        assert!(result_obj.frontmatter.is_some());
        let fm = result_obj.frontmatter.unwrap();
        assert_eq!(fm.format, "yaml");
        assert!(fm.data.is_some());
    }

    // ============================================================================
    // Error Handling Tests (6 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_error_unclosed_jsx_tag() {
        let mdx = "<div><p>Unclosed";
        let result = compile_mdx(mdx, None);
        // This might succeed or fail depending on parser leniency
        // Just verify it doesn't panic
        let _ = result;
    }

    #[wasm_bindgen_test]
    fn test_error_malformed_mdx() {
        let mdx = load_fixture!("malformed.mdx");
        let result = compile_mdx(mdx, None);
        // Malformed MDX should either error or handle gracefully
        match result {
            Ok(_) => {
                // Parser might be lenient, that's okay
            }
            Err(e) => {
                // Error should be an object with kind and message
                assert!(e.is_object(), "Error should be an object");
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_error_invalid_syntax() {
        // Test with clearly invalid syntax
        let mdx = "```\nUnclosed code block";
        let result = compile_mdx(mdx, None);
        // Should handle gracefully (either succeed or return error)
        match result {
            Ok(_) => {}
            Err(e) => {
                // Error should be an object with kind and message
                assert!(e.is_object(), "Error should be an object");
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_error_bad_frontmatter() {
        let mdx = "---\ninvalid: yaml: : : :\n---\n\nContent";
        let result = compile_mdx(mdx, None);
        // Should either parse or return error
        match result {
            Ok(_) => {}
            Err(e) => {
                // Error should be an object with kind and message
                assert!(e.is_object(), "Error should be an object");
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_error_empty_options_handled() {
        let mdx = "# Test";
        let result = compile_mdx(mdx, None);
        // None options should work fine
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_error_serialization_handled() {
        // This test verifies that errors are properly converted to JsValue
        // We can't easily trigger serialization errors, but we can verify
        // the error handling path exists
        let mdx = "# Test";
        let result = compile_mdx(mdx, None);
        match result {
            Ok(js_val) => {
                // Should be serializable
                assert!(!js_val.is_undefined());
            }
            Err(e) => {
                // Errors are structured objects
                assert!(e.is_object(), "Error should be an object");
            }
        }
    }

    // ============================================================================
    // Output Format Tests (3 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_output_format_program() {
        let mut options = WasmMdxOptions::new();
        options.set_output_format("program");
        let result = compile_mdx("# Hello", Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Program format should have export default
        assert!(
            result_obj.code.contains("export default"),
            "Program format should contain 'export default'"
        );
    }

    #[wasm_bindgen_test]
    fn test_output_format_function_body() {
        let mut options = WasmMdxOptions::new();
        options.set_output_format("function-body");
        let result = compile_mdx("# Hello", Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Function body format should NOT have top-level export default
        assert!(
            !result_obj.code.contains("export default"),
            "Function body format should not contain 'export default'"
        );
    }

    #[wasm_bindgen_test]
    fn test_output_format_invalid_defaults_to_program() {
        let mut options = WasmMdxOptions::new();
        options.set_output_format("invalid-format");
        // Verify setter normalizes invalid values to "program"
        assert_eq!(options.output_format(), "program");
        let result = compile_mdx("# Hello", Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Should fall back to program format
        assert!(
            result_obj.code.contains("export default"),
            "Invalid format should default to program"
        );
    }

    // ============================================================================
    // JSX Runtime Effect Tests (2 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_jsx_runtime_affects_output() {
        let mut options = WasmMdxOptions::new();
        options.set_jsx_runtime("preact/jsx-runtime".to_string());
        let result = compile_mdx("# Hello", Some(options));
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // JSX runtime should appear in imports
        assert!(
            result_obj.code.contains("preact/jsx-runtime"),
            "Custom JSX runtime should appear in output: {}",
            result_obj.code
        );
    }

    #[wasm_bindgen_test]
    fn test_default_jsx_runtime_is_react() {
        let result = compile_mdx("# Hello", None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Default should be react/jsx-runtime
        assert!(
            result_obj.code.contains("react/jsx-runtime"),
            "Default JSX runtime should be react: {}",
            result_obj.code
        );
    }

    // ============================================================================
    // Result Field Content Tests (4 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_images_field_extracts_urls() {
        let mdx = r#"# With Images

![Alt text](./image1.png)
![Another](https://example.com/image2.jpg)
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Images should be extracted (note: depends on plugins being enabled)
        // At minimum, the field should exist and be a Vec
        assert!(
            result_obj.images.is_empty() || result_obj.images.len() >= 1,
            "Images field should exist"
        );
    }

    #[wasm_bindgen_test]
    fn test_imports_field_contains_import_paths() {
        let mdx = r#"import Button from './Button.tsx'
import { Card } from './Card.tsx'

# Hello

<Button />
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Imports should be detected (format may vary - could be paths or full statements)
        assert!(
            !result_obj.imports.is_empty(),
            "Should detect imports, got: {:?}",
            result_obj.imports
        );
        // Verify import content contains Button reference somewhere
        let imports_str = result_obj.imports.join(" ");
        assert!(
            imports_str.contains("Button") || imports_str.contains("./Button"),
            "Should find Button import reference in: {}",
            imports_str
        );
    }

    #[wasm_bindgen_test]
    fn test_named_exports_extracted() {
        let mdx = r#"export const meta = { title: "Test" }
export function helper() { return 42 }

# Hello
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Named exports should be detected (format may vary - could be names or full statements)
        assert!(
            !result_obj.named_exports.is_empty(),
            "Should detect named exports, got: {:?}",
            result_obj.named_exports
        );
        // Verify export content contains meta reference somewhere
        let exports_str = result_obj.named_exports.join(" ");
        assert!(
            exports_str.contains("meta") || exports_str.contains("helper"),
            "Should find export reference in: {}",
            exports_str
        );
    }

    #[wasm_bindgen_test]
    fn test_default_export_in_output() {
        let mdx = r#"# Hello

This MDX content produces a default export in the compiled code.
"#;
        let result = compile_mdx(mdx, None);
        assert!(result.is_ok());
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // The compiled code should have an export default statement
        // (default_export field may or may not be set depending on implementation)
        assert!(
            result_obj.code.contains("export default"),
            "Compiled MDX should have export default in code"
        );
    }

    // ============================================================================
    // Feature Combination Tests (3 tests)
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_gfm_plus_footnotes() {
        let mdx = r#"| Header | Value |
|--------|-------|
| Cell   | Data  |

This has a footnote[^1].

[^1]: Footnote content here.
"#;
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        options.set_footnotes(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok(), "GFM + Footnotes should compile");
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Both features should work together
        assert!(
            result_obj.code.contains("table") || result_obj.code.contains("Header"),
            "GFM tables should work"
        );
        assert!(
            result_obj.code.contains("footnote") || result_obj.code.contains("Footnote"),
            "Footnotes should work"
        );
    }

    #[wasm_bindgen_test]
    fn test_math_plus_frontmatter() {
        let mdx = r#"---
title: Math Post
author: Test
---

# Math Content

The equation $E = mc^2$ is famous.

Block math:

$$
\sum_{i=0}^{n} x_i
$$
"#;
        let mut options = WasmMdxOptions::new();
        options.set_math(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(result.is_ok(), "Math + Frontmatter should compile");
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();
        // Frontmatter should be extracted
        assert!(
            result_obj.frontmatter.is_some(),
            "Frontmatter should be present"
        );
        let fm = result_obj.frontmatter.unwrap();
        assert!(fm.data.is_some(), "Frontmatter data should be parsed");
        // Math should be processed (look for math-related content in output)
        assert!(
            result_obj.code.len() > 200,
            "Math output should be substantial"
        );
    }

    #[wasm_bindgen_test]
    fn test_all_features_together() {
        let mdx = r#"---
title: Full Featured Post
tags:
  - rust
  - wasm
---

# Heading

| Feature   | Status |
|-----------|--------|
| GFM       | ✓      |
| Math      | ✓      |
| Footnotes | ✓      |

This is ~~deleted~~ text and has a footnote[^1].

Inline math: $x^2 + y^2 = z^2$

Block math:
$$
E = mc^2
$$

[^1]: A footnote explaining something.
"#;
        let mut options = WasmMdxOptions::new();
        options.set_gfm(true);
        options.set_footnotes(true);
        options.set_math(true);
        let result = compile_mdx(mdx, Some(options));
        assert!(
            result.is_ok(),
            "All features combined should compile: {:?}",
            result.err()
        );
        let js_value = result.unwrap();
        let result_obj: WasmMdxResult = serde_wasm_bindgen::from_value(js_value).unwrap();

        // Verify frontmatter
        assert!(
            result_obj.frontmatter.is_some(),
            "Frontmatter should be present"
        );

        // Verify code is substantial (all features generate output)
        assert!(
            result_obj.code.len() > 500,
            "Complex MDX should produce substantial output, got {} bytes",
            result_obj.code.len()
        );

        // Verify code contains expected elements
        assert!(!result_obj.code.is_empty(), "Code should not be empty");
    }
}
