//! Integration tests for bundle writing functionality.
//!
//! These tests verify the complete write pipeline including:
//! - Directory creation
//! - Atomic writes with rollback
//! - Overwrite behavior
//! - Security (path traversal prevention)
//! - Error handling

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rolldown::BundleOutput;
use rolldown_common::{Output, OutputAsset};
use tempfile::TempDir;

use fob_bundler::output::writer::write_bundle_to;
use fob_bundler::Error;

/// Helper to create a mock BundleOutput for testing.
fn create_mock_bundle(assets: Vec<(&str, &str)>) -> BundleOutput {
    let outputs: Vec<Output> = assets
        .into_iter()
        .map(|(filename, content)| {
            let asset = OutputAsset {
                names: vec![],
                original_file_names: vec![],
                filename: filename.to_string().into(),
                source: content.as_bytes().to_vec().into(),
            };
            Output::Asset(Arc::new(asset))
        })
        .collect();

    BundleOutput {
        assets: outputs,
        warnings: Vec::new(),
    }
}

#[test]
fn test_write_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    let bundle = create_mock_bundle(vec![("bundle.js", "console.log('hello');")]);

    // Write should succeed
    write_bundle_to(&bundle, output_dir, false).unwrap();

    // Verify file exists and has correct content
    let content = fs::read_to_string(output_dir.join("bundle.js")).unwrap();
    assert_eq!(content, "console.log('hello');");
}

#[test]
fn test_write_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    let bundle = create_mock_bundle(vec![
        ("bundle.js", "console.log('hello');"),
        ("bundle.js.map", r#"{"version":3}"#),
        ("styles.css", "body { margin: 0; }"),
    ]);

    write_bundle_to(&bundle, output_dir, false).unwrap();

    // Verify all files exist
    assert!(output_dir.join("bundle.js").exists());
    assert!(output_dir.join("bundle.js.map").exists());
    assert!(output_dir.join("styles.css").exists());

    // Verify content
    let js_content = fs::read_to_string(output_dir.join("bundle.js")).unwrap();
    assert_eq!(js_content, "console.log('hello');");
}

#[test]
fn test_write_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    let bundle = create_mock_bundle(vec![
        ("assets/js/bundle.js", "console.log('nested');"),
        ("assets/css/styles.css", "body {}"),
    ]);

    write_bundle_to(&bundle, output_dir, false).unwrap();

    // Verify nested structure was created
    assert!(output_dir.join("assets/js/bundle.js").exists());
    assert!(output_dir.join("assets/css/styles.css").exists());

    let content = fs::read_to_string(output_dir.join("assets/js/bundle.js")).unwrap();
    assert_eq!(content, "console.log('nested');");
}

#[test]
fn test_write_auto_creates_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("dist");

    // Directory doesn't exist yet
    assert!(!output_dir.exists());

    let bundle = create_mock_bundle(vec![("bundle.js", "test")]);

    write_bundle_to(&bundle, &output_dir, false).unwrap();

    // Directory should be created
    assert!(output_dir.exists());
    assert!(output_dir.join("bundle.js").exists());
}

#[test]
fn test_write_fails_when_file_exists_and_no_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    // Create an existing file
    fs::write(output_dir.join("bundle.js"), "existing content").unwrap();

    let bundle = create_mock_bundle(vec![("bundle.js", "new content")]);

    // Write should fail because file exists
    let result = write_bundle_to(&bundle, output_dir, false);
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::OutputExists(msg) => {
            assert!(msg.contains("bundle.js"));
            assert!(msg.contains("already exists"));
        }
        _ => panic!("Expected OutputExists error"),
    }

    // Original file should be unchanged
    let content = fs::read_to_string(output_dir.join("bundle.js")).unwrap();
    assert_eq!(content, "existing content");
}

#[test]
fn test_write_overwrites_when_forced() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    // Create an existing file
    fs::write(output_dir.join("bundle.js"), "existing content").unwrap();

    let bundle = create_mock_bundle(vec![("bundle.js", "new content")]);

    // Write with overwrite=true should succeed
    write_bundle_to(&bundle, output_dir, true).unwrap();

    // File should be overwritten
    let content = fs::read_to_string(output_dir.join("bundle.js")).unwrap();
    assert_eq!(content, "new content");
}

#[test]
fn test_write_prevents_directory_traversal_simple() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("dist");

    let bundle = create_mock_bundle(vec![("../../../etc/passwd", "malicious")]);

    let result = write_bundle_to(&bundle, &output_dir, false);
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::InvalidOutputPath(msg) => {
            assert!(msg.contains("escapes output directory"));
        }
        _ => panic!("Expected InvalidOutputPath error"),
    }

    // Verify no file was written
    assert!(
        !PathBuf::from("/etc/passwd").exists()
            || fs::read_to_string("/etc/passwd").unwrap() != "malicious"
    );
}

#[test]
fn test_write_prevents_directory_traversal_complex() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("dist");

    let bundle = create_mock_bundle(vec![("safe/../../../etc/passwd", "malicious")]);

    let result = write_bundle_to(&bundle, &output_dir, false);
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::InvalidOutputPath(_) => {}
        _ => panic!("Expected InvalidOutputPath error"),
    }
}

#[test]
fn test_write_prevents_null_byte_in_filename() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    let bundle = create_mock_bundle(vec![("file\0name.js", "content")]);

    let result = write_bundle_to(&bundle, output_dir, false);
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::InvalidOutputPath(msg) => {
            assert!(msg.contains("null byte"));
        }
        _ => panic!("Expected InvalidOutputPath error"),
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_write_prevents_windows_device_names() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    let dangerous_names = vec!["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];

    for name in dangerous_names {
        let bundle = create_mock_bundle(vec![(name, "content")]);
        let result = write_bundle_to(&bundle, output_dir, false);
        assert!(result.is_err(), "Should reject device name: {}", name);
    }
}

#[test]
fn test_write_handles_dot_segments_safely() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    // These should be normalized and work fine
    let bundle = create_mock_bundle(vec![
        ("./bundle.js", "content1"),
        ("./assets/./styles.css", "content2"),
    ]);

    write_bundle_to(&bundle, output_dir, false).unwrap();

    assert!(output_dir.join("bundle.js").exists());
    assert!(output_dir.join("assets/styles.css").exists());
}

#[test]
fn test_atomic_write_rollback_on_failure() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    // Create a file that will conflict
    fs::write(output_dir.join("file2.js"), "existing").unwrap();

    let bundle = create_mock_bundle(vec![
        ("file1.js", "content1"),
        ("file2.js", "content2"),
        ("file3.js", "content3"),
    ]);

    // Write should fail due to file2.js existing
    let result = write_bundle_to(&bundle, output_dir, false);
    assert!(result.is_err());

    // file1.js should NOT exist (atomic rollback)
    assert!(!output_dir.join("file1.js").exists());

    // file2.js should still have original content
    let content = fs::read_to_string(output_dir.join("file2.js")).unwrap();
    assert_eq!(content, "existing");

    // file3.js should NOT exist
    assert!(!output_dir.join("file3.js").exists());
}

#[test]
fn test_write_empty_bundle() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    let bundle = create_mock_bundle(vec![]);

    // Should succeed even with no files
    write_bundle_to(&bundle, output_dir, false).unwrap();

    // Directory should be created
    assert!(output_dir.exists());
}

#[test]
fn test_write_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    // Create a large file (1 MB)
    let large_content = "x".repeat(1024 * 1024);
    let bundle = create_mock_bundle(vec![("large.js", &large_content)]);

    write_bundle_to(&bundle, output_dir, false).unwrap();

    // Verify full content was written
    let content = fs::read_to_string(output_dir.join("large.js")).unwrap();
    assert_eq!(content.len(), 1024 * 1024);
}

#[test]
fn test_write_special_characters_in_filename() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();

    // These characters should be allowed in filenames
    let bundle = create_mock_bundle(vec![
        ("file-with-dashes.js", "content1"),
        ("file_with_underscores.js", "content2"),
        ("file.with.dots.js", "content3"),
    ]);

    write_bundle_to(&bundle, output_dir, false).unwrap();

    assert!(output_dir.join("file-with-dashes.js").exists());
    assert!(output_dir.join("file_with_underscores.js").exists());
    assert!(output_dir.join("file.with.dots.js").exists());
}

#[test]
fn test_write_relative_output_path() {
    // Save current directory
    let current_dir = std::env::current_dir().unwrap();

    let temp_dir = TempDir::new().unwrap();

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let bundle = create_mock_bundle(vec![("bundle.js", "content")]);

    // Use relative path
    write_bundle_to(&bundle, Path::new("dist"), false).unwrap();

    // Verify file was written relative to current directory
    assert!(temp_dir.path().join("dist/bundle.js").exists());

    // Restore original directory
    std::env::set_current_dir(current_dir).unwrap();
}
