use std::path::PathBuf;
use std::sync::Arc;

use crate::test_utils::TestRuntime;
use crate::BuildOptions;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bundle_with_analysis_collects_data() {
    let project_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rolldown/simple");
    let entry = project_dir.join("src/index.js");

    let runtime = Arc::new(TestRuntime::new(project_dir.clone()));

    let result = BuildOptions::library(entry.clone())
        .cwd(project_dir.clone())
        .runtime(runtime)
        .build()
        .await
        .expect("bundle with analysis");

    assert!(result.stats().module_count >= 2);
    assert_eq!(result.cache.total_requests, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_module_collection_plugin_integration() {
    let project_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rolldown/simple");
    let entry = project_dir.join("src/index.js");

    let runtime = Arc::new(TestRuntime::new(project_dir.clone()));

    // Build using the modern plugin-based collection API
    let result = BuildOptions::library(entry.clone())
        .cwd(project_dir.clone())
        .runtime(runtime)
        .build()
        .await
        .expect("build with collection plugin");

    // Verify the graph was populated correctly
    let graph = &result.analysis.graph;

    // Should have at least the entry and one dependency (utils.js)
    let module_count = graph.len().expect("get module count");
    assert!(
        module_count >= 2,
        "Expected at least 2 modules, got {}",
        module_count
    );

    // Verify entry points were captured
    let entry_points = graph.entry_points().expect("get entry points");
    assert!(
        !entry_points.is_empty(),
        "Expected at least one entry point"
    );

    // Verify unused exports were detected
    let unused_exports = graph.unused_exports().expect("get unused exports");

    // The simple fixture has an unused export in utils.js
    assert!(
        unused_exports.iter().any(|item| {
            item.export.name == "unused"
                && item
                    .module_id
                    .as_path()
                    .to_string_lossy()
                    .contains("utils.js")
        }),
        "Expected to find unused export 'unused' from utils.js"
    );
}
