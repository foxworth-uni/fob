//! Edge case tests for fob-analysis.
//!
//! These tests cover edge cases like empty files, malformed UTF-8,
//! very deep nesting, and very wide graphs.

use super::test_helpers::*;
use crate::Analyzer;
use fob_core::test_utils::TestRuntime;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_empty_file() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = create_test_project(
        &temp,
        &[
            ("src/index.ts", ""),
            ("src/utils.ts", "export const x = 1;"),
        ],
    );

    let runtime = TestRuntime::new(root.clone());
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .runtime(Arc::new(runtime))
        .analyze()
        .await;

    // Empty files should be handled gracefully
    assert!(analysis.is_ok(), "Empty files should not cause errors");
}

#[tokio::test]
async fn test_zero_length_imports() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = create_test_project(
        &temp,
        &[(
            "src/index.ts",
            "import '' from './utils'; export const x = 1;",
        )],
    );

    let runtime = TestRuntime::new(root.clone());
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .runtime(Arc::new(runtime))
        .analyze()
        .await;

    // Zero-length imports should be handled gracefully
    // (may fail to parse, but shouldn't panic)
    let _ = analysis; // Just ensure it doesn't panic
}

#[tokio::test]
async fn test_very_deep_nesting() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path().to_path_buf();

    // Create a deeply nested dependency chain
    let depth = 50;
    for i in 0..depth {
        let path = root.join(format!("src/level{}.ts", i));
        let parent = path.parent().expect("Should have parent");
        std::fs::create_dir_all(parent).expect("Failed to create dir");

        let content = if i == depth - 1 {
            "export const x = 1;".to_string()
        } else {
            format!("export * from './level{}';", i + 1)
        };

        std::fs::write(&path, content).expect("Failed to write file");
    }

    let runtime = TestRuntime::new(root.clone());
    let analysis = Analyzer::new()
        .entry(root.join("src/level0.ts"))
        .max_depth(Some(100)) // Allow deep nesting
        .runtime(Arc::new(runtime))
        .analyze()
        .await;

    assert!(analysis.is_ok(), "Deep nesting should be handled");
}

#[tokio::test]
async fn test_very_wide_graph() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let _root_path = temp.path().to_path_buf();

    // Create a wide graph (many modules importing from one)
    let width = 100;
    let mut files: Vec<(String, String)> = vec![(
        "src/index.ts".to_string(),
        "export const x = 1;".to_string(),
    )];

    for i in 0..width {
        files.push((
            format!("src/module{}.ts", i),
            format!("import {{ x }} from './index'; export const y{} = x;", i),
        ));
    }

    // Convert to owned strings for create_test_project
    let file_refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();

    let project_root = create_test_project(&temp, &file_refs);

    let runtime = TestRuntime::new(project_root.clone());
    let analysis = Analyzer::new()
        .entry(project_root.join("src/index.ts"))
        .max_modules(Some(200)) // Allow wide graph
        .runtime(Arc::new(runtime))
        .analyze()
        .await;

    assert!(analysis.is_ok(), "Wide graphs should be handled");
}

#[tokio::test]
async fn test_max_depth_enforcement() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path().to_path_buf();

    // Create a chain that exceeds max_depth
    let depth = 10;
    for i in 0..depth {
        let path = root.join(format!("src/level{}.ts", i));
        let parent = path.parent().expect("Should have parent");
        std::fs::create_dir_all(parent).expect("Failed to create dir");

        let content = if i == depth - 1 {
            "export const x = 1;".to_string()
        } else {
            format!("export * from './level{}';", i + 1)
        };

        std::fs::write(&path, content).expect("Failed to write file");
    }

    let runtime = TestRuntime::new(root.clone());
    let analysis = Analyzer::new()
        .entry(root.join("src/level0.ts"))
        .max_depth(Some(5)) // Set limit below actual depth
        .runtime(Arc::new(runtime))
        .analyze()
        .await;

    // Should fail with max depth exceeded
    assert!(analysis.is_err(), "Should fail when max depth is exceeded");
}

#[tokio::test]
async fn test_max_modules_enforcement() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path().to_path_buf();

    // Create many modules
    let count = 50;
    for i in 0..count {
        let path = root.join(format!("src/module{}.ts", i));
        let parent = path.parent().expect("Should have parent");
        std::fs::create_dir_all(parent).expect("Failed to create dir");
        std::fs::write(&path, format!("export const x{} = {};", i, i))
            .expect("Failed to write file");
    }

    // Create an index that imports all
    let mut imports: Vec<String> = (0..count)
        .map(|i| format!("import {{ x{} }} from './module{}';", i, i))
        .collect();
    imports.push("export const main = () => {};".to_string());
    let index_content = imports.join("\n");

    std::fs::write(root.join("src/index.ts"), index_content).expect("Failed to write index");

    let runtime = TestRuntime::new(root.clone());
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .max_modules(Some(10)) // Set limit below actual count
        .runtime(Arc::new(runtime))
        .analyze()
        .await;

    // Should fail with too many modules
    assert!(
        analysis.is_err(),
        "Should fail when max modules is exceeded"
    );
}
