//! Fob bundler PHP class
#![allow(dead_code)] // Functions exported via #[php_function] macro

use crate::RUNTIME;
use crate::api::config::BundleConfig;
use crate::api::primitives::{CodeSplittingConfig, EntryMode};
use crate::api::utils::{
    normalize_string, parse_format_normalized, parse_platform_normalized, zval_to_bool,
    zval_to_int, zval_to_string,
};
use crate::conversion::result::build_result_to_php_array;
use crate::core::bundler::CoreBundler;
use crate::error::bundler_error_to_php_exception;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};

/// Common build options shared by all preset functions
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub out_dir: Option<String>,
    pub format: Option<String>,
    pub sourcemap: Option<String>,
    pub external: Option<Vec<String>>,
    pub platform: Option<String>,
    pub minify: Option<bool>,
    pub cwd: Option<String>,
    pub code_splitting: Option<CodeSplittingConfig>,
}

impl BuildOptions {
    pub fn from_php_array(arr: Option<&ZendHashTable>) -> PhpResult<Self> {
        let mut opts = Self::default();

        let arr = match arr {
            Some(a) => a,
            None => return Ok(opts),
        };

        // Parse code_splitting first if present
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
                opts.code_splitting = Some(CodeSplittingConfig {
                    min_size,
                    min_imports,
                });
            }
        }

        if let Some(v) = arr.get("out_dir") {
            opts.out_dir = zval_to_string(v);
        }

        if let Some(v) = arr.get("format") {
            if let Some(s) = zval_to_string(v) {
                opts.format = Some(s);
            }
        }

        if let Some(v) = arr.get("sourcemap") {
            if let Some(s) = zval_to_string(v) {
                opts.sourcemap = Some(normalize_string(&s));
            } else if let Some(b) = zval_to_bool(v) {
                opts.sourcemap = Some(if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                });
            }
        }

        if let Some(v) = arr.get("external") {
            if let Some(external_arr) = v.array() {
                opts.external = Some(crate::api::utils::array_to_strings(external_arr));
            } else if let Some(single) = zval_to_string(v) {
                opts.external = Some(vec![single]);
            }
        }

        if let Some(v) = arr.get("platform") {
            if let Some(s) = zval_to_string(v) {
                opts.platform = parse_platform_normalized(&s);
            }
        }

        if let Some(v) = arr.get("minify") {
            opts.minify = zval_to_bool(v);
        }

        if let Some(v) = arr.get("cwd") {
            opts.cwd = zval_to_string(v);
        }

        Ok(opts)
    }
}

/// Fob bundler for PHP
#[php_class]
pub struct Fob {
    bundler: CoreBundler,
}

#[php_impl]
impl Fob {
    /// Create a new bundler instance with full configuration control.
    pub fn __construct(config: &ZendHashTable) -> PhpResult<Self> {
        let bundle_config = BundleConfig::from_php_array(config)?;
        let bundler =
            CoreBundler::new(bundle_config).map_err(|e| PhpException::default(e.to_string()))?;
        Ok(Self { bundler })
    }

    /// Bundle the configured entries and return detailed bundle information.
    pub fn bundle(&self) -> PhpResult<Zval> {
        let result = RUNTIME
            .block_on(self.bundler.bundle())
            .map_err(bundler_error_to_php_exception)?;

        build_result_to_php_array(&result)
    }
}

// Standalone preset functions
/// Build a standalone bundle (single entry, full bundling).
#[php_function]
pub fn bundle_entry(entry: String, options: Option<&ZendHashTable>) -> PhpResult<Zval> {
    let opts = BuildOptions::from_php_array(options)?;

    let config = BundleConfig {
        entries: vec![entry],
        output_dir: opts.out_dir,
        format: opts
            .format
            .as_ref()
            .and_then(|f| parse_format_normalized(f)),
        sourcemap: opts.sourcemap,
        external: opts.external,
        platform: opts.platform,
        minify: opts.minify,
        cwd: opts.cwd,
        mdx: None,
        entry_mode: Some(EntryMode::Shared),
        code_splitting: None,
        external_from_manifest: None,
        virtual_files: None,
    };

    let bundler = RUNTIME.block_on(async {
        CoreBundler::new(config).map_err(|e| PhpException::default(e.to_string()))
    })?;

    let result = RUNTIME
        .block_on(bundler.bundle())
        .map_err(bundler_error_to_php_exception)?;

    build_result_to_php_array(&result)
}

/// Build a library (single entry, externalize dependencies).
#[php_function]
pub fn library(entry: String, options: Option<&ZendHashTable>) -> PhpResult<Zval> {
    let opts = BuildOptions::from_php_array(options)?;

    let config = BundleConfig {
        entries: vec![entry],
        output_dir: opts.out_dir,
        format: opts
            .format
            .as_ref()
            .and_then(|f| parse_format_normalized(f)),
        sourcemap: opts.sourcemap,
        external: opts.external,
        platform: opts.platform,
        minify: opts.minify,
        cwd: opts.cwd,
        mdx: None,
        entry_mode: Some(EntryMode::Shared),
        code_splitting: None,
        external_from_manifest: Some(true),
        virtual_files: None,
    };

    let bundler = RUNTIME.block_on(async {
        CoreBundler::new(config).map_err(|e| PhpException::default(e.to_string()))
    })?;

    let result = RUNTIME
        .block_on(bundler.bundle())
        .map_err(bundler_error_to_php_exception)?;

    build_result_to_php_array(&result)
}

/// Build an app with code splitting (multiple entries, unified output).
#[php_function]
pub fn app(entries: &ZendHashTable, options: Option<&ZendHashTable>) -> PhpResult<Zval> {
    let entries_vec = crate::api::utils::array_to_strings(entries);
    let opts = BuildOptions::from_php_array(options)?;

    let config = BundleConfig {
        entries: entries_vec,
        output_dir: opts.out_dir,
        format: opts
            .format
            .as_ref()
            .and_then(|f| parse_format_normalized(f)),
        sourcemap: opts.sourcemap,
        external: opts.external,
        platform: opts.platform,
        minify: opts.minify,
        cwd: opts.cwd,
        mdx: None,
        entry_mode: Some(EntryMode::Shared),
        code_splitting: opts.code_splitting,
        external_from_manifest: None,
        virtual_files: None,
    };

    let bundler = RUNTIME.block_on(async {
        CoreBundler::new(config).map_err(|e| PhpException::default(e.to_string()))
    })?;

    let result = RUNTIME
        .block_on(bundler.bundle())
        .map_err(bundler_error_to_php_exception)?;

    build_result_to_php_array(&result)
}

/// Build a component library (multiple entries, separate bundles).
#[php_function]
pub fn components(entries: &ZendHashTable, options: Option<&ZendHashTable>) -> PhpResult<Zval> {
    let entries_vec = crate::api::utils::array_to_strings(entries);
    let opts = BuildOptions::from_php_array(options)?;

    let config = BundleConfig {
        entries: entries_vec,
        output_dir: opts.out_dir,
        format: opts
            .format
            .as_ref()
            .and_then(|f| parse_format_normalized(f)),
        sourcemap: opts.sourcemap,
        external: opts.external,
        platform: opts.platform,
        minify: opts.minify,
        cwd: opts.cwd,
        mdx: None,
        entry_mode: Some(EntryMode::Isolated),
        code_splitting: None,
        external_from_manifest: Some(true),
        virtual_files: None,
    };

    let bundler = RUNTIME.block_on(async {
        CoreBundler::new(config).map_err(|e| PhpException::default(e.to_string()))
    })?;

    let result = RUNTIME
        .block_on(bundler.bundle())
        .map_err(bundler_error_to_php_exception)?;

    build_result_to_php_array(&result)
}
