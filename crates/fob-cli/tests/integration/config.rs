//! Integration tests for configuration loading.
//!
//! Tests verify multi-source configuration loading with priority ordering:
//! CLI > Environment > File > Defaults

use fob_cli::config::FobConfig;
use fob_cli::cli::BuildArgs;
// BuildArgs uses CLI types, so we need both CLI and config types
use fob_cli::cli;
use fob_cli::config;
use std::fs;
use std::path::PathBuf;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_config_cli_overrides_file() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create config file
    fs::write(
        project_dir.join("fob.config.json"),
        r#"{
            "entry": ["src/file.ts"],
            "outDir": "file-dist",
            "format": "cjs"
        }"#,
    )
    .unwrap();

    // CLI args should override file config
    let args = BuildArgs {
        entry: vec!["src/cli.ts".to_string()],
        format: cli::Format::Esm,
        out_dir: PathBuf::from("cli-dist"),
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
        platform: cli::Platform::Browser,
        sourcemap: Some(cli::SourceMapMode::External),
        minify: false,
        target: cli::EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let config = FobConfig::load(&args, None).unwrap();
    
    // CLI entry should override file entry
    assert_eq!(config.entry, vec!["src/cli.ts"]);
    assert_eq!(config.out_dir, PathBuf::from("cli-dist"));
    assert_eq!(config.format, config::Format::Esm);
}

#[test]
#[serial]
fn test_config_file_fallback_when_cli_empty() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create config file
    fs::write(
        project_dir.join("fob.config.json"),
        r#"{
            "entry": ["src/file.ts"],
            "outDir": "file-dist",
            "format": "cjs"
        }"#,
    )
    .unwrap();

    // Empty CLI entry should allow file config to be used
    let args = BuildArgs {
        entry: vec![], // Empty - should use file config
        format: cli::Format::Esm,
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
        platform: cli::Platform::Browser,
        sourcemap: Some(cli::SourceMapMode::External),
        minify: false,
        target: cli::EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let config = FobConfig::load(&args, None).unwrap();
    
    // File entry should be used when CLI entry is empty
    assert_eq!(config.entry, vec!["src/file.ts"]);
    // Non-entry CLI flags should still override config file values
    assert_eq!(config.out_dir, PathBuf::from("dist"));
    assert_eq!(config.format, config::Format::Esm);
}

#[test]
#[serial]
fn test_config_defaults_when_no_file() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // No config file - should use defaults
    let args = BuildArgs {
        entry: vec!["src/index.ts".to_string()],
        format: cli::Format::Esm,
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
        platform: cli::Platform::Browser,
        sourcemap: Some(cli::SourceMapMode::External),
        minify: false,
        target: cli::EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let config = FobConfig::load(&args, None).unwrap();
    
    // Should use CLI args (which become the config)
    assert_eq!(config.entry, vec!["src/index.ts"]);
    assert_eq!(config.out_dir, PathBuf::from("dist"));
    assert_eq!(config.format, config::Format::Esm);
}

#[test]
#[serial]
fn test_config_environment_variables() {
    // RAII guard to ensure environment variables are cleaned up even if test panics
    struct EnvGuard {
        vars: Vec<&'static str>,
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for var in &self.vars {
                std::env::remove_var(var);
            }
        }
    }
    let _guard = EnvGuard {
        vars: vec!["FOB_OUT_DIR", "FOB_FORMAT"],
    };

    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Set environment variables
    std::env::set_var("FOB_OUT_DIR", "env-dist");
    std::env::set_var("FOB_FORMAT", "cjs");

    // Create config file
    fs::write(
        project_dir.join("fob.config.json"),
        r#"{
            "entry": ["src/index.ts"],
            "outDir": "file-dist",
            "format": "esm"
        }"#,
    )
    .unwrap();

    let args = BuildArgs {
        entry: vec!["src/index.ts".to_string()],
        format: cli::Format::Esm,
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
        platform: cli::Platform::Browser,
        sourcemap: Some(cli::SourceMapMode::External),
        minify: false,
        target: cli::EsTarget::Es2020,
        global_name: None,
        splitting: false,
        no_treeshake: false,
        clean: false,
        cwd: Some(project_dir.to_path_buf()),
        bundle: true,
    };

    let config = FobConfig::load(&args, None).unwrap();
    
    // Environment variables should override file config
    // (but CLI args override everything)
    // Note: CLI args are merged, so out_dir from CLI should win
    assert_eq!(config.out_dir, PathBuf::from("dist")); // CLI wins

    // Note: EnvGuard RAII will clean up environment variables automatically
}
