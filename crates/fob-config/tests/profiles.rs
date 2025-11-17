//! Tests for configuration profiles and merging behavior.

use fob_config::ConfigDiscovery;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn profile_overrides_bundle_options() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
entries = ["src/index.ts"]
minify = false
code_splitting = true

[profiles.production.bundle]
minify = true
code_splitting = false
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("production")
        .expect("load with profile");

    assert!(config.bundle.minify);
    assert!(!config.bundle.code_splitting);
}

#[test]
fn profile_overrides_dev_config() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[dev]
host = "localhost"
port = 3000

[profiles.ci.dev]
host = "0.0.0.0"
port = 8080
open = false
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("ci")
        .expect("load with profile");

    let dev = config.dev.expect("dev config present");
    assert_eq!(dev.host, "0.0.0.0");
    assert_eq!(dev.port, Some(8080));
    assert!(!dev.open);
}

#[test]
fn profile_overrides_settings() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[settings]
log_level = "info"
trace = false

[profiles.debug.settings]
log_level = "trace"
trace = true
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("debug")
        .expect("load with profile");

    assert_eq!(config.settings.log_level.as_deref(), Some("trace"));
    assert!(config.settings.trace);
}

#[test]
fn profile_merges_deeply_nested_objects() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle.cache_config]
enabled = true
max_size = 1000

[profiles.prod.bundle.cache_config]
max_size = 5000
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("prod")
        .expect("load with profile");

    assert!(config.bundle.cache_config.enabled);
    assert_eq!(config.bundle.cache_config.max_size, 5000);
}

#[test]
fn profile_replaces_arrays() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
entries = ["src/index.ts", "src/worker.ts"]
external = ["react", "react-dom"]

[profiles.library.bundle]
entries = ["src/lib.ts"]
external = ["react"]
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("library")
        .expect("load with profile");

    assert_eq!(config.bundle.entries, vec![PathBuf::from("src/lib.ts")]);
    assert_eq!(config.bundle.external, vec!["react"]);
}

#[test]
fn profile_not_found_uses_base_config() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
minify = false
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("nonexistent")
        .expect("load with profile");

    assert!(!config.bundle.minify);
}

#[test]
fn plugin_profile_overrides_config() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[[plugins]]
name = "transformer"
path = "./plugins/transform.wasm"
order = 1

[plugins.config]
mode = "dev"
debug = true

[plugins.profiles.production]
order = 10

[plugins.profiles.production.config]
mode = "prod"
debug = false
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("production")
        .expect("load with profile");

    let plugin = config.bundle.plugins.first().expect("plugin present");
    assert_eq!(plugin.order, 10);
    assert_eq!(plugin.config["mode"], Value::String("prod".into()));
    assert_eq!(plugin.config["debug"], Value::Bool(false));
}

#[test]
fn multiple_plugins_with_profiles() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[[plugins]]
name = "plugin_a"
path = "./a.wasm"
enabled = true

[plugins.profiles.test]
enabled = false

[[plugins]]
name = "plugin_b"
path = "./b.wasm"
order = 1

[plugins.profiles.test]
order = 5
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("test")
        .expect("load with profile");

    assert_eq!(config.bundle.plugins.len(), 2);

    let plugin_a = &config.bundle.plugins[0];
    assert_eq!(plugin_a.name.as_deref(), Some("plugin_a"));
    assert!(!plugin_a.enabled);

    let plugin_b = &config.bundle.plugins[1];
    assert_eq!(plugin_b.name.as_deref(), Some("plugin_b"));
    assert_eq!(plugin_b.order, 5);
}

#[test]
fn top_level_plugins_promoted_to_bundle() {
    let _guard = test_lock().lock().expect("lock");
    let dir = TempDir::new().expect("tempdir");
    fs::write(
        dir.path().join("fob.toml"),
        r#"
[[bundle.plugins]]
name = "bundle_plugin"
path = "./bundle.wasm"

[[plugins]]
name = "top_level_plugin"
path = "./top.wasm"
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path()).load().expect("load");

    assert_eq!(config.bundle.plugins.len(), 2);
    let names: Vec<_> = config
        .bundle
        .plugins
        .iter()
        .filter_map(|p| p.name.as_deref())
        .collect();
    assert!(names.contains(&"bundle_plugin"));
    assert!(names.contains(&"top_level_plugin"));
}
