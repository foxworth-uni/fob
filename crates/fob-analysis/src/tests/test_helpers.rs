//! Shared test utilities for analysis tests.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test project with the given files.
///
/// # Arguments
/// * `temp` - Temporary directory
/// * `files` - Array of (path, content) tuples
///
/// # Returns
/// The root path of the created project
pub fn create_test_project(temp: &TempDir, files: &[(&str, &str)]) -> PathBuf {
    let root = temp.path().to_path_buf();

    for (path, content) in files {
        let file_path = root.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .expect(&format!("Failed to create parent directory for {}", path));
        }
        fs::write(&file_path, content)
            .expect(&format!("Failed to write file {}", path));
    }

    root
}

/// Create a simple TypeScript project for testing.
pub fn create_simple_ts_project(temp: &TempDir) -> PathBuf {
    create_test_project(
        temp,
        &[
            (
                "src/index.ts",
                r#"
            import { helper } from './utils';
            export const main = () => helper();
        "#,
            ),
            (
                "src/utils.ts",
                r#"
            export const helper = () => 'hello';
        "#,
            ),
        ],
    )
}

/// Create a project with external dependencies.
pub fn create_project_with_externals(temp: &TempDir) -> PathBuf {
    create_test_project(
        temp,
        &[
            (
                "src/index.ts",
                r#"
            import React from 'react';
            import { helper } from './utils';
            export const App = () => React.createElement('div');
        "#,
            ),
            (
                "src/utils.ts",
                r#"
            import lodash from 'lodash';
            export const helper = () => lodash.identity('test');
        "#,
            ),
        ],
    )
}

/// Create a project with path aliases.
pub fn create_project_with_aliases(temp: &TempDir) -> PathBuf {
    create_test_project(
        temp,
        &[
            (
                "src/index.ts",
                r#"
            import { Button } from '@/components/Button';
            export { Button };
        "#,
            ),
            (
                "src/components/Button.ts",
                r#"
            export const Button = () => 'button';
        "#,
            ),
        ],
    )
}

/// Create a project with circular dependencies.
pub fn create_circular_project(temp: &TempDir) -> PathBuf {
    create_test_project(
        temp,
        &[
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
            import { a } from './a';
            export const b = () => a();
        "#,
            ),
        ],
    )
}

/// Create a deep nesting project.
pub fn create_deep_project(temp: &TempDir, depth: usize) -> PathBuf {
    let mut files = vec![];

    for i in 0..depth {
        let next = if i < depth - 1 {
            format!("import {{ fn{} }} from './module{}';", i + 1, i + 1)
        } else {
            String::new()
        };

        files.push((
            format!("src/module{}.ts", i),
            format!(
                r#"
                {}
                export const fn{} = () => 'level{}';
            "#,
                next, i, i
            ),
        ));
    }

    // Convert Vec<(String, String)> to Vec<(&str, &str)>
    // We need to collect into a vector that owns the strings
    let owned_files: Vec<(String, String)> = files;
    let files_ref: Vec<(&str, &str)> = owned_files
        .iter()
        .map(|(path, content)| (path.as_str(), content.as_str()))
        .collect();

    create_test_project(temp, &files_ref)
}

/// Assert that a graph contains a module with the given path.
pub fn assert_graph_contains_module(graph: &fob_graph::ModuleGraph, path: &str) -> bool {
    let modules = graph.modules().unwrap();
    modules
        .iter()
        .any(|m| m.path.to_string_lossy().contains(path))
}

/// Assert that analysis found a circular dependency.
pub fn assert_has_circular_dependency(analysis: &crate::AnalysisResult) -> bool {
    let circular = analysis.find_circular_dependencies().unwrap();
    !circular.is_empty()
}
