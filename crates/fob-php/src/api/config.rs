//! Bundle configuration types

use crate::api::primitives::{CodeSplittingConfig, EntryMode};
use crate::api::utils::{
    normalize_string, parse_entry_mode_normalized, parse_format_normalized,
    parse_platform_normalized, zval_to_bool, zval_to_int, zval_to_string,
};
use crate::types::OutputFormat;
use ext_php_rs::prelude::*;
use ext_php_rs::types::ZendHashTable;

/// MDX compilation options
#[derive(Debug, Clone, Default)]
pub struct MdxOptions {
    pub gfm: Option<bool>,
    pub footnotes: Option<bool>,
    pub math: Option<bool>,
    pub jsx_runtime: Option<String>,
    pub use_default_plugins: Option<bool>,
}

/// Bundle configuration
#[derive(Debug, Clone, Default)]
pub struct BundleConfig {
    pub entries: Vec<String>,
    pub output_dir: Option<String>,
    pub format: Option<OutputFormat>,
    pub sourcemap: Option<String>,
    pub platform: Option<String>,
    pub minify: Option<bool>,
    pub cwd: Option<String>,
    pub mdx: Option<MdxOptions>,
    pub entry_mode: Option<EntryMode>,
    pub code_splitting: Option<CodeSplittingConfig>,
    pub external: Option<Vec<String>>,
    pub external_from_manifest: Option<bool>,
}

impl BundleConfig {
    /// Parse from PHP array (ZendHashTable)
    pub fn from_php_array(arr: &ZendHashTable) -> PhpResult<Self> {
        let mut config = Self::default();

        // Parse entries - accept both single string and array
        if let Some(entries_zval) = arr.get("entries") {
            if let Some(entries_arr) = entries_zval.array() {
                config.entries = crate::api::utils::array_to_strings(entries_arr);
            } else if let Some(entry_str) = zval_to_string(entries_zval) {
                config.entries = vec![entry_str];
            }
        }

        // Parse optional fields
        if let Some(v) = arr.get("output_dir") {
            config.output_dir = zval_to_string(v);
        }

        if let Some(format_zval) = arr.get("format") {
            if let Some(format_str) = zval_to_string(format_zval) {
                config.format = parse_format_normalized(&format_str);
            }
        }

        if let Some(v) = arr.get("sourcemap") {
            if let Some(s) = zval_to_string(v) {
                config.sourcemap = Some(normalize_string(&s));
            } else if let Some(b) = zval_to_bool(v) {
                config.sourcemap = Some(if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                });
            }
        }

        if let Some(v) = arr.get("platform") {
            if let Some(s) = zval_to_string(v) {
                config.platform = parse_platform_normalized(&s);
            }
        }

        if let Some(v) = arr.get("minify") {
            config.minify = zval_to_bool(v);
        }

        if let Some(v) = arr.get("cwd") {
            config.cwd = zval_to_string(v);
        }

        // Parse external packages - accept both single string and array
        if let Some(external_zval) = arr.get("external") {
            if let Some(external_arr) = external_zval.array() {
                config.external = Some(crate::api::utils::array_to_strings(external_arr));
            } else if let Some(external_str) = zval_to_string(external_zval) {
                config.external = Some(vec![external_str]);
            }
        }

        if let Some(v) = arr.get("external_from_manifest") {
            config.external_from_manifest = zval_to_bool(v);
        }

        // Parse entry_mode with normalization
        if let Some(mode_zval) = arr.get("entry_mode") {
            if let Some(mode_str) = zval_to_string(mode_zval) {
                config.entry_mode = parse_entry_mode_normalized(&mode_str);
            }
        }

        // Parse code_splitting
        if let Some(cs_zval) = arr.get("code_splitting") {
            if let Some(cs_arr) = cs_zval.array() {
                let min_size = cs_arr
                    .get("min_size")
                    .and_then(zval_to_int)
                    .map(|v| v as u32)
                    .unwrap_or(20_000);
                let min_imports = cs_arr
                    .get("min_imports")
                    .and_then(zval_to_int)
                    .map(|v| v as u32)
                    .unwrap_or(2);
                config.code_splitting = Some(CodeSplittingConfig {
                    min_size,
                    min_imports,
                });
            }
        }

        // Parse MDX options
        if let Some(mdx_zval) = arr.get("mdx") {
            if let Some(mdx_arr) = mdx_zval.array() {
                let mut mdx_opts = MdxOptions::default();
                if let Some(v) = mdx_arr.get("gfm") {
                    mdx_opts.gfm = zval_to_bool(v);
                }
                if let Some(v) = mdx_arr.get("footnotes") {
                    mdx_opts.footnotes = zval_to_bool(v);
                }
                if let Some(v) = mdx_arr.get("math") {
                    mdx_opts.math = zval_to_bool(v);
                }
                if let Some(v) = mdx_arr.get("jsx_runtime") {
                    mdx_opts.jsx_runtime = zval_to_string(v);
                }
                if let Some(v) = mdx_arr.get("use_default_plugins") {
                    mdx_opts.use_default_plugins = zval_to_bool(v);
                }
                config.mdx = Some(mdx_opts);
            }
        }

        Ok(config)
    }
}
