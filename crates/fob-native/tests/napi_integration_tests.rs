//! NAPI integration tests for fob-native.
//!
//! These tests verify the actual NAPI bindings work correctly.

use fob_native::types::{OutputFormat, SourceMapMode};
use fob_native::{bundle_single, version, BundleConfig, Fob};
use tempfile::TempDir;

/// Test helper to create a temporary project directory
fn create_test_project() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_string_lossy().to_string();
    (temp, cwd)
}

/// Test helper to create a simple test file
fn create_test_file(dir: &str, name: &str, content: &str) -> String {
    let file_path = std::path::Path::new(dir).join(name);
    std::fs::write(&file_path, content).unwrap();
    file_path.to_string_lossy().to_string()
}

#[tokio::test]
async fn test_fob_constructor_with_valid_config() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: Some(SourceMapMode::External),
        cwd: Some(cwd),
    };

    let result = Fob::new(config);
    assert!(result.is_ok(), "Fob::new should succeed with valid config");
}

#[tokio::test]
async fn test_fob_constructor_rejects_empty_entries() {
    let (_temp, cwd) = create_test_project();

    let config = BundleConfig {
        entries: vec![],
        output_dir: Some("dist".to_string()),
        format: None,
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config);
    assert!(bundler.is_ok(), "Constructor should accept empty entries");

    // But bundle should fail
    let bundler = bundler.unwrap();
    let result = bundler.bundle().await;
    assert!(result.is_err(), "Bundle should fail with empty entries");

    // Verify error is JSON serializable
    let error_str = result.err().unwrap().to_string();
    assert!(
        error_str.contains("\"kind\":\"NoEntries\"")
            || error_str.contains("\"type\":\"NoEntries\"")
            || error_str.contains("NoEntries"),
        "Error should be JSON with NoEntries kind: {}",
        error_str
    );
}

#[tokio::test]
async fn test_fob_bundle_success() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: Some(SourceMapMode::External),
        cwd: Some(cwd.clone()),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(result.is_ok(), "Bundle should succeed");
    let bundle_result = result.unwrap();

    // Verify result structure
    assert!(
        !bundle_result.chunks.is_empty(),
        "Should have at least one chunk"
    );
    assert_eq!(bundle_result.chunks[0].kind, "entry");
    assert!(
        !bundle_result.chunks[0].code.is_empty(),
        "Chunk should have code"
    );
}

#[tokio::test]
async fn test_fob_bundle_with_all_formats() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let formats = vec![OutputFormat::Esm, OutputFormat::Cjs, OutputFormat::Iife];

    for format in formats {
        let config = BundleConfig {
            entries: vec!["index.js".to_string()],
            output_dir: Some("dist".to_string()),
            format: Some(format),
            sourcemap: Some(SourceMapMode::External),
            cwd: Some(cwd.clone()),
        };

        let bundler = Fob::new(config).unwrap();
        let result = bundler.bundle().await;

        assert!(result.is_ok(), "Bundle should succeed for format");
    }
}

#[tokio::test]
async fn test_fob_bundle_with_all_sourcemap_modes() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let modes = vec![
        SourceMapMode::External,
        SourceMapMode::Inline,
        SourceMapMode::Hidden,
        SourceMapMode::Disabled,
    ];

    for mode in modes {
        let config = BundleConfig {
            entries: vec!["index.js".to_string()],
            output_dir: Some("dist".to_string()),
            format: Some(OutputFormat::Esm),
            sourcemap: Some(mode),
            cwd: Some(cwd.clone()),
        };

        let bundler = Fob::new(config).unwrap();
        let result = bundler.bundle().await;

        assert!(result.is_ok(), "Bundle should succeed for sourcemap mode");
    }
}

#[tokio::test]
async fn test_fob_bundle_multiple_entries() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "a.js", "export const a = 'a';");
    create_test_file(&cwd, "b.js", "export const b = 'b';");

    let config = BundleConfig {
        entries: vec!["a.js".to_string(), "b.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: Some(SourceMapMode::External),
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(
        result.is_ok(),
        "Bundle should succeed with multiple entries"
    );
    let bundle_result = result.unwrap();
    assert!(
        bundle_result.chunks.len() >= 2,
        "Should have multiple chunks"
    );
}

#[tokio::test]
async fn test_bundle_single_function() {
    let (_temp, cwd) = create_test_project();
    let entry = create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let result = bundle_single(entry, cwd.clone() + "/dist", Some(OutputFormat::Esm)).await;

    match &result {
        Ok(bundle_result) => {
            assert!(!bundle_result.chunks.is_empty(), "Should have chunks");
        }
        Err(e) => {
            panic!("bundle_single should succeed, but got error: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_bundle_single_with_different_formats() {
    let (_temp, cwd) = create_test_project();
    let entry = create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let formats = vec![
        Some(OutputFormat::Esm),
        Some(OutputFormat::Cjs),
        Some(OutputFormat::Iife),
        None, // Should default to ESM
    ];

    for format in formats {
        let result = bundle_single(entry.clone(), cwd.clone() + "/dist", format).await;

        assert!(result.is_ok(), "bundle_single should succeed for format");
    }
}

#[test]
fn test_version_function() {
    let v = version();
    assert!(!v.is_empty(), "Version should not be empty");
    // Version should be semver-like
    assert!(
        v.chars().any(|c| c.is_ascii_digit()),
        "Version should contain digits"
    );
}

#[tokio::test]
async fn test_fob_bundle_error_serialization() {
    let (_temp, cwd) = create_test_project();

    // Try to bundle a non-existent file
    let config = BundleConfig {
        entries: vec!["nonexistent.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(result.is_err(), "Should fail with non-existent file");

    // Error should be JSON-serializable string
    let error_str = result.err().unwrap().to_string();

    // Should be valid JSON or at least contain error structure
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&error_str) {
        // If it's JSON, verify it has a kind or type field
        assert!(
            json.get("kind").is_some() || json.get("type").is_some(),
            "Error JSON should have 'kind' or 'type' field"
        );
    } else {
        // If not JSON, should at least contain error indicators
        assert!(
            error_str.contains("\"kind\"") ||  // JSON format has "kind" field
            error_str.contains("error") || 
            error_str.contains("Error") ||
            error_str.contains("failed") ||
            error_str.contains("Failed") ||
            error_str.contains("InvalidEntry"), // Specific error type
            "Error message should indicate failure: {}",
            error_str
        );
    }
}

#[tokio::test]
async fn test_fob_bundle_with_defaults() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "index.js", "export const hello = 'world';");

    // Test with minimal config (all defaults)
    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: None, // Should default to "dist"
        format: None,     // Should default to ESM
        sourcemap: None,  // Should default to disabled
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(result.is_ok(), "Bundle should succeed with defaults");
}
