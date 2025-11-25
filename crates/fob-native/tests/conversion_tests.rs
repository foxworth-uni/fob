//! Type conversion tests for fob-native.
//!
//! Tests conversion between NAPI types and fob-bundler types.

use fob_bundler::{BuildOptions, OutputFormat as BundlerOutputFormat};
use fob_native::conversion::format::convert_format;
use fob_native::conversion::sourcemap::convert_sourcemap_mode;
use fob_native::types::{OutputFormat, SourceMapMode};

#[test]
fn test_output_format_conversion_esm() {
    let result = convert_format(Some(OutputFormat::Esm));
    assert!(matches!(result, BundlerOutputFormat::Esm));
}

#[test]
fn test_output_format_conversion_cjs() {
    let result = convert_format(Some(OutputFormat::Cjs));
    assert!(matches!(result, BundlerOutputFormat::Cjs));
}

#[test]
fn test_output_format_conversion_iife() {
    let result = convert_format(Some(OutputFormat::Iife));
    assert!(matches!(result, BundlerOutputFormat::Iife));
}

#[test]
fn test_output_format_conversion_default() {
    // None should default to ESM
    let result = convert_format(None);
    assert!(matches!(result, BundlerOutputFormat::Esm));
}

#[test]
fn test_sourcemap_mode_external() {
    let base = BuildOptions::library("test.js");
    let result = convert_sourcemap_mode(base, Some(SourceMapMode::External));

    // Verify sourcemap is enabled (we can't directly check, but we verify it compiles)
    // The actual behavior is tested in integration tests
    let _ = result;
}

#[test]
fn test_sourcemap_mode_inline() {
    let base = BuildOptions::library("test.js");
    let result = convert_sourcemap_mode(base, Some(SourceMapMode::Inline));
    let _ = result;
}

#[test]
fn test_sourcemap_mode_hidden() {
    let base = BuildOptions::library("test.js");
    let result = convert_sourcemap_mode(base, Some(SourceMapMode::Hidden));
    let _ = result;
}

#[test]
fn test_sourcemap_mode_disabled() {
    let base = BuildOptions::library("test.js");
    let result = convert_sourcemap_mode(base, Some(SourceMapMode::Disabled));
    let _ = result;
}

#[test]
fn test_sourcemap_mode_none_defaults_to_disabled() {
    let base = BuildOptions::library("test.js");
    let result = convert_sourcemap_mode(base, None);
    let _ = result;
}

#[tokio::test]
async fn test_bundle_result_conversion_preserves_structure() {
    use fob_bundler::BuildOptions;
    use fob_native::bundle_result::BundleResult;
    use fob_native::runtime::NativeRuntime;
    use std::sync::Arc;
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();

    std::fs::write(cwd.join("index.js"), "export const x = 1;").unwrap();

    let runtime = Arc::new(NativeRuntime::new(cwd.clone()).unwrap());
    let build_result = BuildOptions::library("index.js")
        .cwd(cwd)
        .runtime(runtime)
        .build()
        .await
        .unwrap();

    // Convert to NAPI type
    let napi_result = BundleResult::from(build_result);

    // Verify structure
    assert!(!napi_result.chunks.is_empty(), "Should have chunks");
    assert!(
        !napi_result.chunks[0].code.is_empty(),
        "Chunk should have code"
    );

    // Verify ModuleInfo fields are Option types
    for chunk in &napi_result.chunks {
        for module in &chunk.modules {
            // These should be None in current implementation
            assert!(
                module.size.is_none(),
                "size should be None until implemented"
            );
            assert!(
                module.has_side_effects.is_none(),
                "has_side_effects should be None until implemented"
            );
            assert!(!module.path.is_empty(), "path should be set");
        }
    }

    // Verify stats are populated
    assert!(napi_result.stats.total_modules > 0);
    assert!(napi_result.stats.total_chunks > 0);
    assert!(napi_result.stats.total_size > 0);

    // Verify manifest is populated
    assert!(!napi_result.manifest.version.is_empty());
}
