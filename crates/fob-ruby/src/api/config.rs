//! Bundle configuration types

use crate::api::primitives::{CodeSplittingConfig, EntryMode};
use crate::types::OutputFormat;
use magnus::{RHash, Ruby, TryConvert};

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
    /// Parse from Ruby hash (keyword arguments)
    pub fn from_ruby_hash(ruby: &Ruby, hash: RHash) -> Result<Self, magnus::Error> {
        let mut config = Self::default();

        // Parse entries - accept both single string and array
        if let Some(entries_val) = hash.get(ruby.sym_new("entries")) {
            if let Ok(entries_array) = TryConvert::try_convert(entries_val) {
                config.entries = entries_array;
            } else if let Ok(entry_string) = String::try_convert(entries_val) {
                config.entries = vec![entry_string];
            }
        }

        // Parse optional fields
        if let Some(v) = hash.get(ruby.sym_new("out_dir")) {
            config.output_dir = String::try_convert(v).ok();
        }

        if let Some(format_val) = hash.get(ruby.sym_new("format")) {
            if let Ok(format_str) = String::try_convert(format_val) {
                config.format = OutputFormat::from_str(&format_str);
            } else if let Ok(format_sym) = magnus::Symbol::try_convert(format_val) {
                if let Ok(sym_str) = format_sym.name() {
                    config.format = OutputFormat::from_symbol(&sym_str);
                }
            }
        }

        if let Some(v) = hash.get(ruby.sym_new("sourcemap")) {
            if let Ok(s) = String::try_convert(v) {
                config.sourcemap = Some(s);
            } else if let Ok(b) = bool::try_convert(v) {
                config.sourcemap = Some(if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                });
            }
        }

        if let Some(v) = hash.get(ruby.sym_new("platform")) {
            config.platform = String::try_convert(v).ok();
        }

        if let Some(v) = hash.get(ruby.sym_new("minify")) {
            config.minify = bool::try_convert(v).ok();
        }

        if let Some(v) = hash.get(ruby.sym_new("cwd")) {
            config.cwd = String::try_convert(v).ok();
        }

        // Parse external packages
        if let Some(external_val) = hash.get(ruby.sym_new("external")) {
            if let Ok(external_array) = TryConvert::try_convert(external_val) {
                config.external = Some(external_array);
            } else if let Ok(external_string) = String::try_convert(external_val) {
                config.external = Some(vec![external_string]);
            }
        }

        if let Some(v) = hash.get(ruby.sym_new("external_from_manifest")) {
            config.external_from_manifest = bool::try_convert(v).ok();
        }

        // Parse entry_mode
        if let Some(mode_val) = hash.get(ruby.sym_new("entry_mode")) {
            if let Ok(mode_str) = String::try_convert(mode_val) {
                config.entry_mode = EntryMode::from_str(&mode_str);
            } else if let Ok(mode_sym) = magnus::Symbol::try_convert(mode_val) {
                if let Ok(sym_str) = mode_sym.name() {
                    config.entry_mode = EntryMode::from_symbol(&sym_str);
                }
            }
        }

        // Parse code_splitting
        if let Some(cs_val) = hash.get(ruby.sym_new("code_splitting")) {
            if let Ok(cs_hash) = RHash::try_convert(cs_val) {
                let min_size = cs_hash
                    .get(ruby.sym_new("min_size"))
                    .and_then(|v| u32::try_convert(v).ok())
                    .unwrap_or(20_000);
                let min_imports = cs_hash
                    .get(ruby.sym_new("min_imports"))
                    .and_then(|v| u32::try_convert(v).ok())
                    .unwrap_or(2);
                config.code_splitting = Some(CodeSplittingConfig {
                    min_size,
                    min_imports,
                });
            }
        }

        // Parse MDX options
        if let Some(mdx_val) = hash.get(ruby.sym_new("mdx")) {
            if let Ok(mdx_hash) = RHash::try_convert(mdx_val) {
                let mut mdx_opts = MdxOptions::default();
                if let Some(v) = mdx_hash.get(ruby.sym_new("gfm")) {
                    mdx_opts.gfm = bool::try_convert(v).ok();
                }
                if let Some(v) = mdx_hash.get(ruby.sym_new("footnotes")) {
                    mdx_opts.footnotes = bool::try_convert(v).ok();
                }
                if let Some(v) = mdx_hash.get(ruby.sym_new("math")) {
                    mdx_opts.math = bool::try_convert(v).ok();
                }
                if let Some(v) = mdx_hash.get(ruby.sym_new("jsx_runtime")) {
                    mdx_opts.jsx_runtime = String::try_convert(v).ok();
                }
                if let Some(v) = mdx_hash.get(ruby.sym_new("use_default_plugins")) {
                    mdx_opts.use_default_plugins = bool::try_convert(v).ok();
                }
                config.mdx = Some(mdx_opts);
            }
        }

        Ok(config)
    }
}
