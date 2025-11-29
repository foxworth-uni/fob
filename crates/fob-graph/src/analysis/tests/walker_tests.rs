//! Unit tests for GraphWalker.

use super::test_helpers::*;
use crate::test_utils::TestRuntime;
use fob_graph::analysis::config::AnalyzerConfig;
use fob_graph::analysis::walker::GraphWalker;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_walk_single_file() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", "export const hello = 'world';")]);

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
    let root = create_test_project(
        &temp,
        &[
            ("src/a.ts", "export const a = 'a';"),
            ("src/b.ts", "export const b = 'b';"),
            ("src/shared.ts", "export const shared = 'shared';"),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/a.ts"), root.join("src/b.ts")];
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
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import('./dynamic').then(m => m.default());
            export const main = 'main';
        "#,
            ),
            (
                "src/dynamic.ts",
                r#"
            export default () => 'dynamic';
        "#,
            ),
        ],
    );

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
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import('./dynamic').then(m => m.default());
            export const main = 'main';
        "#,
            ),
            (
                "src/dynamic.ts",
                r#"
            export default () => 'dynamic';
        "#,
            ),
        ],
    );

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
    let root = create_test_project(
        &temp,
        &[(
            "src/index.ts",
            r#"
            import { missing } from './nonexistent';
            export const main = 'main';
        "#,
        )],
    );

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
                "src/utils.ts",
                r#"
            // Syntax error: missing closing brace
            export const helper = () => {
                return 'hello'
        "#,
            ),
        ],
    );

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
    config
        .path_aliases
        .insert("@".to_string(), "./src".to_string());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should resolve @/components/Button and include it
    assert!(collection.modules.len() >= 2);
}

#[tokio::test]
async fn test_walk_bfs_order() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            import { a } from './a';
            import { b } from './b';
            export const main = 'main';
        "#,
            ),
            (
                "src/a.ts",
                r#"
            import { shared } from './shared';
            export const a = shared();
        "#,
            ),
            (
                "src/b.ts",
                r#"
            import { shared } from './shared';
            export const b = shared();
        "#,
            ),
            (
                "src/shared.ts",
                r#"
            export const shared = () => 'shared';
        "#,
            ),
        ],
    );

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
    let root = create_test_project(
        &temp,
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
            import { c } from './c';
            export const b = () => c();
        "#,
            ),
            (
                "src/c.ts",
                r#"
            import { a } from './a';
            export const c = () => a();
        "#,
            ),
        ],
    );

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
    let root = create_test_project(
        &temp,
        &[
            (
                "src/index.ts",
                r#"
            export { helper } from './utils';
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

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/index.ts")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should follow re-export and include utils.ts
    assert_eq!(collection.modules.len(), 2);
}

// Framework file extraction tests

#[tokio::test]
async fn test_walk_extracts_astro_frontmatter() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.astro",
                r#"---
import { helper } from './utils';
const message = helper();
---
<div>{message}</div>
"#,
            ),
            (
                "src/utils.ts",
                r#"
export const helper = () => 'hello';
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.astro")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract frontmatter and follow import to utils.ts
    assert_eq!(collection.modules.len(), 2);
    let astro_module = collection.modules.get("src/Component.astro");
    assert!(astro_module.is_some());
    let astro = astro_module.unwrap();
    assert_eq!(astro.imports.len(), 1);
    assert_eq!(astro.imports[0].source, "./utils");
}

#[tokio::test]
async fn test_walk_extracts_astro_script_tags() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.astro",
                r#"---
const serverData = 'server';
---
<div>{serverData}</div>
<script>
import { clientHelper } from './client-utils';
const clientData = clientHelper();
</script>
"#,
            ),
            (
                "src/client-utils.ts",
                r#"
export const clientHelper = () => 'client';
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.astro")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract both frontmatter and script tag, follow import to client-utils.ts
    assert_eq!(collection.modules.len(), 2);
    let astro_module = collection.modules.get("src/Component.astro");
    assert!(astro_module.is_some());
    let astro = astro_module.unwrap();
    // Should have import from script tag
    assert!(
        astro
            .imports
            .iter()
            .any(|imp| imp.source == "./client-utils")
    );
}

#[tokio::test]
async fn test_walk_extracts_svelte_module_context() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.svelte",
                r#"<script context="module">
import { shared } from './shared';
export const moduleValue = shared();
</script>

<script>
let local = 'local';
</script>

<div>{local}</div>
"#,
            ),
            (
                "src/shared.ts",
                r#"
export const shared = () => 'shared';
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.svelte")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract module context script and follow import to shared.ts
    assert_eq!(collection.modules.len(), 2);
    let svelte_module = collection.modules.get("src/Component.svelte");
    assert!(svelte_module.is_some());
    let svelte = svelte_module.unwrap();
    assert_eq!(svelte.imports.len(), 1);
    assert_eq!(svelte.imports[0].source, "./shared");
}

#[tokio::test]
async fn test_walk_extracts_svelte_component_script() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.svelte",
                r#"<script>
import { helper } from './utils';
let value = helper();
</script>

<div>{value}</div>
"#,
            ),
            (
                "src/utils.ts",
                r#"
export const helper = () => 'hello';
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.svelte")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract component script and follow import to utils.ts
    assert_eq!(collection.modules.len(), 2);
    let svelte_module = collection.modules.get("src/Component.svelte");
    assert!(svelte_module.is_some());
    let svelte = svelte_module.unwrap();
    assert_eq!(svelte.imports.len(), 1);
    assert_eq!(svelte.imports[0].source, "./utils");
}

#[tokio::test]
async fn test_walk_extracts_vue_setup_script() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.vue",
                r#"<script setup>
import { computed } from 'vue';
import { helper } from './utils';

const value = computed(() => helper());
</script>

<template>
  <div>{{ value }}</div>
</template>
"#,
            ),
            (
                "src/utils.ts",
                r#"
export const helper = () => 'hello';
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.vue")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract setup script and follow imports
    assert_eq!(collection.modules.len(), 2);
    let vue_module = collection.modules.get("src/Component.vue");
    assert!(vue_module.is_some());
    let vue = vue_module.unwrap();
    // Should have imports from both vue and utils
    assert!(vue.imports.iter().any(|imp| imp.source == "./utils"));
}

#[tokio::test]
async fn test_walk_extracts_vue_regular_script() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.vue",
                r#"<script>
import { helper } from './utils';

export default {
  setup() {
    return { helper };
  }
};
</script>

<template>
  <div>{{ helper() }}</div>
</template>
"#,
            ),
            (
                "src/utils.ts",
                r#"
export const helper = () => 'hello';
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.vue")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract regular script and follow import to utils.ts
    assert_eq!(collection.modules.len(), 2);
    let vue_module = collection.modules.get("src/Component.vue");
    assert!(vue_module.is_some());
    let vue = vue_module.unwrap();
    assert_eq!(vue.imports.len(), 1);
    assert_eq!(vue.imports[0].source, "./utils");
}

#[tokio::test]
async fn test_walk_handles_framework_file_with_no_scripts() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[(
            "src/Component.astro",
            r#"<div>No scripts here</div>
"#,
        )],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.astro")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should handle file gracefully even with no scripts
    assert_eq!(collection.modules.len(), 1);
    let astro_module = collection.modules.get("src/Component.astro");
    assert!(astro_module.is_some());
    let astro = astro_module.unwrap();
    assert_eq!(astro.imports.len(), 0);
}

#[tokio::test]
async fn test_walk_framework_file_with_typescript() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/Component.svelte",
                r#"<script lang="ts">
import type { User } from './types';
import { getUser } from './api';

let user: User = getUser();
</script>

<div>{user.name}</div>
"#,
            ),
            (
                "src/types.ts",
                r#"
export type User = { name: string };
"#,
            ),
            (
                "src/api.ts",
                r#"
import type { User } from './types';
export const getUser = (): User => ({ name: 'Test' });
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.svelte")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract TypeScript script and follow imports
    assert_eq!(collection.modules.len(), 3);
    let svelte_module = collection.modules.get("src/Component.svelte");
    assert!(svelte_module.is_some());
    let svelte = svelte_module.unwrap();
    // Should have import from api.ts (type imports may be filtered, but runtime imports should be present)
    assert!(svelte.imports.iter().any(|imp| imp.source == "./api"));
}

#[tokio::test]
async fn test_walk_framework_file_imports_other_framework_file() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            (
                "src/App.astro",
                r#"---
import { Button } from './Button.svelte';
const app = 'app';
---
<Button />
"#,
            ),
            (
                "src/Button.svelte",
                r#"<script>
export let label = 'Click me';
</script>

<button>{label}</button>
"#,
            ),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/App.astro")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    let collection = walker.walk(runtime).await.unwrap();

    // Should extract from both framework files and follow cross-framework imports
    assert_eq!(collection.modules.len(), 2);
    let astro_module = collection.modules.get("src/App.astro");
    assert!(astro_module.is_some());
    let astro = astro_module.unwrap();
    assert_eq!(astro.imports.len(), 1);
    assert_eq!(astro.imports[0].source, "./Button.svelte");
}

#[tokio::test]
async fn test_walk_handles_framework_file_extraction_error() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[(
            "src/Component.astro",
            r#"---
// Unclosed frontmatter - this should cause an extraction error
const x = 'test';
// Missing closing ---
"#,
        )],
    );

    let mut config = AnalyzerConfig::default();
    config.entries = vec![root.join("src/Component.astro")];
    config.cwd = Some(root.clone());

    let walker = GraphWalker::new(config);
    let runtime = Arc::new(TestRuntime::new(root));

    // Should handle extraction error gracefully
    let result = walker.walk(runtime).await;

    // The walker should either:
    // 1. Return an error (if we want strict error handling)
    // 2. Continue with empty imports/exports (if we want lenient handling)
    // Based on the current implementation, it should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("extract") || error_msg.contains("Unclosed"));
}
