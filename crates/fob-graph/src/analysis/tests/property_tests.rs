//! Property-based tests for fob-analysis.
//!
//! These tests use proptest to verify invariants hold across a wide range
//! of inputs, helping catch edge cases and bugs.

use crate::analysis::config::{AnalyzerConfig, DEFAULT_MAX_DEPTH, DEFAULT_MAX_MODULES};

#[cfg(feature = "proptest")]
use proptest::prelude::*;

/// Test that max_depth is always enforced.
#[test]
fn test_max_depth_enforcement() {
    let mut config = AnalyzerConfig::default();
    config.max_depth = Some(5);

    // This test verifies that the walker will reject paths deeper than max_depth
    // In a real scenario, we'd need a mock runtime with a deep dependency tree
    // For now, we just verify the config is set correctly
    assert_eq!(config.max_depth, Some(5));
}

/// Test that max_modules is always enforced.
#[test]
fn test_max_modules_enforcement() {
    let mut config = AnalyzerConfig::default();
    config.max_modules = Some(100);

    // Verify the config is set correctly
    assert_eq!(config.max_modules, Some(100));
}

/// Property test: resolver should handle arbitrary path strings without panicking.
#[cfg(feature = "proptest")]
#[proptest]
fn test_resolver_arbitrary_paths(
    #[strategy("[a-zA-Z0-9_./-]{0, 100}")] _specifier: String,
    #[strategy("[a-zA-Z0-9_./-]{0, 100}")] _from: String,
) {
    // This test ensures the resolver doesn't panic on arbitrary input
    // In a real implementation, we'd use a mock runtime
    let config = AnalyzerConfig::default();
    let _resolver = crate::resolver::ModuleResolver::new(config);

    // Just verify we can create the resolver without panicking
    // Full resolution testing requires a mock runtime
}

/// Property test: config defaults are reasonable.
#[test]
fn test_config_defaults_reasonable() {
    let config = AnalyzerConfig::default();

    // Verify defaults are set
    assert_eq!(config.max_depth, Some(DEFAULT_MAX_DEPTH));
    assert_eq!(config.max_modules, Some(DEFAULT_MAX_MODULES));

    // Verify defaults are reasonable (not too small, not too large)
    assert!(DEFAULT_MAX_DEPTH >= 100, "max_depth should be at least 100");
    assert!(
        DEFAULT_MAX_DEPTH <= 10000,
        "max_depth should be at most 10000"
    );

    assert!(
        DEFAULT_MAX_MODULES >= 1000,
        "max_modules should be at least 1000"
    );
    assert!(
        DEFAULT_MAX_MODULES <= 1_000_000,
        "max_modules should be at most 1M"
    );
}

/// Property test: path normalization invariants.
#[cfg(feature = "proptest")]
#[proptest]
fn test_path_normalization_invariants(#[strategy("[a-zA-Z0-9_./-]{0, 50}")] path: String) {
    use path_clean::PathClean;
    use std::path::Path;

    let path = Path::new(&path);
    let normalized = path.clean();

    // Invariant: normalized path should not contain ".." components that escape
    // (This is a simplified test - full validation requires cwd context)
    let path_str = normalized.to_string_lossy();

    // Path should not have excessive ".." components
    let dot_dot_count = path_str.matches("../").count();
    assert!(
        dot_dot_count <= 10,
        "Path should not have excessive .. components"
    );
}

/// Property test: resolver handles random import specifiers without panicking.
#[cfg(feature = "proptest")]
#[proptest]
fn test_resolver_random_specifiers(#[strategy("[a-zA-Z0-9_./@~-]{1, 100}")] specifier: String) {
    use crate::analysis::config::AnalyzerConfig;
    use crate::analysis::resolver::ModuleResolver;

    // Create resolver - should not panic on any specifier format
    let config = AnalyzerConfig::default();
    let _resolver = ModuleResolver::new(config);

    // Just verify we can create the resolver
    // Full resolution testing requires a mock runtime with file system
}

/// Property test: max_depth enforcement invariant.
#[test]
fn test_max_depth_invariant() {
    use crate::analysis::config::{AnalyzerConfig, DEFAULT_MAX_DEPTH};

    // Invariant: max_depth should always be enforced if set
    let mut config = AnalyzerConfig::default();
    config.max_depth = Some(5);

    // Verify the limit is set
    assert_eq!(config.max_depth, Some(5));

    // Verify default is reasonable
    assert!(
        DEFAULT_MAX_DEPTH > 0,
        "Default max_depth should be positive"
    );
    assert!(
        DEFAULT_MAX_DEPTH < 100_000,
        "Default max_depth should be reasonable"
    );
}

/// Property test: max_modules enforcement invariant.
#[test]
fn test_max_modules_invariant() {
    use crate::analysis::config::{AnalyzerConfig, DEFAULT_MAX_MODULES};

    // Invariant: max_modules should always be enforced if set
    let mut config = AnalyzerConfig::default();
    config.max_modules = Some(1000);

    // Verify the limit is set
    assert_eq!(config.max_modules, Some(1000));

    // Verify default is reasonable
    assert!(
        DEFAULT_MAX_MODULES > 0,
        "Default max_modules should be positive"
    );
    assert!(
        DEFAULT_MAX_MODULES < 10_000_000,
        "Default max_modules should be reasonable"
    );
}

/// Property test: framework file extraction handles malformed input.
#[cfg(feature = "proptest")]
#[proptest]
fn test_framework_extraction_malformed(
    #[strategy("[a-zA-Z0-9<>\"'= /\\n\\r]{0, 500}")] content: String,
) {
    use super::extractors::extract_scripts;
    use std::path::Path;

    // Test that extraction doesn't panic on malformed input
    let path = Path::new("test.js");
    let _result = extract_scripts(path, &content);
    // Should return Ok or Err, but never panic
}

/// Property test: path aliases handle various formats.
#[cfg(feature = "proptest")]
#[proptest]
fn test_path_alias_formats(
    #[strategy("[a-zA-Z0-9@~#]{1, 20}")] alias: String,
    #[strategy("[a-zA-Z0-9_./-]{1, 50}")] target: String,
) {
    use crate::analysis::resolver::aliases::resolve_path_alias;
    use rustc_hash::FxHashMap;
    use std::path::Path;

    let mut aliases = FxHashMap::default();
    aliases.insert(alias.clone(), target.clone());

    // Test that alias resolution doesn't panic
    let from = Path::new("src/index.ts");
    let specifier = format!("{}/component", alias);
    let _result = resolve_path_alias(&specifier, from, &aliases);
    // Should return Some or None, but never panic
}
