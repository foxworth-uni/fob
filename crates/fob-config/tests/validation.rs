//! Tests for configuration validation.

use fob_config::{ConfigError, ConfigValidator, FsValidator, JoyConfig, PluginOptions};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn validate_catches_missing_entry() {
    let dir = TempDir::new().expect("tempdir");
    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/nonexistent.ts")];

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::EntryNotFound { path } => {
            assert!(path.ends_with("src/nonexistent.ts"));
        }
        _ => panic!("expected EntryNotFound error"),
    }
}

#[test]
fn validate_succeeds_when_entry_exists() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    let entry_path = src_dir.join("index.ts");
    fs::write(&entry_path, "export {};").expect("write entry");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_ok());
}

#[test]
fn validate_catches_missing_plugin() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("index.ts"), "").expect("write entry");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];
    cfg.bundle.plugins.push(PluginOptions {
        path: PathBuf::from("plugins/missing.wasm"),
        name: Some("test".into()),
        backend: Default::default(),
        config: Default::default(),
        order: 0,
        enabled: true,
        pool_size: None,
        max_memory_bytes: None,
        timeout_ms: None,
        profiles: Default::default(),
    });

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::PluginNotFound { path } => {
            assert!(path.ends_with("plugins/missing.wasm"));
        }
        _ => panic!("expected PluginNotFound error"),
    }
}

#[test]
fn validate_succeeds_when_plugin_exists() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("index.ts"), "").expect("write entry");

    let plugins_dir = dir.path().join("plugins");
    fs::create_dir(&plugins_dir).expect("create plugins dir");
    let plugin_path = plugins_dir.join("example.wasm");
    fs::write(&plugin_path, &[0x00, 0x61, 0x73, 0x6d]).expect("write plugin");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];
    cfg.bundle.plugins.push(PluginOptions {
        path: PathBuf::from("plugins/example.wasm"),
        name: Some("example".into()),
        backend: Default::default(),
        config: Default::default(),
        order: 0,
        enabled: true,
        pool_size: None,
        max_memory_bytes: None,
        timeout_ms: None,
        profiles: Default::default(),
    });

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_ok());
}

#[test]
fn validate_catches_missing_cache_dir() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("index.ts"), "").expect("write entry");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];
    cfg.bundle.cache_config.cache_dir = Some(PathBuf::from(".cache/fob"));

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::CacheDirNotWritable { path } => {
            assert!(path.ends_with(".cache/fob"));
        }
        _ => panic!("expected CacheDirNotWritable error"),
    }
}

#[test]
fn validate_succeeds_when_cache_dir_exists() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("index.ts"), "").expect("write entry");

    let cache_dir = dir.path().join(".cache");
    fs::create_dir(&cache_dir).expect("create cache dir");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];
    cfg.bundle.cache_config.cache_dir = Some(PathBuf::from(".cache"));

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_ok());
}

#[test]
fn validate_succeeds_with_no_cache_dir() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("index.ts"), "").expect("write entry");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];
    cfg.bundle.cache_config.cache_dir = None;

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_ok());
}

#[test]
fn validate_checks_multiple_entries() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("a.ts"), "").expect("write a.ts");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![
        PathBuf::from("src/a.ts"),
        PathBuf::from("src/b.ts"), // missing
    ];

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::EntryNotFound { path } => {
            assert!(path.ends_with("src/b.ts"));
        }
        _ => panic!("expected EntryNotFound error"),
    }
}

#[test]
fn validate_checks_multiple_plugins() {
    let dir = TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).expect("create src dir");
    fs::write(src_dir.join("index.ts"), "").expect("write entry");

    let plugins_dir = dir.path().join("plugins");
    fs::create_dir(&plugins_dir).expect("create plugins dir");
    fs::write(plugins_dir.join("a.wasm"), &[0x00]).expect("write a.wasm");

    let mut cfg = JoyConfig::default();
    cfg.bundle.entries = vec![PathBuf::from("src/index.ts")];
    cfg.bundle.plugins.push(PluginOptions {
        path: PathBuf::from("plugins/a.wasm"),
        name: None,
        backend: Default::default(),
        config: Default::default(),
        order: 0,
        enabled: true,
        pool_size: None,
        max_memory_bytes: None,
        timeout_ms: None,
        profiles: Default::default(),
    });
    cfg.bundle.plugins.push(PluginOptions {
        path: PathBuf::from("plugins/b.wasm"), // missing
        name: None,
        backend: Default::default(),
        config: Default::default(),
        order: 0,
        enabled: true,
        pool_size: None,
        max_memory_bytes: None,
        timeout_ms: None,
        profiles: Default::default(),
    });

    let result = FsValidator::new(dir.path()).validate(&cfg.bundle);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::PluginNotFound { path } => {
            assert!(path.ends_with("plugins/b.wasm"));
        }
        _ => panic!("expected PluginNotFound error"),
    }
}
