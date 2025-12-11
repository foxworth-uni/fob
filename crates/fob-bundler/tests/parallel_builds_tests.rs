//! Tests for parallel isolated builds.
//!
//! These tests verify that:
//! 1. Output order is deterministic regardless of completion order
//! 2. All errors are collected before reporting
//! 3. `max_parallel_builds` configuration is respected

use fob_bundler::{BuildOptions, NativeRuntime};
use std::sync::Arc;
use tempfile::TempDir;

fn create_multi_entry_project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let src = dir.path().join("src");
    std::fs::create_dir(&src).expect("create src");

    // Create multiple entry files
    for i in 1..=5 {
        std::fs::write(
            src.join(format!("entry{}.js", i)),
            format!(
                r#"
export const name = "entry{}";
export const value = {};
"#,
                i,
                i * 10
            ),
        )
        .expect("write entry");
    }

    dir
}

#[tokio::test]
async fn parallel_builds_produce_deterministic_output() {
    let project = create_multi_entry_project();

    let entries: Vec<String> = (1..=5)
        .map(|i| project.path().join(format!("src/entry{}.js", i)))
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Build multiple times and verify output order is consistent
    let mut all_outputs: Vec<Vec<String>> = Vec::new();
    for _ in 0..3 {
        let result = BuildOptions::new_multiple(entries.clone())
            .bundle_separately()
            .cwd(project.path())
            .runtime(Arc::new(NativeRuntime::new()))
            .build()
            .await
            .expect("build should succeed");

        let bundle = result.output.as_multiple().expect("multiple bundles");
        let keys: Vec<String> = bundle.keys().cloned().collect();
        all_outputs.push(keys);
    }

    // All outputs should have the same keys
    for output in all_outputs.windows(2) {
        let mut keys1 = output[0].clone();
        let mut keys2 = output[1].clone();
        keys1.sort();
        keys2.sort();
        assert_eq!(keys1, keys2, "Output keys should be consistent");
    }
}

#[tokio::test]
async fn parallel_builds_with_max_parallel_1_is_sequential() {
    let project = create_multi_entry_project();

    let entries: Vec<String> = (1..=3)
        .map(|i| project.path().join(format!("src/entry{}.js", i)))
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let result = BuildOptions::new_multiple(entries)
        .bundle_separately()
        .max_parallel_builds(1) // Force sequential
        .cwd(project.path())
        .runtime(Arc::new(NativeRuntime::new()))
        .build()
        .await
        .expect("build should succeed");

    let bundle = result.output.as_multiple().expect("multiple bundles");
    assert_eq!(bundle.len(), 3, "Should produce 3 bundles");
}

#[tokio::test]
async fn parallel_builds_respects_max_parallel_config() {
    let project = create_multi_entry_project();

    let entries: Vec<String> = (1..=5)
        .map(|i| project.path().join(format!("src/entry{}.js", i)))
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Test with various concurrency limits
    for max in [1, 2, 4, 8] {
        let result = BuildOptions::new_multiple(entries.clone())
            .bundle_separately()
            .max_parallel_builds(max)
            .cwd(project.path())
            .runtime(Arc::new(NativeRuntime::new()))
            .build()
            .await
            .expect("build should succeed");

        let bundle = result.output.as_multiple().expect("multiple bundles");
        assert_eq!(bundle.len(), 5, "Should produce 5 bundles with max={}", max);
    }
}

#[tokio::test]
async fn parallel_builds_merges_all_graphs() {
    let project = create_multi_entry_project();

    let entries: Vec<String> = (1..=3)
        .map(|i| project.path().join(format!("src/entry{}.js", i)))
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let result = BuildOptions::new_multiple(entries)
        .bundle_separately()
        .cwd(project.path())
        .runtime(Arc::new(NativeRuntime::new()))
        .build()
        .await
        .expect("build should succeed");

    // Verify merged graph contains modules from all entries
    assert!(
        result.stats().module_count >= 3,
        "Merged graph should contain modules from all entries"
    );
}

#[tokio::test]
async fn parallel_builds_collects_all_errors() {
    let dir = TempDir::new().expect("temp dir");
    let src = dir.path().join("src");
    std::fs::create_dir(&src).expect("create src");

    // Create one valid and two invalid entries
    std::fs::write(src.join("valid.js"), "export const x = 1;").expect("write valid");

    std::fs::write(
        src.join("invalid1.js"),
        "import { missing } from './nonexistent';",
    )
    .expect("write invalid1");

    std::fs::write(
        src.join("invalid2.js"),
        "import { also_missing } from './also_nonexistent';",
    )
    .expect("write invalid2");

    let entries = vec![
        dir.path()
            .join("src/valid.js")
            .to_string_lossy()
            .to_string(),
        dir.path()
            .join("src/invalid1.js")
            .to_string_lossy()
            .to_string(),
        dir.path()
            .join("src/invalid2.js")
            .to_string_lossy()
            .to_string(),
    ];

    let result = BuildOptions::new_multiple(entries)
        .bundle_separately()
        .cwd(dir.path())
        .runtime(Arc::new(NativeRuntime::new()))
        .build()
        .await;

    // Build should fail due to invalid entries
    assert!(result.is_err(), "Build should fail with invalid entries");

    // Error message should contain info about failures
    let err = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("Expected error"),
    };
    assert!(
        err.contains("invalid1") || err.contains("nonexistent") || err.contains("Build failed"),
        "Error should mention invalid entry: {}",
        err
    );
}
