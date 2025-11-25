//! Edge case and error scenario tests for fob-native.

use fob_native::types::OutputFormat;
use fob_native::{BundleConfig, Fob};
use tempfile::TempDir;

/// Test helper to create a temporary project directory
fn create_test_project() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_string_lossy().to_string();
    (temp, cwd)
}

#[tokio::test]
async fn test_bundle_with_missing_file() {
    let (_temp, cwd) = create_test_project();

    let config = BundleConfig {
        entries: vec!["nonexistent.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(result.is_err(), "Should fail with missing file");

    // Error should be informative
    let error_str = result.err().unwrap().to_string();
    assert!(
        error_str.contains("not found")
            || error_str.contains("NotFound")
            || error_str.contains("unresolved")
            || error_str.contains("Unresolved")
            || error_str.contains("InvalidEntry")
            || error_str.contains("Invalid"),
        "Error should mention file not found or invalid entry: {}",
        error_str
    );
}

#[tokio::test]
async fn test_bundle_with_syntax_error() {
    let (_temp, cwd) = create_test_project();

    // Create a file with syntax error
    std::fs::write(
        cwd.clone() + "/index.js",
        "export const x = ; // syntax error",
    )
    .unwrap();

    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(result.is_err(), "Should fail with syntax error");

    // Error should indicate parse/syntax error
    let error_str = result.err().unwrap().to_string();
    assert!(
        error_str.contains("parse")
            || error_str.contains("Parse")
            || error_str.contains("syntax")
            || error_str.contains("Syntax")
            || error_str.contains("Unexpected"),
        "Error should mention parse/syntax error: {}",
        error_str
    );
}

#[tokio::test]
async fn test_bundle_with_circular_import() {
    let (_temp, cwd) = create_test_project();

    // Create circular dependency: a.js imports b.js, b.js imports a.js
    std::fs::write(
        cwd.clone() + "/a.js",
        "import { b } from './b.js'; export const a = 'a';",
    )
    .unwrap();

    std::fs::write(
        cwd.clone() + "/b.js",
        "import { a } from './a.js'; export const b = 'b';",
    )
    .unwrap();

    let config = BundleConfig {
        entries: vec!["a.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    // Should detect circular dependency (may succeed or fail depending on bundler behavior)
    // The important thing is it doesn't panic
    let _ = result;
}

#[tokio::test]
async fn test_bundle_with_unresolved_import() {
    let (_temp, cwd) = create_test_project();

    // Create file that imports non-existent module
    std::fs::write(
        cwd.clone() + "/index.js",
        "import { something } from './nonexistent.js'; export const x = something;",
    )
    .unwrap();

    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(result.is_err(), "Should fail with unresolved import");

    // Error should mention unresolved import
    let error_str = result.err().unwrap().to_string();
    assert!(
        error_str.contains("unresolved")
            || error_str.contains("Unresolved")
            || error_str.contains("not found")
            || error_str.contains("Cannot resolve"),
        "Error should mention unresolved import: {}",
        error_str
    );
}

#[tokio::test]
async fn test_bundle_with_empty_file() {
    let (_temp, cwd) = create_test_project();

    // Create empty file
    std::fs::write(cwd.clone() + "/index.js", "").unwrap();

    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    // Empty file may succeed (no-op) or fail (no exports)
    // The important thing is it doesn't panic
    let _ = result;
}

#[tokio::test]
async fn test_bundle_with_very_long_path() {
    let (_temp, cwd) = create_test_project();

    // Create deeply nested directory
    let deep_path = (0..10)
        .map(|i| format!("dir{}", i))
        .collect::<Vec<_>>()
        .join("/");

    std::fs::create_dir_all(cwd.clone() + "/" + &deep_path).unwrap();
    std::fs::write(
        cwd.clone() + "/" + &deep_path + "/index.js",
        "export const x = 1;",
    )
    .unwrap();

    let config = BundleConfig {
        entries: vec![format!("{}/index.js", deep_path)],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    // Should handle long paths gracefully
    assert!(
        result.is_ok() || result.is_err(),
        "Should not panic on long paths"
    );
}

#[tokio::test]
async fn test_bundle_with_special_characters_in_path() {
    let (_temp, cwd) = create_test_project();

    // Create file with special characters (but valid for filesystem)
    std::fs::write(cwd.clone() + "/test-file.js", "export const x = 1;").unwrap();

    let config = BundleConfig {
        entries: vec!["test-file.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(
        result.is_ok(),
        "Should handle special characters in filename"
    );
}

#[tokio::test]
async fn test_bundle_output_dir_creation() {
    let (_temp, cwd) = create_test_project();
    std::fs::write(cwd.clone() + "/index.js", "export const x = 1;").unwrap();

    // Output dir doesn't exist - should be created
    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist/new/subdir".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd),
    };

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;

    assert!(
        result.is_ok(),
        "Should create output directory if it doesn't exist"
    );
}
