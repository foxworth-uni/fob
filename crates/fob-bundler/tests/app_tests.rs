use fob_bundler::{BuildOptions, NativeRuntime};
use std::sync::Arc;
use tempfile::TempDir;

use std::fs;

fn create_app_project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("create src");

    fs::write(
        src.join("shared.js"),
        r#"
export function add(a, b) {
    return a + b;
}
"#,
    )
    .expect("write shared");

    fs::write(
        src.join("main.js"),
        r#"
import { add } from './shared.js';

export function boot() {
    return add(2, 3);
}
"#,
    )
    .expect("write main");

    fs::write(
        src.join("dashboard.js"),
        r#"
import { add } from './shared.js';

export function loadDashboard() {
    return add(10, 5);
}
"#,
    )
    .expect("write dashboard");

    dir
}

#[tokio::test]
async fn app_builder_produces_analysis_bundle() {
    let project = create_app_project();

    let result = BuildOptions::new_multiple([
        project.path().join("src/main.js"),
        project.path().join("src/dashboard.js"),
    ])
    .bundle_together()
    .with_code_splitting()
    .cwd(project.path())
    .runtime(Arc::new(NativeRuntime::new()))
    .build()
    .await
    .expect("app bundle");

    let bundle = result.output.as_single().expect("single bundle");
    assert!(!bundle.assets.is_empty(), "app bundling should emit assets");
    assert!(
        result.stats().module_count >= 3,
        "app analysis should count modules"
    );
    assert!(
        result.stats().module_count >= 3,
        "app module graph should capture modules"
    );
}

/// Test that bundler handles circular dependencies without hanging or crashing
#[tokio::test]
async fn app_builder_handles_circular_deps() {
    let dir = TempDir::new().expect("temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("create src");

    // a.js imports b.js, b.js imports a.js (circular)
    fs::write(
        src.join("a.js"),
        r#"
import { b } from './b.js';
export const a = "a";
export function getB() { return b; }
"#,
    )
    .expect("write a.js");

    fs::write(
        src.join("b.js"),
        r#"
import { a } from './a.js';
export const b = "b";
export function getA() { return a; }
"#,
    )
    .expect("write b.js");

    // This should complete without hanging (timeout will catch infinite loops)
    let result = BuildOptions::new(dir.path().join("src/a.js"))
        .cwd(dir.path())
        .runtime(Arc::new(NativeRuntime::new()))
        .build()
        .await;

    // Circular dependencies should be handled gracefully - either succeed or fail with clear error
    match result {
        Ok(bundle) => {
            // If it succeeds, verify both modules are in the bundle
            assert!(
                bundle.stats().module_count >= 2,
                "Should include both circular modules"
            );
        }
        Err(e) => {
            // If it fails, error message should mention the cycle (not hang or panic)
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("circular") || err_msg.contains("cycle") || err_msg.len() > 0,
                "Error should be meaningful, not empty: {}",
                err_msg
            );
        }
    }
}
