//! Bundle configuration types

use crate::api::primitives::{CodeSplittingConfig, EntryMode};
use crate::api::utils::{
    normalize_string, parse_entry_mode_normalized, parse_format_normalized,
    parse_platform_normalized, zval_to_bool, zval_to_int, zval_to_string,
};
use crate::types::OutputFormat;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};
use std::collections::HashMap;

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
    /// Virtual files mapping (path → content)
    /// Used for inline content entries. Keys use "virtual:" prefix.
    pub virtual_files: Option<HashMap<String, String>>,
}

/// Parsed entry configuration with path resolution
#[derive(Debug)]
struct ParsedEntries {
    /// Entry paths including virtual paths (prefixed with "virtual:")
    paths: Vec<String>,
    /// Virtual file content mapping (path → content)
    virtual_files: Option<HashMap<String, String>>,
}

/// Parse flexible entries input: string, array, or Entry arrays with content
///
/// Supports:
/// - `"src/index.js"` - single string path
/// - `["a.js", "b.js"]` - array of paths
/// - `["content" => "console.log('hi');", "name" => "main.js"]` - inline content
/// - `[["path" => "a.js"], ["content" => "..."]]` - mixed entries
fn parse_entries_flexible(entries_zval: &Zval) -> PhpResult<ParsedEntries> {
    let mut paths = Vec::new();
    let mut virtual_files = HashMap::new();
    let mut virtual_counter = 0;

    // Try as array first
    if let Some(entries_arr) = entries_zval.array() {
        for (_, item) in entries_arr.iter() {
            if let Some(item_arr) = item.array() {
                // Entry array with content or path
                if let Some(content_zval) = item_arr.get("content") {
                    if let Some(content_str) = zval_to_string(content_zval) {
                        let loader = item_arr
                            .get("loader")
                            .and_then(zval_to_string)
                            .unwrap_or_else(|| "js".to_string());
                        let name = item_arr
                            .get("name")
                            .and_then(zval_to_string)
                            .unwrap_or_else(|| format!("entry-{}.{}", virtual_counter, loader));

                        let virtual_path = if name.starts_with("virtual:") {
                            name
                        } else {
                            format!("virtual:{}", name)
                        };
                        paths.push(virtual_path.clone());
                        virtual_files.insert(virtual_path, content_str);
                        virtual_counter += 1;
                    }
                } else if let Some(path_zval) = item_arr.get("path") {
                    if let Some(path_str) = zval_to_string(path_zval) {
                        paths.push(path_str);
                    }
                } else {
                    return Err(PhpException::default(
                        "Entry array must have 'path' or 'content'".to_string(),
                    ));
                }
            } else if let Some(path_str) = zval_to_string(item) {
                // String entry
                paths.push(path_str);
            }
        }
    } else if let Some(entry_str) = zval_to_string(entries_zval) {
        // Single string entry
        paths.push(entry_str);
    }

    let vf = if virtual_files.is_empty() {
        None
    } else {
        Some(virtual_files)
    };
    Ok(ParsedEntries {
        paths,
        virtual_files: vf,
    })
}

impl BundleConfig {
    /// Parse from PHP array (ZendHashTable)
    pub fn from_php_array(arr: &ZendHashTable) -> PhpResult<Self> {
        let mut config = Self::default();

        // Parse entries with flexible input support
        if let Some(entries_zval) = arr.get("entries") {
            let parsed = parse_entries_flexible(entries_zval)?;
            config.entries = parsed.paths;
            config.virtual_files = parsed.virtual_files;
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
