//! Integration tests for TypeScript declaration file (.d.ts) generation
//!
//! These tests verify that LibraryBuilder correctly generates .d.ts files
//! using OXC's isolated declarations feature.

#![cfg(feature = "dts-generation")]

use fob_bundler::{self as fob, Platform};
use std::fs;
use tempfile::TempDir;

/// Helper to create a TypeScript library project
fn create_ts_library_project() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("Failed to create src dir");

    // Create a TypeScript library entry file
    let ts_content = r#"
/**
 * Greets a person by name
 * @param name - The name to greet
 * @returns A greeting message
 */
export function greet(name: string): string {
    return `Hello, ${name}!`;
}

/**
 * Adds two numbers
 */
export function add(a: number, b: number): number {
    return a + b;
}

/** @internal */
export function _internalHelper(): void {
    console.log("This is internal");
}
"#;

    fs::write(src.join("index.ts"), ts_content).expect("Failed to write index.ts");
    dir
}

#[tokio::test]
async fn test_library_auto_detects_typescript_and_generates_dts() {
    let project = create_ts_library_project();
    let entry = project.path().join("src/index.ts");

    // Auto-detection should enable .d.ts generation for .ts files
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .emit_dts(true)
        .cwd(project.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");

    // Check that .d.ts file was generated
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(
        dts_assets.len(),
        1,
        "Should generate exactly one .d.ts file"
    );

    let dts_content = String::from_utf8_lossy(dts_assets[0].content_as_bytes());
    assert!(dts_content.contains("greet"));
    assert!(dts_content.contains("string"));
}

#[tokio::test]
async fn test_library_respects_emit_dts_false() {
    let project = create_ts_library_project();
    let entry = project.path().join("src/index.ts");

    // Explicitly disable .d.ts generation
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .emit_dts(false)
        .cwd(project.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");

    // No .d.ts files should be generated
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(
        dts_assets.len(),
        0,
        "Should not generate .d.ts files when disabled"
    );
}

#[tokio::test]
async fn test_library_explicit_emit_dts_true() {
    let project = create_ts_library_project();
    let entry = project.path().join("src/index.ts");

    // Explicitly enable .d.ts generation
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .emit_dts(true)
        .cwd(project.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(dts_assets.len(), 1);
}

#[tokio::test]
async fn test_library_strip_internal_declarations() {
    let project = create_ts_library_project();
    let entry = project.path().join("src/index.ts");

    // Enable strip_internal
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .emit_dts(true)
        .strip_internal(true)
        .cwd(project.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(dts_assets.len(), 1);

    let dts_content = String::from_utf8_lossy(dts_assets[0].content_as_bytes());

    // @internal function should be stripped
    assert!(!dts_content.contains("_internalHelper"));

    // Public functions should still be there
    assert!(dts_content.contains("greet"));
    assert!(dts_content.contains("add"));
}

#[tokio::test]
async fn test_library_custom_dts_dir() {
    let project = create_ts_library_project();
    let entry = project.path().join("src/index.ts");

    // Specify custom .d.ts output directory
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .emit_dts(true)
        .dts_outdir("types")
        .cwd(project.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(dts_assets.len(), 1);

    // Check that the filename includes the custom directory
    let filename = dts_assets[0].filename();
    assert!(
        filename.contains("types/") || filename == "types/index.d.ts",
        "Expected filename to contain 'types/', got: {}",
        filename
    );
}

#[tokio::test]
async fn test_javascript_entry_no_dts_by_default() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("Failed to create src dir");

    // Create a JavaScript file
    fs::write(
        src.join("index.js"),
        "export function hello() { return 'world'; }",
    )
    .expect("Failed to write index.js");

    let entry = src.join("index.js");

    // JavaScript files should NOT auto-generate .d.ts
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .cwd(dir.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(
        dts_assets.len(),
        0,
        "JavaScript files should not generate .d.ts by default"
    );
}

#[tokio::test]
async fn test_tsx_file_generates_dts() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let src = dir.path().join("src");
    fs::create_dir(&src).expect("Failed to create src dir");

    // Create a TSX file with React component
    // Note: isolated declarations requires explicit return types
    let tsx_content = r#"
export interface ButtonProps {
    label: string;
    onClick: () => void;
}

export function Button(props: ButtonProps): JSX.Element {
    return <button onClick={props.onClick}>{props.label}</button>;
}
"#;

    fs::write(src.join("Button.tsx"), tsx_content).expect("Failed to write Button.tsx");

    let entry = src.join("Button.tsx");

    // TSX files should auto-generate .d.ts
    let result = fob::BuildOptions::new(entry)
        .externalize_from("package.json")
        .platform(Platform::Node)
        .emit_dts(true)
        .cwd(dir.path())
        .sourcemap(false)
        .build()
        .await
        .expect("Failed to bundle");

    let bundle = result.output.as_single().expect("single bundle");
    let dts_assets: Vec<_> = bundle
        .assets
        .iter()
        .filter(|a| a.filename().ends_with(".d.ts"))
        .collect();

    assert_eq!(dts_assets.len(), 1, "TSX files should generate .d.ts");

    let dts_content = String::from_utf8_lossy(dts_assets[0].content_as_bytes());
    assert!(dts_content.contains("ButtonProps"));
    assert!(dts_content.contains("Button"));
}
