use fob_bundler::{BuildOptions, Platform};
use tempfile::TempDir;

use std::fs;

fn create_library_project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("create src");

    fs::write(
        src.join("index.js"),
        r#"
import { greet } from './greet.js';

export function run(name) {
    return greet(name);
}
"#,
    )
    .expect("write index.js");

    fs::write(
        src.join("greet.js"),
        r#"
export function greet(name) {
    return `hello ${name}`;
}
"#,
    )
    .expect("write greet.js");

    dir
}

#[tokio::test]
async fn library_builder_produces_assets() {
    let project = create_library_project();

    let result = BuildOptions::new(project.path().join("src/index.js"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(project.path())
        .sourcemap(true)
        .build()
        .await
        .expect("library bundle");

    let bundle = result.output.as_single().expect("single bundle");
    assert!(
        !bundle.assets.is_empty(),
        "library bundling should produce assets"
    );
    assert!(
        !result.entry_points().is_empty(),
        "Expected entry points to be recorded. Entry points: {:?}, Module count: {}",
        result.entry_points(),
        result.stats().module_count
    );
}

#[tokio::test]
async fn library_builder_accepts_globals_and_minify() {
    let project = create_library_project();

    let result = BuildOptions::new(project.path().join("src/index.js"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(project.path())
        .globals_map([("react", "React")])
        .minify_level("identifiers")
        .build()
        .await
        .expect("library bundle with globals");

    let stats = result.stats();
    assert!(stats.module_count >= 2, "should analyse multiple modules");
}

#[tokio::test]
async fn virtual_file_basic() {
    let project = create_library_project();

    let result = BuildOptions::new("virtual:entry.js")
        .bundle(false)
        .platform(Platform::Node)
        .virtual_file(
            "virtual:entry.js",
            r#"
import { greet } from './src/greet.js';
export const message = greet('Virtual');
"#,
        )
        .cwd(project.path())
        .build()
        .await
        .expect("bundle with virtual entry");

    let bundle = result.output.as_single().expect("single bundle");
    assert!(!bundle.assets.is_empty());
}

#[tokio::test]
async fn virtual_file_mixed_with_physical() {
    let project = create_library_project();

    // Virtual file imports physical file
    let result = BuildOptions::new("virtual:entry.js")
        .bundle(false)
        .platform(Platform::Node)
        .virtual_file(
            "virtual:entry.js",
            r#"
import { greet } from './src/greet.js';
export function run() { return greet('world'); }
"#,
        )
        .cwd(project.path())
        .build()
        .await
        .expect("bundle mixing virtual and physical files");

    assert_eq!(result.stats().module_count, 2);
}

#[tokio::test]
async fn virtual_file_multiple() {
    let project = create_library_project();

    let result = BuildOptions::new("virtual:main.js")
        .bundle(false)
        .platform(Platform::Node)
        .virtual_file(
            "virtual:main.js",
            r#"export { config } from 'virtual:config.js';"#,
        )
        .virtual_file(
            "virtual:config.js",
            r#"export const config = { name: 'test' };"#,
        )
        .cwd(project.path())
        .build()
        .await
        .expect("bundle with multiple virtual files");

    assert_eq!(result.stats().module_count, 2);
}

#[tokio::test]
async fn virtual_file_size_limit() {
    // Create content larger than 1MB limit
    let huge_content = "export const x = 1;".to_string() + &"//comment\n".repeat(120_000);

    let result = BuildOptions::new("virtual:huge.js")
        .bundle(false)
        .platform(Platform::Node)
        .virtual_file("virtual:huge.js", huge_content)
        .build()
        .await;

    assert!(result.is_err(), "should fail for oversized virtual file");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("too large") || err_msg.contains("size"),
            "error should mention size: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn virtual_file_invalid_module_id() {
    let result = BuildOptions::new("virtual:bad\0file.js")
        .bundle(false)
        .platform(Platform::Node)
        .virtual_file("virtual:bad\0file.js", "export const x = 1;")
        .build()
        .await;

    assert!(result.is_err(), "should fail for invalid module ID");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("null byte") || err_msg.contains("Invalid"),
            "error should mention validation issue: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn virtual_file_with_typescript() {
    let project = create_library_project();

    let result = BuildOptions::new("virtual:entry.ts")
        .bundle(false)
        .platform(Platform::Node)
        .virtual_file(
            "virtual:entry.ts",
            r#"
const greeting: string = "Hello";
export default greeting;
"#,
        )
        .cwd(project.path())
        .build()
        .await
        .expect("bundle TypeScript virtual file");

    let bundle = result.output.as_single().expect("single bundle");
    assert!(!bundle.assets.is_empty());
}
