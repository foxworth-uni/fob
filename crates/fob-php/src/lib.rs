//! PHP bindings for Fob bundler core
//!
//! This module provides ext-php-rs bindings that mirror the Node.js API,
//! allowing PHP users to bundle JavaScript/TypeScript code using Fob.

mod api;
mod conversion;
mod core;
mod error;
mod runtime;
mod tokio_runtime;
mod types;

pub use tokio_runtime::RUNTIME;

use api::config::BundleConfig;
use api::primitives::EntryMode;
use api::utils::parse_format_normalized;
use conversion::result::build_result_to_php_array;
use core::bundler::CoreBundler;
use error::bundler_error_to_php_exception;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};
use types::LogLevel;

// Re-export the Fob class
pub use api::bundler::Fob;

/// Initialize fob logging with specified level.
#[php_function]
pub fn init_logging(level: Option<String>) -> PhpResult<()> {
    let log_level = level
        .as_ref()
        .and_then(|s| LogLevel::from_str(s))
        .unwrap_or_default();
    fob_bundler::init_logging(log_level.to_bundler_level());
    Ok(())
}

/// Initialize logging from RUST_LOG environment variable.
#[php_function]
pub fn init_logging_from_env() -> PhpResult<()> {
    fob_bundler::init_logging_from_env();
    Ok(())
}

/// Get the bundler version.
#[php_function]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Quick helper to bundle a single entry.
#[php_function]
pub fn bundle_single(entry: String, output_dir: String, format: Option<String>) -> PhpResult<Zval> {
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let output_format = format.as_ref().and_then(|f| parse_format_normalized(f));

    let config = BundleConfig {
        entries: vec![entry],
        output_dir: Some(output_dir),
        format: output_format,
        sourcemap: Some("external".to_string()),
        external: None,
        platform: None,
        minify: None,
        cwd,
        mdx: None,
        entry_mode: None,
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

/// Build a standalone bundle (single entry, full bundling).
#[php_function]
pub fn bundle_entry(entry: String, options: Option<&ZendHashTable>) -> PhpResult<Zval> {
    let opts = api::bundler::BuildOptions::from_php_array(options)?;

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
    let opts = api::bundler::BuildOptions::from_php_array(options)?;

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
    let entries_vec = api::utils::array_to_strings(entries);
    let opts = api::bundler::BuildOptions::from_php_array(options)?;

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
    let entries_vec = api::utils::array_to_strings(entries);
    let opts = api::bundler::BuildOptions::from_php_array(options)?;

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

/// PHP module initialization
#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<Fob>()
        .function(wrap_function!(init_logging))
        .function(wrap_function!(init_logging_from_env))
        .function(wrap_function!(version))
        .function(wrap_function!(bundle_single))
        .function(wrap_function!(bundle_entry))
        .function(wrap_function!(library))
        .function(wrap_function!(app))
        .function(wrap_function!(components))
}
