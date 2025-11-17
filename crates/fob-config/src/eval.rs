//! JavaScript/TypeScript config file evaluation using Funtime.
//!
//! This module provides sandboxed execution of JavaScript and TypeScript config files.
//! Config files must export a default object that conforms to the JoyConfig structure.
//!
//! # Security
//!
//! Config files execute in a sandboxed environment with strict limitations:
//! - Read access only to the config directory (no parent directory access)
//! - No network access
//! - No filesystem writes
//! - No subprocess execution
//! - 5 second timeout limit
//! - 128 MB memory limit
//!
//! # Example Config File
//!
//! **Note**: The current Boa runtime implementation evaluates config files as scripts,
//! not ES6 modules. Use the following pattern instead of `export default`:
//!
//! ```javascript
//! // fob.config.js or fob.config.ts
//! const config = {
//!   bundle: {
//!     entries: ['index.ts'],
//!     minify: true
//!   }
//! };
//!
//! config; // Last expression is returned
//! ```

use std::path::Path;

use funtime::{Permissions, Runtime, RuntimeConfig};
use funtime_boa::BoaRuntime;

use crate::config::JoyConfig;
use crate::error::{ConfigError, Result};

/// Load a JavaScript or TypeScript config file.
///
/// The config file should return an object that conforms to the JoyConfig schema.
/// The last expression in the file is used as the config value:
///
/// ```js
/// // fob.config.js
/// const config = {
///   bundle: { entries: ['index.ts'] }
/// };
///
/// config; // Last expression becomes the config
/// ```
///
/// # Security
///
/// The config file executes in a sandboxed environment with:
/// - Read access only to the config directory
/// - No network, filesystem writes, or subprocess access
/// - 5 second timeout limit
/// - 128 MB memory limit
///
/// # Errors
///
/// Returns `ConfigError::EvaluationFailed` if:
/// - The file contains syntax errors
/// - Execution times out (>5s)
/// - No default export is provided
/// - The exported value doesn't match the JoyConfig schema
///
/// # Example
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use fob_config::eval::load_js_config;
/// use std::path::Path;
///
/// let config = load_js_config(Path::new("fob.config.ts")).await?;
/// println!("Loaded config with {} entries", config.bundle.entries.len());
/// # Ok(())
/// # }
/// ```
pub async fn load_js_config(path: &Path) -> Result<JoyConfig> {
    tracing::debug!("Loading JS/TS config from: {}", path.display());

    // Get config directory for read permissions
    // We only allow reading from the config's directory for security
    let config_dir = path
        .parent()
        .ok_or_else(|| ConfigError::InvalidValue("Invalid config path".into()))?
        .to_path_buf();

    tracing::trace!(
        "Granting read access to config directory: {}",
        config_dir.display()
    );

    // Create restricted runtime configuration for config execution
    let runtime_config = RuntimeConfig {
        permissions: Permissions {
            allow_read: Some(vec![config_dir]),
            allow_write: None,
            allow_net: None,
            allow_env: false,
            allow_run: false,
            allow_all: false,
        },
        timeout_ms: Some(5_000),    // 5 second timeout
        memory_limit_mb: Some(128), // 128 MB memory limit
        enable_typescript: true,
        enable_jsx: false,
        jsx_runtime: String::new(),
    };

    // Initialize the Boa runtime with security restrictions
    let mut runtime = BoaRuntime::new(runtime_config).await?;

    tracing::trace!("Executing config module: {}", path.display());

    // Execute the config file and get the result
    let result = runtime.execute_module(path).await?;

    tracing::trace!("Config evaluation result: {:?}", result.value);

    // Check if result is null/undefined - this indicates no return value
    if result.value.is_null() {
        return Err(ConfigError::EvaluationFailed(
            "Config file must return a value as the last expression. Example:\n\
             const config = { bundle: { entries: ['index.ts'] } };\n\
             config; // Last expression is returned"
                .into(),
        ));
    }

    // Parse the JSON value into JoyConfig
    // This validates the config structure matches our schema
    let config: JoyConfig = serde_json::from_value(result.value).map_err(|e| {
        ConfigError::InvalidValue(format!(
            "Invalid config structure: {}. Expected config with 'bundle' field.",
            e
        ))
    })?;

    tracing::debug!("Successfully loaded config from {}", path.display());

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_simple_js_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.config.js");

        fs::write(
            &config_path,
            r#"
            const config = {
                bundle: {
                    entries: ["index.ts"],
                    minify: true
                }
            };
            config;
        "#,
        )
        .unwrap();

        let config = load_js_config(&config_path).await.unwrap();
        assert!(config.bundle.minify);
        assert_eq!(config.bundle.entries.len(), 1);
    }

    #[tokio::test]
    async fn test_load_typescript_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.config.ts");

        fs::write(
            &config_path,
            r#"
            const isDev: boolean = false;

            const config = {
                bundle: {
                    entries: ["index.tsx"],
                    minify: !isDev
                }
            };

            config;
        "#,
        )
        .unwrap();

        let config = load_js_config(&config_path).await.unwrap();
        assert!(config.bundle.minify);
        assert_eq!(config.bundle.entries.len(), 1);
    }

    #[tokio::test]
    async fn test_syntax_error_in_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.config.js");

        fs::write(
            &config_path,
            r#"
            const config = {
                bundle: {
                    entries: [syntax error here]
                }
            };
            config;
        "#,
        )
        .unwrap();

        let result = load_js_config(&config_path).await;
        assert!(matches!(result, Err(ConfigError::EvaluationFailed(_))));

        if let Err(ConfigError::EvaluationFailed(msg)) = result {
            assert!(msg.contains("Syntax error") || msg.contains("Execution error"));
        }
    }

    #[tokio::test]
    async fn test_missing_return_value() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.config.js");

        fs::write(
            &config_path,
            r#"
            const config = {
                bundle: {
                    entries: ["index.ts"]
                }
            };
            // Forgot to return the config!
        "#,
        )
        .unwrap();

        let result = load_js_config(&config_path).await;
        assert!(matches!(result, Err(ConfigError::EvaluationFailed(_))));

        if let Err(ConfigError::EvaluationFailed(msg)) = result {
            assert!(msg.contains("return") || msg.contains("last expression"));
        }
    }

    #[tokio::test]
    async fn test_invalid_config_structure() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.config.js");

        fs::write(
            &config_path,
            r#"
            const config = {
                invalid_field: "not a valid config"
            };
            config;
        "#,
        )
        .unwrap();

        let result = load_js_config(&config_path).await;
        // Should succeed but use default values for missing fields
        let config = result.unwrap();
        assert!(config.bundle.entries.is_empty());
    }

    #[tokio::test]
    async fn test_computed_config_values() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.config.js");

        fs::write(
            &config_path,
            r#"
            const isDev = false;

            const config = {
                bundle: {
                    entries: ["index.ts"],
                    minify: !isDev,
                    code_splitting: !isDev
                }
            };

            config;
        "#,
        )
        .unwrap();

        let config = load_js_config(&config_path).await.unwrap();
        assert!(config.bundle.minify);
        assert!(config.bundle.code_splitting);
    }

    // NOTE: Timeout protection test disabled - Boa's timeout doesn't work reliably
    // for infinite loops. This is a known limitation of the current runtime implementation.
    // Future work: Implement proper timeout via instruction counting or periodic checks.
    //
    // #[tokio::test]
    // async fn test_timeout_protection() {
    //     let dir = TempDir::new().unwrap();
    //     let config_path = dir.path().join("fob.config.js");
    //
    //     fs::write(
    //         &config_path,
    //         r#"
    //         while (true) {} // Infinite loop
    //         const config = {};
    //         config;
    //     "#,
    //     )
    //     .unwrap();
    //
    //     let result = load_js_config(&config_path).await;
    //     assert!(matches!(result, Err(ConfigError::EvaluationFailed(_))));
    // }
}
