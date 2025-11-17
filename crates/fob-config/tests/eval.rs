#![cfg(feature = "eval")]

//! Integration tests for JavaScript/TypeScript config file evaluation.
//!
//! These tests verify that JS/TS config files can be loaded, executed safely,
//! and produce valid JoyConfig structures.

use fob_config::eval::load_js_config;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_typescript_with_types() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.ts"),
        r#"
        const isDev: boolean = false;

        const config = {
            bundle: {
                entries: ["index.tsx", "worker.tsx"],
                minify: !isDev
            }
        };

        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.ts");
    let config = load_js_config(&config_path).await.unwrap();

    assert!(config.bundle.minify);
    assert_eq!(config.bundle.entries.len(), 2);
}

#[tokio::test]
async fn test_javascript_with_computed_values() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.js"),
        r#"
        const isProd = true;
        const entries = isProd ? ["index.ts"] : ["index.ts", "dev.ts"];

        const config = {
            bundle: {
                entries,
                minify: isProd,
                code_splitting: isProd
            }
        };

        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.js");
    let config = load_js_config(&config_path).await.unwrap();

    assert!(config.bundle.minify);
    assert!(config.bundle.code_splitting);
    assert_eq!(config.bundle.entries.len(), 1);
}

#[tokio::test]
async fn test_no_default_export() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.js"),
        r#"
        const config = { bundle: {} };
        // Forgot to export!
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.js");
    let result = load_js_config(&config_path).await;
    assert!(result.is_err());
}

// NOTE: Timeout protection test disabled - see eval.rs unit tests for explanation
// #[tokio::test]
// async fn test_timeout_protection() {
//     let dir = TempDir::new().unwrap();
//
//     fs::write(
//         dir.path().join("fob.config.js"),
//         r#"
//         while (true) {} // Infinite loop
//         const config = {};
//         config;
//     "#,
//     )
//     .unwrap();
//
//     let config_path = dir.path().join("fob.config.js");
//     let result = load_js_config(&config_path).await;
//     assert!(result.is_err());
// }

#[tokio::test]
async fn test_syntax_error() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.js"),
        r#"
        const config = {
            bundle: {
                entries: [not valid syntax]
            }
        };
        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.js");
    let result = load_js_config(&config_path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_config_structure() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.js"),
        r#"
        const config = {
            not_a_valid_field: "value"
        };
        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.js");
    let result = load_js_config(&config_path).await;
    // Should succeed but use default values for missing fields
    let config = result.unwrap();
    assert!(config.bundle.entries.is_empty());
}

#[tokio::test]
async fn test_complex_typescript_config() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.ts"),
        r#"
        const env: string = 'production';

        const config = {
            bundle: {
                entries: ['src/index.ts', 'src/worker.ts'],
                minify: env === 'production',
                code_splitting: env === 'production'
            }
        };

        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.ts");
    let config = load_js_config(&config_path).await.unwrap();

    assert!(config.bundle.minify);
    assert!(config.bundle.code_splitting);
    assert_eq!(config.bundle.entries.len(), 2);
}

#[tokio::test]
async fn test_config_with_helpers() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.js"),
        r#"
        function getEntries(isDev) {
            const base = ['index.ts'];
            if (isDev) {
                base.push('dev-tools.ts');
            }
            return base;
        }

        const config = {
            bundle: {
                entries: getEntries(false),
                minify: true
            }
        };

        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.js");
    let config = load_js_config(&config_path).await.unwrap();

    assert_eq!(config.bundle.entries.len(), 1);
    assert!(config.bundle.minify);
}

#[tokio::test]
async fn test_config_with_profiles() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.ts"),
        r#"
        const config = {
            bundle: {
                entries: ['index.ts'],
                minify: false
            },
            profiles: {
                production: {
                    bundle: {
                        minify: true,
                        code_splitting: true
                    }
                }
            }
        };

        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.ts");
    let config = load_js_config(&config_path).await.unwrap();

    // Base config should have minify: false
    assert!(!config.bundle.minify);

    // Check that profiles are present
    assert!(config.profiles.contains_key("production"));
}

#[tokio::test]
async fn test_empty_bundle_entries_defaults() {
    let dir = TempDir::new().unwrap();

    fs::write(
        dir.path().join("fob.config.js"),
        r#"
        const config = {
            bundle: {}
        };
        config;
    "#,
    )
    .unwrap();

    let config_path = dir.path().join("fob.config.js");
    let config = load_js_config(&config_path).await.unwrap();

    // Should use default empty entries
    assert!(config.bundle.entries.is_empty());
}
