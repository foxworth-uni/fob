//! Tests for default values and edge cases.

use fob_config::{
    BundleOptions, CacheConfig, DevConfig, EsTarget, ExperimentalOptions, GlobalSettings,
    JoyConfig, JsxRuntime, OutputFormat, Platform, SourceMapOptions, TransformOptions,
    TypeCheckMode,
};
use std::path::PathBuf;

#[test]
fn joy_config_defaults() {
    let config = JoyConfig::default();
    assert!(config.bundle.entries.is_empty());
    assert_eq!(config.bundle.output_dir, PathBuf::from("dist"));
    assert!(config.profiles.is_empty());
    assert!(config.dev.is_none());
}

#[test]
fn bundle_options_defaults() {
    let opts = BundleOptions::default();
    assert!(opts.entries.is_empty());
    assert_eq!(opts.output_dir, PathBuf::from("dist"));
    assert_eq!(opts.format, OutputFormat::Esm);
    assert_eq!(opts.platform, Platform::Browser);
    assert!(opts.code_splitting);
    assert!(!opts.minify);
    assert_eq!(opts.source_maps, SourceMapOptions::External);
    assert_eq!(opts.shared_chunk_threshold, 20_000);
    assert!(opts.external.is_empty());
    assert!(opts.plugins.is_empty());
}

#[test]
fn cache_config_defaults() {
    let cache = CacheConfig::default();
    assert!(cache.enabled);
    assert!(cache.cache_dir.is_none());
    assert_eq!(cache.max_size, 0);
}

#[test]
fn dev_config_defaults() {
    let dev = DevConfig::default();
    assert_eq!(dev.host, "127.0.0.1");
    assert!(dev.port.is_none());
    assert!(dev.open);
    assert_eq!(dev.hmr_path, "/__joy/hmr");
    assert!(dev.watch_paths.is_empty());
    assert_eq!(dev.debounce_ms, 100);
    assert!(dev.proxy.is_empty());
    assert!(dev.cors.is_none());
    assert!(dev.https.is_none());
}

#[test]
fn global_settings_defaults() {
    let settings = GlobalSettings::default();
    assert!(settings.log_level.is_none());
    assert!(settings.log_format.is_none());
    assert!(!settings.trace);
    assert!(settings.parallel_jobs.is_none());
    assert!(settings.environment.is_empty());
}

#[test]
fn transform_options_defaults() {
    let transform = TransformOptions::default();
    assert!(transform.typescript);
    assert!(transform.jsx);
    assert_eq!(transform.target, EsTarget::ES2022);
    assert_eq!(transform.type_check, TypeCheckMode::None);
    assert_eq!(transform.jsx_runtime, JsxRuntime::Automatic);
    assert!(transform.jsx_import_source.is_none());
    assert!(transform.jsx_dev);
}

#[test]
fn experimental_options_defaults() {
    let exp = ExperimentalOptions::default();
    assert!(!exp.wasm);
    assert!(!exp.css);
    // Note: json defaults to false via Default trait, but true when deserialized
    assert!(!exp.json);
    assert!(!exp.analysis);
}

#[test]
fn output_format_enum() {
    assert_eq!(OutputFormat::default(), OutputFormat::Esm);
    assert_ne!(OutputFormat::Esm, OutputFormat::PreserveModules);
}

#[test]
fn platform_enum() {
    assert_eq!(Platform::default(), Platform::Browser);
    assert_ne!(Platform::Browser, Platform::Node);
    assert_ne!(Platform::Browser, Platform::Worker);
    assert_ne!(Platform::Browser, Platform::Deno);
}

#[test]
fn source_map_options_enum() {
    assert_eq!(SourceMapOptions::default(), SourceMapOptions::External);
    assert_ne!(SourceMapOptions::None, SourceMapOptions::Inline);
    assert_ne!(
        SourceMapOptions::External,
        SourceMapOptions::ExternalWithContent
    );
}

#[test]
fn es_target_enum() {
    assert_eq!(EsTarget::default(), EsTarget::ES2022);
}

#[test]
fn type_check_mode_enum() {
    assert_eq!(TypeCheckMode::default(), TypeCheckMode::None);
}

#[test]
fn jsx_runtime_enum() {
    assert_eq!(JsxRuntime::default(), JsxRuntime::Automatic);
    assert_ne!(JsxRuntime::Automatic, JsxRuntime::Classic);
}

#[test]
fn empty_profiles_map() {
    let config = JoyConfig::default();
    let result = config.materialize_profile(Some("nonexistent"));
    assert!(result.is_ok());
}

#[test]
fn settings_environment_variables() {
    let mut settings = GlobalSettings::default();
    settings
        .environment
        .insert("NODE_ENV".into(), "production".into());
    settings
        .environment
        .insert("API_KEY".into(), "secret".into());

    assert_eq!(settings.environment.len(), 2);
    assert_eq!(settings.environment.get("NODE_ENV").unwrap(), "production");
}

#[test]
fn dev_proxy_configuration() {
    let mut dev = DevConfig::default();
    dev.proxy.insert("/api".into(), Default::default());

    assert_eq!(dev.proxy.len(), 1);
    assert!(dev.proxy.contains_key("/api"));
}

#[test]
fn multiple_entry_points() {
    let mut opts = BundleOptions::default();
    opts.entries = vec![
        PathBuf::from("src/main.ts"),
        PathBuf::from("src/worker.ts"),
        PathBuf::from("src/admin.ts"),
    ];

    assert_eq!(opts.entries.len(), 3);
}

#[test]
fn multiple_external_modules() {
    let mut opts = BundleOptions::default();
    opts.external = vec![
        "react".into(),
        "react-dom".into(),
        "lodash".into(),
        "@emotion/react".into(),
    ];

    assert_eq!(opts.external.len(), 4);
    assert!(opts.external.contains(&"react".into()));
}

#[test]
fn custom_shared_chunk_threshold() {
    let mut opts = BundleOptions::default();
    opts.shared_chunk_threshold = 50_000; // 50KB

    assert_eq!(opts.shared_chunk_threshold, 50_000);
}

#[test]
fn all_source_map_options() {
    let options = [
        SourceMapOptions::None,
        SourceMapOptions::Inline,
        SourceMapOptions::External,
        SourceMapOptions::ExternalWithContent,
    ];

    assert_eq!(options.len(), 4);
}

#[test]
fn all_platforms() {
    let platforms = [
        Platform::Browser,
        Platform::Node,
        Platform::Worker,
        Platform::Deno,
    ];

    assert_eq!(platforms.len(), 4);
}

#[test]
fn all_es_targets() {
    let targets = [
        EsTarget::ES2015,
        EsTarget::ES2016,
        EsTarget::ES2017,
        EsTarget::ES2018,
        EsTarget::ES2019,
        EsTarget::ES2020,
        EsTarget::ES2021,
        EsTarget::ES2022,
        EsTarget::ES2023,
        EsTarget::ES2024,
        EsTarget::ESNext,
    ];

    assert_eq!(targets.len(), 11);
}
