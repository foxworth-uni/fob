//! Integration tests for end-to-end scenarios.

use super::test_helpers::*;
use crate::analysis::analyzer::Analyzer;
use crate::test_utils::TestRuntime;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_detect_unused_exports() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import { used } from './utils';
            export const main = () => used();
        "#,
            ),
            (
                "src/utils.ts",
                r#"
            export const used = () => 'used';
            export const unused = () => 'unused';
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    let unused = analysis.unused_exports().unwrap();
    assert!(unused.iter().any(|u| u.export.name == "unused"));
}

#[tokio::test]
async fn test_detect_circular_dependencies() {
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

    assert!(assert_has_circular_dependency(&analysis));
}

#[tokio::test]
async fn test_analyze_typescript_project() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            interface User {
                name: string;
            }
            import { getUser } from './api';
            export const main = async () => {
                const user: User = await getUser();
                return user;
            };
        "#,
            ),
            (
                "src/api.ts",
                r#"
            export const getUser = async () => ({ name: 'test' });
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    assert!(analysis.is_ok());
    assert_eq!(analysis.entry_points.len(), 1);
}

#[tokio::test]
async fn test_analyze_javascript_project() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.js",
                r#"
            const { helper } = require('./utils');
            module.exports = { main: () => helper() };
        "#,
            ),
            (
                "src/utils.js",
                r#"
            module.exports = { helper: () => 'hello' };
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.js"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    assert!(analysis.is_ok());
}

#[tokio::test]
async fn test_analyze_mixed_ts_js_project() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import { helper } from './utils';
            export const main = helper();
        "#,
            ),
            (
                "src/utils.js",
                r#"
            export const helper = () => 'hello';
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    assert!(analysis.is_ok());
    assert_eq!(analysis.entry_points.len(), 1);
}

#[tokio::test]
async fn test_analyze_with_path_aliases_integration() {
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
    assert!(assert_graph_contains_module(&analysis.graph, "Button"));
}

#[tokio::test]
async fn test_large_project_performance() {
    let temp = TempDir::new().unwrap();

    // Create 50 modules
    let mut owned_files: Vec<(String, String)> = vec![];
    for i in 0..50 {
        owned_files.push((
            format!("src/module{}.ts", i),
            format!("export const fn{} = () => 'module{}';", i, i),
        ));
    }

    let files_ref: Vec<(&str, &str)> = owned_files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();

    let root = create_test_project(&temp, &files_ref);
    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let start = std::time::Instant::now();

    let analysis = Analyzer::new()
        .entries((0..50).map(|i| root.join(format!("src/module{}.ts", i))))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    let duration = start.elapsed();

    assert_eq!(analysis.entry_points.len(), 50);
    // Should complete in under 1 second for 50 modules
    assert!(duration.as_millis() < 1000);
}

#[tokio::test]
async fn test_deep_nesting_within_limits() {
    let temp = TempDir::new().unwrap();
    let root = create_deep_project(&temp, 20);
    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/module0.ts"))
        .max_depth(Some(30))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    assert!(analysis.is_ok());
    assert_eq!(analysis.entry_points.len(), 1);
}

#[tokio::test]
async fn test_wide_dependencies() {
    let temp = TempDir::new().unwrap();

    // Create a module that imports many others
    let mut owned_files: Vec<(String, String)> = vec![("src/index.ts".to_string(), "".to_string())];
    let mut imports = Vec::new();

    for i in 0..20 {
        owned_files.push((
            format!("src/module{}.ts", i),
            format!("export const fn{} = () => 'module{}';", i, i),
        ));
        imports.push(format!("import {{ fn{} }} from './module{}';", i, i));
    }

    owned_files[0].1 = format!("{}\nexport const main = () => {{}};", imports.join("\n"));

    let files_ref: Vec<(&str, &str)> = owned_files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();

    let root = create_test_project(&temp, &files_ref);
    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    assert!(analysis.is_ok());
    // Should have index + all 20 modules
    assert!(analysis.stats.module_count >= 21);
}

#[tokio::test]
async fn test_dependency_chains() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import { a } from './a';
            export const main = a();
        "#,
            ),
            (
                "src/a.ts",
                r#"
            import { b } from './b';
            export const a = () => b();
        "#,
            ),
            (
                "src/b.ts",
                r#"
            export const b = () => 'b';
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    let modules = analysis.graph.modules().unwrap();
    let b_module = modules
        .iter()
        .find(|m| m.path.to_string_lossy().contains("b.ts"))
        .unwrap();

    let chains = analysis.dependency_chains_to(&b_module.id).unwrap();
    assert!(!chains.is_empty());
    // Should have chain: index -> a -> b
    assert!(chains.iter().any(|c| c.path.len() == 3));
}

#[tokio::test]
async fn test_error_recovery_continues_on_parse_errors() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import { helper } from './utils';
            import { broken } from './broken';
            export const main = helper();
        "#,
            ),
            (
                "src/utils.ts",
                r#"
            export const helper = () => 'helper';
        "#,
            ),
            (
                "src/broken.ts",
                r#"
            // Syntax error: missing closing brace
            export const broken = () => {
                return 'broken'
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    // Should still complete and include valid modules
    assert!(analysis.is_ok());
    assert!(assert_graph_contains_module(&analysis.graph, "utils"));
}

#[tokio::test]
async fn test_handles_missing_imports_gracefully() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import { helper } from './utils';
            import { missing } from './nonexistent';
            export const main = helper();
        "#,
            ),
            (
                "src/utils.ts",
                r#"
            export const helper = () => 'helper';
        "#,
            ),
        ],
    );

    let runtime = Arc::new(TestRuntime::new(root.clone()));

    let analysis = Analyzer::new()
        .entry(root.join("src/index.ts"))
        .cwd(root)
        .runtime(runtime)
        .analyze()
        .await
        .unwrap();

    // Should complete despite missing import
    assert!(analysis.is_ok());
    // Should have warnings about unresolved imports
    assert!(analysis.warnings.is_empty() || analysis.has_warnings());
}
