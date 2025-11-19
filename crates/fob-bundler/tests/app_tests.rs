#![cfg(feature = "bundler")]

use fob_bundler::BuildOptions;
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
    .bundle(true)
    .splitting(true)
    .cwd(project.path())
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
