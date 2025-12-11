//! Shared test utilities for fob-bundler tests
//!
//! This module provides common helper functions used across test files
//! to reduce duplication and ensure consistent test patterns.

#![allow(dead_code)]

use fob_bundler::runtime::BundlerRuntime;
use fob_bundler::{BuildOptions, Platform, Runtime};
use fob_graph::runtime::native::NativeRuntime;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Get the path to the test fixtures directory
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Get the path to a specific fixture file
pub fn fixture_path(relative: &str) -> PathBuf {
    fixtures_dir().join(relative)
}

/// Create a BuildOptions with sensible defaults for testing
///
/// Uses library mode (externalize dependencies) and Node platform by default
/// since most tests are verifying transformation, not bundling.
pub fn test_build_options(entry: impl AsRef<Path>) -> BuildOptions {
    BuildOptions::new(entry.as_ref())
        .externalize_from("package.json") // Externalize all dependencies (library mode)
        .platform(Platform::Node)
        .cwd(env!("CARGO_MANIFEST_DIR"))
        .runtime(Arc::new(NativeRuntime))
}

/// Create a BundlerRuntime for testing with the given cwd
pub fn test_bundler_runtime(cwd: impl Into<PathBuf>) -> Arc<BundlerRuntime> {
    Arc::new(BundlerRuntime::new(cwd))
}

/// Create a NativeRuntime for testing
pub fn test_native_runtime() -> Arc<dyn Runtime> {
    Arc::new(NativeRuntime)
}

/// Assert that the first chunk contains a substring
pub fn assert_chunk_contains(result: &fob_bundler::BuildResult, substring: &str) {
    let chunk = result
        .chunks()
        .next()
        .expect("Should have at least one chunk");

    assert!(
        chunk.code.contains(substring),
        "Expected chunk to contain '{}', but it didn't.\nChunk preview (first 500 chars): {}",
        substring,
        &chunk.code[..chunk.code.len().min(500)]
    );
}

/// Assert that the first chunk does NOT contain a substring
pub fn assert_chunk_not_contains(result: &fob_bundler::BuildResult, substring: &str) {
    let chunk = result
        .chunks()
        .next()
        .expect("Should have at least one chunk");

    assert!(
        !chunk.code.contains(substring),
        "Expected chunk NOT to contain '{}', but it did.\nChunk preview (first 500 chars): {}",
        substring,
        &chunk.code[..chunk.code.len().min(500)]
    );
}

/// Assert that the build produced at least one asset
pub fn assert_has_assets(result: &fob_bundler::BuildResult) {
    let bundle = result.output.as_single().expect("single bundle");
    assert!(
        !bundle.assets.is_empty(),
        "Expected build to produce assets, but got none"
    );
}

/// Assert that the build failed with a specific error substring
pub fn assert_build_error_contains(
    result: fob_bundler::Result<fob_bundler::BuildResult>,
    substring: &str,
) {
    match result {
        Ok(_) => panic!("Expected build to fail, but it succeeded"),
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains(substring),
                "Expected error to contain '{}', but got: {}",
                substring,
                err_msg
            );
        }
    }
}
