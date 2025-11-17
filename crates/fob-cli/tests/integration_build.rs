//! Integration tests for the build command.
//!
//! These tests verify end-to-end build functionality with real files and directories.

use fob_cli::commands::build;
use fob_cli::cli::BuildArgs;
use fob_cli::cli::{Format, Platform, SourceMapMode, EsTarget};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_build_successful_single_entry() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create source file
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("index.ts"),
        r#"export const hello = () => console.log("Hello, world!");"#,
    )
    .unwrap();

    // Create config
    fs::write(
        project_dir.join("fob.config.json"),
        r#"{
            "entry": ["src/index.ts"],
            "outDir": "dist",
            "format": "esm"
        }"#,
    )
    .unwrap();

    // Build
    let args = BuildArgs {
        entry: vec!["src/index.ts".to_string()],
        format: Format::Esm,
        out_dir: PathBuf::from("dist"),
        dts: false,
        dts_bundle: false,
        external: vec![],
        docs: false,
        docs_format: None,
        docs_dir: None,
        docs_include_internal: false,
        docs_enhance: false,
        docs_enhance_mode: None,
        docs_llm_model: None,
        docs_no_cache: false,
        docs_llm_url: None,
        docs_write_back: false,
        docs_merge_strategy: None,
        docs_no_backup: false,
        platform: Platform::Browser,
        sourcemap: Some(SourceMapMode::External),
        minify: false,
        target: EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    // Change to project directory
    std::env::set_current_dir(project_dir).unwrap();

    let result = build::execute(args).await;
    assert!(result.is_ok(), "Build should succeed");

    // Verify output files exist
    let dist_dir = project_dir.join("dist");
    assert!(dist_dir.exists(), "Output directory should exist");
    
    // Check for JS file
    let js_files: Vec<_> = fs::read_dir(&dist_dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.ends_with(".js") && !name_str.ends_with(".map.js")
        })
        .collect();
    
    assert!(!js_files.is_empty(), "Should generate at least one JS file");
}

#[tokio::test]
async fn test_build_missing_entry_point() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create config without creating entry file
    fs::write(
        project_dir.join("fob.config.json"),
        r#"{
            "entry": ["src/index.ts"],
            "outDir": "dist"
        }"#,
    )
    .unwrap();

    std::env::set_current_dir(project_dir).unwrap();

    let args = BuildArgs {
        entry: vec!["src/index.ts".to_string()],
        format: Format::Esm,
        out_dir: PathBuf::from("dist"),
        dts: false,
        dts_bundle: false,
        external: vec![],
        docs: false,
        docs_format: None,
        docs_dir: None,
        docs_include_internal: false,
        docs_enhance: false,
        docs_enhance_mode: None,
        docs_llm_model: None,
        docs_no_cache: false,
        docs_llm_url: None,
        docs_write_back: false,
        docs_merge_strategy: None,
        docs_no_backup: false,
        platform: Platform::Browser,
        sourcemap: Some(SourceMapMode::External),
        minify: false,
        target: EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let result = build::execute(args).await;
    assert!(result.is_err(), "Build should fail with missing entry");
    
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(
        error_msg.contains("not found") || error_msg.contains("Entry"),
        "Error should mention missing entry point"
    );
}

#[tokio::test]
async fn test_build_clean_output_dir() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create source file
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("index.ts"),
        r#"export const hello = () => console.log("Hello");"#,
    )
    .unwrap();

    // Create output directory with old files
    let dist_dir = project_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(dist_dir.join("old.js"), "old content").unwrap();
    fs::write(dist_dir.join("old.txt"), "old text").unwrap();

    std::env::set_current_dir(project_dir).unwrap();

    let args = BuildArgs {
        entry: vec!["src/index.ts".to_string()],
        format: Format::Esm,
        out_dir: PathBuf::from("dist"),
        dts: false,
        dts_bundle: false,
        external: vec![],
        docs: false,
        docs_format: None,
        docs_dir: None,
        docs_include_internal: false,
        docs_enhance: false,
        docs_enhance_mode: None,
        docs_llm_model: None,
        docs_no_cache: false,
        docs_llm_url: None,
        docs_write_back: false,
        docs_merge_strategy: None,
        docs_no_backup: false,
        platform: Platform::Browser,
        sourcemap: Some(SourceMapMode::External),
        minify: false,
        target: EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: true, // Enable clean
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let result = build::execute(args).await;
    assert!(result.is_ok(), "Build should succeed");

    // Verify old files are gone
    assert!(!dist_dir.join("old.js").exists(), "Old JS file should be removed");
    assert!(!dist_dir.join("old.txt").exists(), "Old text file should be removed");

    // Verify new files exist
    let js_files: Vec<_> = fs::read_dir(&dist_dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.ends_with(".js") && !name_str.ends_with(".map.js")
        })
        .collect();
    
    assert!(!js_files.is_empty(), "Should generate new JS files");
}

#[tokio::test]
async fn test_build_empty_entry_list() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    std::env::set_current_dir(project_dir).unwrap();

    let args = BuildArgs {
        entry: vec![], // Empty entry list
        format: Format::Esm,
        out_dir: PathBuf::from("dist"),
        dts: false,
        dts_bundle: false,
        external: vec![],
        docs: false,
        docs_format: None,
        docs_dir: None,
        docs_include_internal: false,
        docs_enhance: false,
        docs_enhance_mode: None,
        docs_llm_model: None,
        docs_no_cache: false,
        docs_llm_url: None,
        docs_write_back: false,
        docs_merge_strategy: None,
        docs_no_backup: false,
        platform: Platform::Browser,
        sourcemap: Some(SourceMapMode::External),
        minify: false,
        target: EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let result = build::execute(args).await;
    assert!(result.is_err(), "Build should fail with empty entry list");
    
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(
        error_msg.contains("entry") || error_msg.contains("required"),
        "Error should mention entry point requirement"
    );
}

