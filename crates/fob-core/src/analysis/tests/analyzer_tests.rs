//! Unit tests for Analyzer API.

use super::test_helpers::*;
use crate::analysis::analyzer::Analyzer;
use crate::test_utils::TestRuntime;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_analyzer_builder_pattern() {
    // Test that builder methods can be chained
    let analyzer = Analyzer::new()
        .entry("src/index.ts")
        .external(vec!["react"])
        .path_alias("@", "./src")
        .max_depth(Some(100));
    
    // Verify by attempting to analyze (will fail without runtime, but that's ok)
    // The builder pattern itself is tested by the fact that it compiles
    let result = analyzer.analyze().await;
    // Should fail due to missing runtime or file, but not due to builder pattern
    assert!(result.is_err());
}

#[tokio::test]
async fn test_analyzer_default_values() {
    // Test that default analyzer requires entry
    let analyzer = Analyzer::new();
    let result = analyzer.analyze().await;
    
    // Should fail because no entries provided
    assert!(result.is_err());
    if let Err(crate::Error::InvalidConfig(msg)) = result {
        assert!(msg.contains("entry"));
    }
}

#[tokio::test]
async fn test_analyzer_fluent_api() {
    // Test that fluent API methods can be chained
    let analyzer = Analyzer::new()
        .entry("src/index.ts")
        .entries(vec!["src/a.ts", "src/b.ts"])
        .external(vec!["react"])
        .path_alias("@", "./src")
        .follow_dynamic_imports(true)
        .include_type_imports(false)
        .max_depth(Some(50));
    
    // Verify by attempting to analyze
    let result = analyzer.analyze().await;
    // Should fail due to missing runtime/files, but API is valid
    assert!(result.is_err());
}

#[tokio::test]
async fn test_analyzer_requires_entry() {
    let analyzer = Analyzer::new();
    let result = analyzer.analyze().await;
    
    assert!(result.is_err());
    if let Err(crate::Error::InvalidConfig(msg)) = result {
        assert!(msg.contains("entry"));
    } else {
        panic!("Expected InvalidConfig error");
    }
}

#[tokio::test]
async fn test_analyze_simple_project() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    assert_eq!(analysis.entry_points.len(), 1);
    assert!(analysis.is_ok());
    assert!(!analysis.has_warnings());
}

#[tokio::test]
async fn test_analyze_with_externals() {
    let temp = TempDir::new().unwrap();
    let root = create_project_with_externals(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .external(vec!["react", "lodash"])
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    let externals = analysis.external_dependencies().await.unwrap();
    assert!(externals.iter().any(|d| d.specifier == "react"));
    assert!(externals.iter().any(|d| d.specifier == "lodash"));
}

#[tokio::test]
async fn test_analyze_with_path_aliases() {
    let temp = TempDir::new().unwrap();
    let root = create_project_with_aliases(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .path_alias("@", "./src")
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    assert!(analysis.is_ok());
    assert!(assert_graph_contains_module(&analysis.graph, "Button").await);
}

#[tokio::test]
async fn test_analyze_multiple_entries() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/a.ts", "export const a = 'a';"),
        ("src/b.ts", "export const b = 'b';"),
    ]);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entries(vec![root.join("src/a.ts"), root.join("src/b.ts")])
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    assert_eq!(analysis.entry_points.len(), 2);
}

#[tokio::test]
async fn test_analyze_result_contains_stats() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    assert!(analysis.stats.module_count >= 2);
    assert_eq!(analysis.stats.entry_point_count, 1);
}

#[tokio::test]
async fn test_analyze_result_contains_symbol_stats() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    // Symbol stats should be present (may be empty for simple cases)
    // Just verify it exists - actual values depend on parsing
    let _ = analysis.symbol_stats.total_symbols;
}

#[tokio::test]
async fn test_analyze_with_custom_max_depth() {
    let temp = TempDir::new().unwrap();
    let root = create_deep_project(&temp, 20);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/module0.ts"))
        .max_depth(Some(10))
        .cwd(root)
        .runtime(runtime)
        .analyze();
    
    // Should fail due to max depth
    assert!(analysis.await.is_err());
}

#[tokio::test]
async fn test_analyze_respects_cwd() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry("src/index.ts")
        .cwd(root.clone())
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    assert!(analysis.is_ok());
}

#[tokio::test]
async fn test_analyze_uses_provided_runtime() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    assert!(analysis.is_ok());
}

#[tokio::test]
#[cfg(not(target_family = "wasm"))]
async fn test_analyze_creates_default_runtime_on_native() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    
    // Don't provide runtime - should use default NativeRuntime
    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .analyze()
        .await
        .unwrap();
    
    assert!(analysis.is_ok());
}

#[tokio::test]
async fn test_analyze_handles_circular_dependencies() {
    let temp = TempDir::new().unwrap();
    let root = create_circular_project(&temp);
    let runtime = Arc::new(TestRuntime::new(root.clone()));
    
    let analysis = Analyzer::new()
        .entry(root.join("src/a.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();
    
    // Should complete successfully
    assert!(analysis.is_ok());
    assert_eq!(analysis.entry_points.len(), 1);
}

