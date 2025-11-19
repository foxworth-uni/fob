//! Configuration system for Fob bundler with multi-source loading.
//!
//! Merges settings from CLI args, environment variables, and config files.
//! Priority: CLI > Environment > File > Defaults

mod conversions;
mod defaults;
mod loading;
mod tests;
mod types;
mod validation;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use defaults::*;
pub use types::*;
pub use validation::*;

/// Fob configuration - loaded from fob.config.json or CLI args.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FobConfig {
    /// Entry points to bundle (e.g., ["src/index.ts"])
    pub entry: Vec<String>,

    /// Output format (esm, cjs, iife)
    #[serde(default = "default_format")]
    pub format: Format,

    /// Output directory
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,

    /// Generate TypeScript declarations
    #[serde(default)]
    pub dts: bool,

    /// Bundle declarations into single .d.ts file (requires dts: true)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dts_bundle: Option<bool>,

    /// External packages to exclude from bundle
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external: Vec<String>,

    /// Target platform
    #[serde(default = "default_platform")]
    pub platform: Platform,

    /// Source map mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sourcemap: Option<SourceMapMode>,

    /// Enable minification
    #[serde(default)]
    pub minify: bool,

    /// JavaScript target version
    #[serde(default = "default_target")]
    pub target: EsTarget,

    /// Global variable name for IIFE bundles (must be valid JS identifier)
    #[schemars(regex(pattern = r"^[a-zA-Z_$][a-zA-Z0-9_$]*$"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_name: Option<String>,

    /// Bundle dependencies into output
    /// - true: Include all dependencies in the bundle
    /// - false: Externalize dependencies (library mode)
    #[serde(default = "default_bundle")]
    pub bundle: bool,

    /// Enable code splitting
    #[serde(default)]
    pub splitting: bool,

    /// Disable tree shaking
    #[serde(default)]
    pub no_treeshake: bool,

    /// Clean output directory before build
    #[serde(default)]
    pub clean: bool,

    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
}

impl FobConfig {
    /// Generate JSON Schema for fob.config.json.
    pub fn json_schema() -> serde_json::Value {
        let schema = schemars::schema_for!(FobConfig);
        serde_json::to_value(schema).expect("Schema serialization should never fail")
    }

    /// Generate example fob.config.json content.
    pub fn example_config() -> String {
        use std::path::PathBuf;
        use types::*;

        serde_json::to_string_pretty(&Self {
            entry: vec!["src/index.ts".to_string(), "src/cli.ts".to_string()],
            format: Format::Esm,
            out_dir: PathBuf::from("dist"),
            dts: true,
            dts_bundle: Some(true),
            external: vec!["react".to_string(), "react-dom".to_string()],
            platform: Platform::Browser,
            sourcemap: Some(SourceMapMode::External),
            minify: true,
            target: EsTarget::Es2020,
            global_name: None,
            bundle: true,
            splitting: true,
            no_treeshake: false,
            clean: true,
            cwd: None,
        })
        .expect("Example config serialization should never fail")
    }
}
