//! Tests for config file discovery and loading
//!
//! Only TOML and package.json/`fob` formats are supported (JSON/YAML removed in v2.0)

use fob_config::ConfigDiscovery;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn discovers_joy_toml() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("fob.toml"),
        r#"
[bundle]
entries = ["index.ts"]
minify = true
"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    let found = discovery.find().unwrap();
    assert_eq!(found.file_name().unwrap(), "fob.toml");

    let config = discovery.load().unwrap();
    assert_eq!(config.bundle.entries, vec![PathBuf::from("index.ts")]);
    assert!(config.bundle.minify);
}

// JS/TS config discovery tests have been removed; only TOML and package.json are supported.

#[test]
fn discovers_package_json() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{
  "name": "test",
  "fob": {
    "bundle": {
      "entries": ["index.ts"],
      "minify": false
    }
  }
}"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    let found = discovery.find().unwrap();
    assert_eq!(found.file_name().unwrap(), "package.json");

    let config = discovery.load().unwrap();
    assert_eq!(config.bundle.entries, vec![PathBuf::from("index.ts")]);
    assert!(!config.bundle.minify);
}

// JS/TS vs TOML precedence tests have been removed; only TOML and package.json are supported.

#[test]
fn toml_takes_precedence_over_package_json() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("fob.toml"),
        r#"
[bundle]
entries = ["toml.ts"]
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{
  "fob": {
    "bundle": {
      "entries": ["pkg.ts"]
    }
  }
}"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    let found = discovery.find().unwrap();
    assert_eq!(found.file_name().unwrap(), "fob.toml");
}

#[test]
fn returns_not_found_when_no_config() {
    let dir = TempDir::new().unwrap();
    let discovery = ConfigDiscovery::new(dir.path());

    assert!(discovery.find().is_none());

    let result = discovery.load();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("config not found"));
}

#[test]
fn ignores_json_files() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("joy.json"),
        r#"{"bundle": {"entries": ["index.ts"]}}"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    assert!(discovery.find().is_none()); // Not discovered
}

#[test]
fn ignores_yaml_files() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("joy.yaml"),
        r#"bundle:
  entries:
    - index.ts
"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    assert!(discovery.find().is_none()); // Not discovered
}

#[test]
fn ignores_yml_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("joy.yml"), "bundle: {}").unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    assert!(discovery.find().is_none());
}

#[test]
fn ignores_joyrc_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".joyrc"), "[bundle]").unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    assert!(discovery.find().is_none());
}

#[test]
fn loads_toml_with_all_fields() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("fob.toml"),
        r#"
[bundle]
entries = ["src/main.ts", "src/worker.ts"]
output_dir = "build"
minify = true
code_splitting = false
external = ["react", "react-dom"]

[bundle.transform]
typescript = true
jsx = true
target = "es2020"

[dev]
host = "0.0.0.0"
port = 4000

[settings]
log_level = "debug"
trace = true
"#,
    )
    .unwrap();

    let config = ConfigDiscovery::new(dir.path()).load().unwrap();

    assert_eq!(config.bundle.entries.len(), 2);
    assert_eq!(config.bundle.output_dir, PathBuf::from("build"));
    assert!(config.bundle.minify);
    assert!(!config.bundle.code_splitting);
    assert_eq!(config.bundle.external, vec!["react", "react-dom"]);

    assert!(config.bundle.transform.typescript);
    assert!(config.bundle.transform.jsx);

    let dev = config.dev.unwrap();
    assert_eq!(dev.host, "0.0.0.0");
    assert_eq!(dev.port, Some(4000));

    assert_eq!(config.settings.log_level.as_deref(), Some("debug"));
    assert!(config.settings.trace);
}

#[test]
fn package_json_without_joy_field_not_discovered() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{
  "name": "test",
  "version": "1.0.0"
}"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    assert!(discovery.find().is_none());
}

#[test]
fn package_json_with_null_joy_field_not_discovered() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{
  "name": "test",
  "fob": null
}"#,
    )
    .unwrap();

    let discovery = ConfigDiscovery::new(dir.path());
    assert!(discovery.find().is_none());
}
