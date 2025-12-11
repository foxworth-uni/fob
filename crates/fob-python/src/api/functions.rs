//! Standalone Python functions

use crate::api::config::BundleConfig;
use crate::api::utils::{parse_format_normalized, py_to_path_string};
use crate::conversion::result::build_result_to_py_dict;
use crate::core::bundler::CoreBundler;
use crate::error::bundler_error_to_py_err;
use crate::types::LogLevel;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

/// Initialize fob logging with specified level.
///
/// Call this once at application startup before any fob operations.
///
/// Args:
///     level: Log level - "silent", "error", "warn", "info", or "debug" (default: "info")
///
/// Example:
///     ```python
///     import fob
///     fob.init_logging("debug")
///     ```
#[pyfunction]
pub fn init_logging(level: Option<String>) -> PyResult<()> {
    let log_level = level
        .as_ref()
        .and_then(|s| LogLevel::from_str(s))
        .unwrap_or_default();
    fob_bundler::init_logging(log_level.to_bundler_level());
    Ok(())
}

/// Initialize logging from RUST_LOG environment variable.
///
/// Falls back to Info level if RUST_LOG is not set or invalid.
///
/// Example:
///     ```python
///     import os
///     import fob
///     
///     os.environ["RUST_LOG"] = "fob=debug"
///     fob.init_logging_from_env()
///     ```
#[pyfunction]
pub fn init_logging_from_env() -> PyResult<()> {
    fob_bundler::init_logging_from_env();
    Ok(())
}

/// Quick helper to bundle a single entry.
///
/// Convenience function for simple bundling scenarios.
///
/// Args:
///     entry: Entry file path (str or pathlib.Path)
///     output_dir: Output directory path (str or pathlib.Path)
///     format: Output format - "esm", "cjs", or "iife" (default: "esm")
///
/// Returns:
///     dict: Bundle result (same structure as Fob.bundle())
///
/// Raises:
///     FobError: If bundling fails
///
/// Example:
///     ```python
///     import asyncio
///     import fob
///     
///     async def main():
///         result = await fob.bundle_single("src/index.ts", "dist", "esm")
///     
///     asyncio.run(main())
///     ```
#[pyfunction]
pub fn bundle_single<'py>(
    py: Python<'py>,
    entry: &Bound<'_, PyAny>,
    output_dir: &Bound<'_, PyAny>,
    format: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let entry_str = py_to_path_string(entry)?;
    let output_dir_str = py_to_path_string(output_dir)?;

    // Use current working directory - Python script should chdir if needed
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let output_format = format.as_ref().and_then(|f| parse_format_normalized(f));

    let config = BundleConfig {
        entries: vec![entry_str],
        output_dir: Some(output_dir_str),
        format: output_format,
        sourcemap: Some("external".to_string()),
        external: None,
        platform: None,
        minify: None,
        cwd,
        mdx: None, // Auto-detect from entry extension
        entry_mode: None,
        code_splitting: None,
        external_from_manifest: None,
    };

    future_into_py(py, async move {
        let bundler =
            CoreBundler::new(config).map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
        let result = bundler.bundle().await.map_err(bundler_error_to_py_err)?;

        Python::attach(|py| build_result_to_py_dict(py, &result))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    })
}

/// Get the bundler version.
///
/// Returns:
///     str: Version string (e.g., "0.3.0")
///
/// Example:
///     ```python
///     import fob
///     print(f"Fob version: {fob.version()}")
///     ```
#[pyfunction]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
