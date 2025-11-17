#![cfg(feature = "rolldown-integration")]

use fob_core::BuildOptions;
use tempfile::TempDir;

use std::fs;

fn create_components_project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("create src");

    fs::write(
        src.join("shared.js"),
        r#"
export function format(label) {
    return label.toUpperCase();
}
"#,
    )
    .expect("write shared");

    fs::write(
        src.join("button.js"),
        r#"
import { format } from './shared.js';

export function renderButton(label) {
    return `<button>${format(label)}</button>`;
}
"#,
    )
    .expect("write button");

    fs::write(
        src.join("badge.js"),
        r#"
import { format } from './shared.js';

export function renderBadge(label) {
    return `<span>${format(label)}</span>`;
}
"#,
    )
    .expect("write badge");

    dir
}

#[tokio::test]
async fn components_builder_creates_multiple_bundles() {
    let project = create_components_project();

    let result = BuildOptions::new_multiple([
        project.path().join("src/button.js"),
        project.path().join("src/badge.js"),
    ])
    .bundle(true)
    .splitting(false)
    .cwd(project.path())
    .build()
    .await
    .expect("components bundle");

    let bundles = result.output.as_multiple().expect("multiple bundles");
    assert_eq!(bundles.len(), 2, "expect one bundle per entry");
    assert!(
        bundles.contains_key("button") || bundles.contains_key("src/button"),
        "bundles should include button"
    );
}

#[tokio::test]
async fn components_builder_accumulates_shared_graph() {
    let project = create_components_project();

    let result = BuildOptions::new_multiple([
        project.path().join("src/button.js"),
        project.path().join("src/badge.js"),
    ])
    .bundle(true)
    .splitting(false)
    .cwd(project.path())
    .build()
    .await
    .expect("components bundle");

    assert!(
        result.stats().module_count >= 3,
        "shared graph should contain modules"
    );
}
