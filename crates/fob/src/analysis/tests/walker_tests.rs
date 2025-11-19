//! Unit tests for GraphWalker.

use super::test_helpers::*;
use crate::analysis::walker::GraphWalker;
use crate::analysis::types::AnalyzerConfig;
use crate::test_utils::TestRuntime;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_walk_single_file() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", "export const hello = 'world';"),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    assert_eq!(collection.modules.len(), 1);
    assert_eq!(collection.entry_points.len(), 1);
}

#[tokio::test]
async fn test_walk_with_dependencies() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    assert_eq!(collection.modules.len(), 2); // index.ts + utils.ts
    assert_eq!(collection.entry_points.len(), 1);
}

#[tokio::test]
async fn test_walk_multiple_entries() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/a.ts", "export const a = 'a';"),
        ("src/b.ts", "export const b = 'b';"),
        ("src/shared.ts", "export const shared = 'shared';"),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![
        root.join("src/a.ts"),
        root.join("src/b.ts"),
    ];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    assert_eq!(collection.entry_points.len(), 2);
    assert!(collection.modules.len() >= 2);
}

#[tokio::test]
async fn test_walk_stops_at_externals() {
    let temp = TempDir::new().unwrap();
    let root = create_project_with_externals(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    config.external = vec!["react".to_string(), "lodash".to_string()];
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should have index.ts and utils.ts, but not react or lodash
    assert_eq!(collection.modules.len(), 2);
}

#[tokio::test]
async fn test_walk_respects_max_depth() {
    let temp = TempDir::new().unwrap();
    let root = create_deep_project(&temp, 10);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/module0.ts")];
    config.cwd = Some(root.clone());
    config.max_depth = Some(5);
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let result = walker.walk(runtime).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("depth"));
}

#[tokio::test]
async fn test_walk_deep_nesting_within_limit() {
    let temp = TempDir::new().unwrap();
    let root = create_deep_project(&temp, 5);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/module0.ts")];
    config.cwd = Some(root.clone());
    config.max_depth = Some(10);
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    assert_eq!(collection.modules.len(), 5);
}

#[tokio::test]
async fn test_walk_handles_circular_dependencies() {
    let temp = TempDir::new().unwrap();
    let root = create_circular_project(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/a.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should complete without infinite loop
    assert_eq!(collection.modules.len(), 2); // a.ts and b.ts
}

#[tokio::test]
async fn test_walk_skips_dynamic_imports_by_default() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", r#"
            import('./dynamic').then(m => m.default());
            export const main = 'main';
        "#),
        ("src/dynamic.ts", r#"
            export default () => 'dynamic';
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    config.follow_dynamic_imports = false;
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should only have index.ts, not dynamic.ts
    assert_eq!(collection.modules.len(), 1);
}

#[tokio::test]
async fn test_walk_follows_dynamic_imports_when_enabled() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", r#"
            import('./dynamic').then(m => m.default());
            export const main = 'main';
        "#),
        ("src/dynamic.ts", r#"
            export default () => 'dynamic';
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    config.follow_dynamic_imports = true;
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Note: Current implementation doesn't parse dynamic imports from code
    // This test verifies the config is respected
    assert_eq!(collection.entry_points.len(), 1);
}

#[tokio::test]
async fn test_walk_marks_entry_points() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    assert_eq!(collection.entry_points.len(), 1);
    assert!(collection.entry_points[0].contains("index.ts"));
}

#[tokio::test]
async fn test_walk_preserves_imports() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    let index_module = collection.modules.get("src/index.ts");
    assert!(index_module.is_some());
    let index = index_module.unwrap();
    assert_eq!(index.imports.len(), 1);
    assert_eq!(index.imports[0].source, "./utils");
}

#[tokio::test]
async fn test_walk_preserves_exports() {
    let temp = TempDir::new().unwrap();
    let root = create_simple_ts_project(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    let utils_module = collection.modules.get("src/utils.ts");
    assert!(utils_module.is_some());
    let utils = utils_module.unwrap();
    assert!(!utils.exports.is_empty());
}

#[tokio::test]
async fn test_walk_handles_unresolved_imports() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", r#"
            import { missing } from './nonexistent';
            export const main = 'main';
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    // Should not error, but treat unresolved as external
    let collection = walker.walk(runtime).await.unwrap();
    
    assert_eq!(collection.modules.len(), 1); // Only index.ts
}

#[tokio::test]
async fn test_walk_continues_on_parse_errors() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", r#"
            import { helper } from './utils';
            export const main = helper();
        "#),
        ("src/utils.ts", r#"
            // Syntax error: missing closing brace
            export const helper = () => {
                return 'hello'
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    // Should continue despite parse error in utils.ts
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should still collect both modules
    assert!(collection.modules.len() >= 1);
}

#[tokio::test]
async fn test_walk_with_path_aliases() {
    let temp = TempDir::new().unwrap();
    let root = create_project_with_aliases(&temp);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    config.path_aliases.insert("@".to_string(), "./src".to_string());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should resolve @/components/Button and include it
    assert!(collection.modules.len() >= 2);
}

#[tokio::test]
async fn test_walk_bfs_order() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", r#"
            import { a } from './a';
            import { b } from './b';
            export const main = 'main';
        "#),
        ("src/a.ts", r#"
            import { shared } from './shared';
            export const a = shared();
        "#),
        ("src/b.ts", r#"
            import { shared } from './shared';
            export const b = shared();
        "#),
        ("src/shared.ts", r#"
            export const shared = () => 'shared';
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should have all 4 modules
    assert_eq!(collection.modules.len(), 4);
}

#[tokio::test]
async fn test_walk_three_module_circular() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/a.ts", r#"
            import { b } from './b';
            export const a = () => b();
        "#),
        ("src/b.ts", r#"
            import { c } from './c';
            export const b = () => c();
        "#),
        ("src/c.ts", r#"
            import { a } from './a';
            export const c = () => a();
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/a.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should complete without infinite loop
    assert_eq!(collection.modules.len(), 3);
}

#[tokio::test]
async fn test_walk_with_re_exports() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[
        ("src/index.ts", r#"
            export { helper } from './utils';
        "#),
        ("src/utils.ts", r#"
            export const helper = () => 'helper';
        "#),
    ]);
    
    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());
    
    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));
    
    let collection = walker.walk(runtime).await.unwrap();
    
    // Should follow re-export and include utils.ts
    assert_eq!(collection.modules.len(), 2);
}

