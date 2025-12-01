//! Security tests for fob-native.
//!
//! Tests path validation and directory traversal protection.

use fob_native::BundleConfig;
use fob_native::Fob;
use fob_native::types::OutputFormat;
use tempfile::TempDir;

#[tokio::test]
async fn test_path_validation_prevents_directory_traversal() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();

    // Create a test entry file
    std::fs::write(cwd.join("index.js"), "export const hello = 'world';").unwrap();

    // Try to use directory traversal in output_dir
    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("../../../etc".to_string()), // Directory traversal attempt
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd.to_string_lossy().to_string()),
        external: None,
        minify: None,
        platform: None,
    };

    let bundler = Fob::new(config);

    // This should fail with validation error
    if let Ok(bundler) = bundler {
        let result = bundler.bundle().await;
        assert!(
            result.is_err(),
            "Should reject directory traversal in output_dir"
        );

        let error_str = result.err().unwrap().to_string();
        assert!(
            error_str.contains("outside project directory")
                || error_str.contains("directory traversal"),
            "Error should mention directory traversal prevention"
        );
    }
}

#[tokio::test]
async fn test_path_validation_prevents_traversal_in_entry() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();

    // Try to use directory traversal in entry path
    let config = BundleConfig {
        entries: vec!["../../../../etc/passwd".to_string()], // Directory traversal attempt
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd.to_string_lossy().to_string()),
        external: None,
        minify: None,
        platform: None,
    };

    let bundler = Fob::new(config);

    // This should fail with validation error
    if let Ok(bundler) = bundler {
        let result = bundler.bundle().await;
        assert!(
            result.is_err(),
            "Should reject directory traversal in entry path"
        );

        let error_str = result.err().unwrap().to_string();
        assert!(
            error_str.contains("outside project directory")
                || error_str.contains("directory traversal"),
            "Error should mention directory traversal prevention"
        );
    }
}

#[tokio::test]
async fn test_path_validation_allows_valid_relative_paths() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();

    // Create a test entry file
    std::fs::create_dir_all(cwd.join("src")).unwrap();
    std::fs::write(cwd.join("src/index.js"), "export const hello = 'world';").unwrap();

    // Use valid relative paths
    let config = BundleConfig {
        entries: vec!["src/index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd.to_string_lossy().to_string()),
        external: None,
        minify: None,
        platform: None,
    };

    let bundler = Fob::new(config);
    assert!(bundler.is_ok(), "Should accept valid relative paths");
}

#[tokio::test]
async fn test_path_validation_handles_absolute_paths() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();

    // Create a test entry file
    let entry_path = cwd.join("index.js");
    std::fs::write(&entry_path, "export const hello = 'world';").unwrap();

    // Use absolute path within project
    let config = BundleConfig {
        entries: vec![entry_path.to_string_lossy().to_string()],
        output_dir: Some(cwd.join("dist").to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd.to_string_lossy().to_string()),
        external: None,
        minify: None,
        platform: None,
    };

    let bundler = Fob::new(config);
    assert!(
        bundler.is_ok(),
        "Should accept absolute paths within project"
    );
}

#[tokio::test]
async fn test_path_validation_rejects_absolute_paths_outside_project() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();

    // Try to use absolute path outside project
    let outside_path = if cfg!(unix) {
        "/etc/passwd"
    } else {
        "C:\\Windows\\System32"
    };

    let config = BundleConfig {
        entries: vec![outside_path.to_string()],
        output_dir: Some("dist".to_string()),
        format: Some(OutputFormat::Esm),
        sourcemap: None,
        cwd: Some(cwd.to_string_lossy().to_string()),
        external: None,
        minify: None,
        platform: None,
    };

    let bundler = Fob::new(config);

    if let Ok(bundler) = bundler {
        let result = bundler.bundle().await;
        assert!(
            result.is_err(),
            "Should reject absolute paths outside project"
        );
    }
}
