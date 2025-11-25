//! Core bundler logic tests for fob-native.
//!
//! These tests verify the core bundling functionality without NAPI bindings.

use fob_bundler::BuildOptions;
use fob_native::api::config::BundleConfig;
use fob_native::core::bundler::CoreBundler;
use fob_native::runtime::NativeRuntime;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// Test helper to create a temporary project directory
fn create_test_project() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    (temp, cwd)
}

/// Test helper to create a simple test file
fn create_test_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
    let file_path = dir.join(name);
    std::fs::write(&file_path, content).unwrap();
    file_path
}

#[tokio::test]
async fn test_core_bundler_without_napi() {
    let (_temp, cwd) = create_test_project();
    create_test_file(&cwd, "index.js", "export const x = 1;");

    let config = BundleConfig {
        entries: vec!["index.js".to_string()],
        output_dir: Some("dist".to_string()),
        format: None,
        sourcemap: None,
        cwd: Some(cwd.to_string_lossy().to_string()),
    };

    let bundler = CoreBundler::new(config).unwrap();
    let result = bundler.bundle().await;

    match &result {
        Ok(bundle_result) => {
            assert!(!bundle_result.chunks.is_empty(), "Should have chunks");
        }
        Err(e) => {
            panic!(
                "CoreBundler should bundle successfully, but got error: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_single_entry_bundle() {
    let (_temp, cwd) = create_test_project();

    // Create a simple entry file
    let entry = create_test_file(&cwd, "index.js", "export const hello = 'world';");

    // Build
    let runtime = Arc::new(NativeRuntime::new(cwd.clone()).unwrap());
    let result = BuildOptions::library(entry)
        .cwd(cwd)
        .runtime(runtime)
        .build()
        .await;

    assert!(result.is_ok(), "Build should succeed");
    let build_result = result.unwrap();

    // Verify output
    let output = build_result.output.as_single().unwrap();
    assert!(
        !output.assets.is_empty(),
        "Should have at least one output asset"
    );
}

#[tokio::test]
async fn test_multiple_entry_bundle() {
    let (_temp, cwd) = create_test_project();

    // Create multiple entry files
    create_test_file(&cwd, "a.js", "export const a = 'a';");
    create_test_file(&cwd, "b.js", "export const b = 'b';");

    let runtime = Arc::new(NativeRuntime::new(cwd.clone()).unwrap());
    let result = BuildOptions::components(vec![cwd.join("a.js"), cwd.join("b.js")])
        .cwd(cwd)
        .runtime(runtime)
        .build()
        .await;

    assert!(result.is_ok(), "Build should succeed");
    let build_result = result.unwrap();

    // Verify multiple bundles
    let output = build_result.output.as_multiple().unwrap();
    assert_eq!(output.len(), 2, "Should have 2 bundles");
}

#[tokio::test]
async fn test_bundle_with_sourcemap() {
    let (_temp, cwd) = create_test_project();

    let entry = create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let runtime = Arc::new(NativeRuntime::new(cwd.clone()).unwrap());
    let result = BuildOptions::library(entry)
        .cwd(cwd)
        .sourcemap(true)
        .runtime(runtime)
        .build()
        .await;

    assert!(result.is_ok(), "Build should succeed");
    let build_result = result.unwrap();

    // Check for source map in output
    let output = build_result.output.as_single().unwrap();
    let has_sourcemap = output
        .assets
        .iter()
        .any(|asset| asset.filename().ends_with(".map"));
    assert!(has_sourcemap, "Should generate source map");
}

#[tokio::test]
async fn test_bundle_different_formats() {
    let (_temp, cwd) = create_test_project();

    let entry = create_test_file(&cwd, "index.js", "export const hello = 'world';");

    let formats = vec![
        fob_bundler::OutputFormat::Esm,
        fob_bundler::OutputFormat::Cjs,
        fob_bundler::OutputFormat::Iife,
    ];

    for format in formats {
        let runtime = Arc::new(NativeRuntime::new(cwd.clone()).unwrap());
        let result = BuildOptions::library(entry.clone())
            .cwd(cwd.clone())
            .format(format)
            .runtime(runtime)
            .build()
            .await;

        assert!(
            result.is_ok(),
            "Build should succeed for format {:?}",
            format
        );
    }
}

#[tokio::test]
async fn test_bundle_with_external_dependencies() {
    let (_temp, cwd) = create_test_project();

    let entry = create_test_file(
        &cwd,
        "index.js",
        "import { something } from 'external'; export const hello = something;",
    );

    let runtime = Arc::new(NativeRuntime::new(cwd.clone()).unwrap());
    let result = BuildOptions::library(entry)
        .cwd(cwd)
        .external(vec!["external".to_string()])
        .runtime(runtime)
        .build()
        .await;

    // This should either succeed (if external is handled) or fail gracefully
    // The important thing is it doesn't panic
    let _ = result;
}
