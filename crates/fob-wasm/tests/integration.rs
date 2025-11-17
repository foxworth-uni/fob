//! Integration tests for fob-wasm
//!
//! These tests verify that fob-wasm can bundle real JavaScript, JSX, and MDX files.

use fob_bundler_wasm::{bundle, bundle_single, BundleConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Get the path to test fixtures
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

#[tokio::test]
async fn test_bundle_javascript() {
    let fixtures = fixtures_dir();
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("dist");

    let config = BundleConfig {
        entries: vec![fixtures.join("src/index.jsx").display().to_string()],
        output_dir: output_dir.display().to_string(),
        format: Some("esm".to_string()),
        sourcemap: Some(false),
    };

    let result = bundle(config).await;

    match &result {
        Ok(res) => {
            assert!(res.success, "Bundle should succeed");
            assert!(res.assets_count > 0, "Should generate at least one asset");
            println!("✓ Bundled {} assets", res.assets_count);
        }
        Err(e) => {
            eprintln!("Bundle error: {}", e);
            panic!("Bundle failed: {}", e);
        }
    }

    // Verify output directory was created
    assert!(output_dir.exists(), "Output directory should be created");

    // Check if any .js files were created
    let output_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "js")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !output_files.is_empty(),
        "Should generate at least one .js file"
    );

    println!("✓ Generated files:");
    for file in output_files {
        println!("  - {}", file.file_name().to_string_lossy());
    }
}

#[tokio::test]
async fn test_bundle_mdx() {
    let fixtures = fixtures_dir();
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("dist");

    // Bundle just the MDX file
    let result = bundle_single(
        fixtures.join("src/blog-post.mdx").display().to_string(),
        output_dir.display().to_string(),
        Some("esm"),
    )
    .await;

    match &result {
        Ok(res) => {
            assert!(res.success, "MDX bundle should succeed");
            assert!(res.assets_count > 0, "Should generate assets for MDX");
            println!("✓ MDX bundled successfully: {} assets", res.assets_count);
        }
        Err(e) => {
            eprintln!("MDX bundle error: {}", e);
            panic!("MDX bundle failed: {}", e);
        }
    }

    // Verify output exists
    assert!(output_dir.exists(), "Output directory should exist");

    // Read one of the output files to verify MDX was compiled
    if let Some(js_file) = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "js")
                .unwrap_or(false)
        })
    {
        let content = fs::read_to_string(js_file.path()).unwrap();

        // Verify MDX was compiled to JSX/JS
        // MDX compiler should generate JSX code
        assert!(
            content.len() > 0,
            "Generated JavaScript file should not be empty"
        );

        println!("✓ MDX compiled to JavaScript ({} bytes)", content.len());
        println!("  Preview: {}...", &content[..content.len().min(200)]);
    }
}

#[tokio::test]
async fn test_bundle_multiple_formats() {
    let fixtures = fixtures_dir();

    for format in &["esm", "cjs", "iife"] {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("dist");

        let config = BundleConfig {
            entries: vec![fixtures.join("src/component.jsx").display().to_string()],
            output_dir: output_dir.display().to_string(),
            format: Some(format.to_string()),
            sourcemap: Some(false),
        };

        let result = bundle(config).await;

        assert!(
            result.is_ok(),
            "Should bundle successfully in {} format",
            format
        );

        let res = result.unwrap();
        assert!(res.success, "{} format bundle should succeed", format);
        println!("✓ {} format: {} assets", format, res.assets_count);
    }
}

#[tokio::test]
async fn test_bundle_with_sourcemaps() {
    let fixtures = fixtures_dir();
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("dist");

    let config = BundleConfig {
        entries: vec![fixtures.join("src/index.jsx").display().to_string()],
        output_dir: output_dir.display().to_string(),
        format: Some("esm".to_string()),
        sourcemap: Some(true),
    };

    let result = bundle(config).await.unwrap();

    assert!(result.success, "Bundle with sourcemaps should succeed");

    // Check if sourcemap files were created
    let has_sourcemap = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "map")
                .unwrap_or(false)
        });

    assert!(
        has_sourcemap,
        "Should generate .map files when sourcemap=true"
    );
    println!("✓ Sourcemaps generated");
}

#[test]
fn test_bundle_config_serialization() {
    let config = BundleConfig {
        entries: vec!["./src/index.js".to_string()],
        output_dir: "./dist".to_string(),
        format: Some("esm".to_string()),
        sourcemap: Some(true),
    };

    // Test JSON serialization (for passing config from JS/TS)
    let json = serde_json::to_string(&config).unwrap();
    println!("Config JSON: {}", json);

    let deserialized: BundleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.entries, deserialized.entries);
    assert_eq!(config.format, deserialized.format);

    println!("✓ Config serialization works");
}
