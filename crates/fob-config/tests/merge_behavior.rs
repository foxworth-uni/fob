//! Tests for value merging logic used in profile overrides.

use fob_config::ConfigDiscovery;
use serde_json::json;
use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn merge_replaces_primitive_values() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_BUNDLE__MINIFY");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
minify = false
shared_chunk_threshold = 1000

[profiles.prod.bundle]
minify = true
shared_chunk_threshold = 5000
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("prod")
        .expect("load with profile");

    assert!(config.bundle.minify);
    assert_eq!(config.bundle.shared_chunk_threshold, 5000);
}

#[test]
fn merge_preserves_unspecified_fields() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_BUNDLE__MINIFY");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
entries = ["src/index.ts"]
minify = false
code_splitting = true

[profiles.prod.bundle]
minify = true
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("prod")
        .expect("load with profile");

    assert!(config.bundle.minify);
    assert!(config.bundle.code_splitting); // preserved
    assert_eq!(config.bundle.entries.len(), 1); // preserved
}

#[test]
fn merge_handles_nested_objects() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_BUNDLE__CACHE_CONFIG__ENABLED");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle.cache_config]
enabled = true
max_size = 1000

[bundle.experimental]
wasm = false
json = true

[profiles.prod.bundle.cache_config]
max_size = 5000

[profiles.prod.bundle.experimental]
wasm = true
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("prod")
        .expect("load with profile");

    // cache_config merged
    assert!(config.bundle.cache_config.enabled);
    assert_eq!(config.bundle.cache_config.max_size, 5000);

    // experimental merged
    assert!(config.bundle.experimental.wasm);
    assert!(config.bundle.experimental.json);
}

#[test]
fn merge_replaces_entire_arrays() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_BUNDLE__EXTERNAL");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
external = ["react", "react-dom", "lodash"]

[profiles.minimal.bundle]
external = ["react"]
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("minimal")
        .expect("load with profile");

    // Array is replaced, not merged
    assert_eq!(config.bundle.external, vec!["react"]);
}

#[test]
fn merge_handles_null_values() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_DEV__HOST");

    let dir = TempDir::new().expect("tempdir");
    fs::write(
        dir.path().join("fob.toml"),
        r#"
[dev]
host = "localhost"
port = 3000

[profiles.ci.dev]
# Note: TOML doesn't support null, so we omit the port field
# This will preserve the base port value
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("ci")
        .expect("load with profile");

    // dev config is still present
    let dev = config.dev.expect("dev config present");
    assert_eq!(dev.host, "localhost");
    // In TOML, omitted fields preserve their base values
    assert_eq!(dev.port, Some(3000));
}

#[test]
fn merge_plugin_config_deeply() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_PLUGINS__CONFIG");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[[plugins]]
name = "example"
path = "./example.wasm"

[plugins.config]
level1 = "base"

[plugins.config.nested]
level2 = "base"
level2b = "base"

[plugins.profiles.prod.config]
level1 = "overridden"

[plugins.profiles.prod.config.nested]
level2 = "overridden"
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("prod")
        .expect("load with profile");

    let plugin = &config.bundle.plugins[0];
    assert_eq!(plugin.config["level1"], json!("overridden"));
    assert_eq!(plugin.config["nested"]["level2"], json!("overridden"));
    assert_eq!(plugin.config["nested"]["level2b"], json!("base")); // preserved
}

#[test]
fn merge_creates_object_from_null() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_DEV__HOST");
    let dir = TempDir::new().expect("tempdir");
    fs::write(
        dir.path().join("fob.toml"),
        r#"
[bundle]
entries = ["src/index.ts"]

[profiles.test.dev]
host = "0.0.0.0"
port = 3000
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("test")
        .expect("load with profile");

    // dev was null, now is an object
    let dev = config.dev.expect("dev config created");
    assert_eq!(dev.host, "0.0.0.0");
    assert_eq!(dev.port, Some(3000));
}

#[test]
fn merge_handles_empty_profile() {
    let _guard = test_lock().lock().expect("lock");
    env::remove_var("JOY_BUNDLE__MINIFY");
    let dir = TempDir::new().expect("tempdir");
    let config_path = dir.path().join("fob.toml");
    fs::write(
        &config_path,
        r#"
[bundle]
minify = true

[profiles.empty]
"#,
    )
    .expect("write config");

    let config = ConfigDiscovery::new(dir.path())
        .load_with_profile("empty")
        .expect("load with profile");

    // Nothing changes
    assert!(config.bundle.minify);
}
