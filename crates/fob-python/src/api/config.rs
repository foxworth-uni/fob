//! Bundle configuration types

use crate::api::primitives::{CodeSplittingConfig, EntryMode};
use crate::api::utils::{
    normalize_string, parse_entry_mode_normalized, parse_format_normalized,
    parse_platform_normalized, py_to_path_string, py_to_path_strings,
};
use crate::types::OutputFormat;
use pyo3::Bound;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
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

/// Parse flexible entries input: string, Path, list, or Entry dicts with content
///
/// Supports:
/// - `"src/index.js"` - single string path
/// - `Path("src/index.js")` - pathlib.Path
/// - `["a.js", "b.js"]` - list of paths
/// - `{"content": "console.log('hi');", "name": "main.js"}` - inline content
/// - `[{"path": "a.js"}, {"content": "..."}]` - mixed entries
fn parse_entries_flexible(entries_obj: &Bound<'_, PyAny>) -> PyResult<ParsedEntries> {
    let mut paths = Vec::new();
    let mut virtual_files = HashMap::new();
    let mut virtual_counter = 0;

    // Normalize to list
    let items: Vec<Bound<'_, PyAny>> = if let Ok(list) = entries_obj.cast::<PyList>() {
        list.iter().collect()
    } else {
        vec![entries_obj.clone()]
    };

    for item in items {
        if let Ok(dict) = item.cast::<PyDict>() {
            // Entry dict with content or path
            if let Some(content) = dict.get_item("content")? {
                // Inline content entry
                let content_str: String = content.extract()?;
                let loader: String = dict
                    .get_item("loader")?
                    .map(|l| l.extract().unwrap_or_else(|_| "js".to_string()))
                    .unwrap_or_else(|| "js".to_string());
                let name: String = dict
                    .get_item("name")?
                    .map(|n| {
                        n.extract()
                            .unwrap_or_else(|_| format!("entry-{}.{}", virtual_counter, loader))
                    })
                    .unwrap_or_else(|| format!("entry-{}.{}", virtual_counter, loader));

                let virtual_path = if name.starts_with("virtual:") {
                    name
                } else {
                    format!("virtual:{}", name)
                };
                paths.push(virtual_path.clone());
                virtual_files.insert(virtual_path, content_str);
                virtual_counter += 1;
            } else if let Some(path) = dict.get_item("path")? {
                // Explicit path entry
                paths.push(py_to_path_string(&path)?);
            } else {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Entry dict must have 'path' or 'content'",
                ));
            }
        } else {
            // String or Path entry
            paths.push(py_to_path_string(&item)?);
        }
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
    /// Parse from Python dict
    pub fn from_py_dict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut config = Self::default();

        // Parse entries with flexible input support
        if let Some(entries_obj) = dict.get_item("entries")? {
            let parsed = parse_entries_flexible(&entries_obj)?;
            config.entries = parsed.paths;
            config.virtual_files = parsed.virtual_files;
        }

        // Parse optional fields with path support
        if let Some(v) = dict.get_item("output_dir")? {
            config.output_dir = py_to_path_string(&v).ok();
        }

        if let Some(format_str) = dict.get_item("format")? {
            if let Ok(s) = format_str.extract::<String>() {
                config.format = parse_format_normalized(&s);
            }
        }

        if let Some(v) = dict.get_item("sourcemap")? {
            if let Ok(s) = v.extract::<String>() {
                config.sourcemap = Some(normalize_string(&s));
            } else if let Ok(b) = v.extract::<bool>() {
                config.sourcemap = Some(if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                });
            }
        }

        if let Some(v) = dict.get_item("platform")? {
            if let Ok(s) = v.extract::<String>() {
                config.platform = parse_platform_normalized(&s);
            }
        }

        if let Some(v) = dict.get_item("minify")? {
            config.minify = v.extract::<bool>().ok();
        }

        if let Some(v) = dict.get_item("cwd")? {
            config.cwd = py_to_path_string(&v).ok();
        }

        // Parse external packages - accept both single string and list
        if let Some(external) = dict.get_item("external")? {
            match py_to_path_strings(&external) {
                Ok(paths) => config.external = Some(paths),
                Err(_) => {
                    // Try as single string
                    if let Ok(single) = py_to_path_string(&external) {
                        config.external = Some(vec![single]);
                    }
                }
            }
        }

        if let Some(v) = dict.get_item("external_from_manifest")? {
            config.external_from_manifest = v.extract::<bool>().ok();
        }

        // Parse entry_mode with normalization
        if let Some(mode_str) = dict.get_item("entry_mode")? {
            if let Ok(s) = mode_str.extract::<String>() {
                config.entry_mode = parse_entry_mode_normalized(&s);
            }
        }

        // Parse code_splitting
        if let Some(cs_bound) = dict.get_item("code_splitting")? {
            if let Ok(cs) = cs_bound.cast::<PyDict>() {
                let min_size = cs
                    .get_item("min_size")?
                    .and_then(|v| v.extract::<u32>().ok())
                    .unwrap_or(20_000);
                let min_imports = cs
                    .get_item("min_imports")?
                    .and_then(|v| v.extract::<u32>().ok())
                    .unwrap_or(2);
                config.code_splitting = Some(CodeSplittingConfig {
                    min_size,
                    min_imports,
                });
            }
        }

        // Parse MDX options
        if let Some(mdx_bound) = dict.get_item("mdx")? {
            if let Ok(mdx) = mdx_bound.cast::<PyDict>() {
                let mut mdx_opts = MdxOptions::default();
                if let Some(v) = mdx.get_item("gfm")? {
                    mdx_opts.gfm = v.extract::<bool>().ok();
                }
                if let Some(v) = mdx.get_item("footnotes")? {
                    mdx_opts.footnotes = v.extract::<bool>().ok();
                }
                if let Some(v) = mdx.get_item("math")? {
                    mdx_opts.math = v.extract::<bool>().ok();
                }
                if let Some(v) = mdx.get_item("jsx_runtime")? {
                    mdx_opts.jsx_runtime = v.extract::<String>().ok();
                }
                if let Some(v) = mdx.get_item("use_default_plugins")? {
                    mdx_opts.use_default_plugins = v.extract::<bool>().ok();
                }
                config.mdx = Some(mdx_opts);
            }
        }

        Ok(config)
    }
}

pub fn register_config_types(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
