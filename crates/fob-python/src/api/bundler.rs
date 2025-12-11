//! Fob bundler Python class

use crate::api::config::BundleConfig;
use crate::api::primitives::{CodeSplittingConfig, EntryMode};
use crate::api::utils::{
    normalize_string, parse_format_normalized, parse_platform_normalized, py_to_path_string,
    py_to_path_strings,
};
use crate::conversion::result::build_result_to_py_dict;
use crate::core::bundler::CoreBundler;
use crate::error::bundler_error_to_py_err;
use pyo3::Bound;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_async_runtimes::tokio::future_into_py;

/// Common build options shared by preset functions
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub out_dir: Option<String>,
    pub format: Option<String>,
    pub sourcemap: Option<String>,
    pub external: Option<Vec<String>>,
    pub platform: Option<String>,
    pub minify: Option<bool>,
    pub cwd: Option<String>,
}

impl BuildOptions {
    pub fn from_py_dict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut opts = Self::default();

        if let Some(v) = dict.get_item("out_dir")? {
            opts.out_dir = py_to_path_string(&v).ok();
        }

        if let Some(v) = dict.get_item("format")? {
            if let Ok(s) = v.extract::<String>() {
                opts.format = Some(s); // Will be normalized when used
            }
        }

        if let Some(v) = dict.get_item("sourcemap")? {
            if let Ok(s) = v.extract::<String>() {
                opts.sourcemap = Some(normalize_string(&s));
            } else if let Ok(b) = v.extract::<bool>() {
                opts.sourcemap = Some(if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                });
            }
        }

        if let Some(v) = dict.get_item("external")? {
            match py_to_path_strings(&v) {
                Ok(paths) => opts.external = Some(paths),
                Err(_) => {
                    if let Ok(single) = py_to_path_string(&v) {
                        opts.external = Some(vec![single]);
                    }
                }
            }
        }

        if let Some(v) = dict.get_item("platform")? {
            if let Ok(s) = v.extract::<String>() {
                opts.platform = parse_platform_normalized(&s);
            }
        }

        if let Some(v) = dict.get_item("minify")? {
            opts.minify = v.extract::<bool>().ok();
        }

        if let Some(v) = dict.get_item("cwd")? {
            opts.cwd = py_to_path_string(&v).ok();
        }

        Ok(opts)
    }
}

/// Options for app builds with code splitting
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    pub out_dir: Option<String>,
    pub format: Option<String>,
    pub sourcemap: Option<String>,
    pub external: Option<Vec<String>>,
    pub platform: Option<String>,
    pub minify: Option<bool>,
    pub cwd: Option<String>,
    pub code_splitting: Option<CodeSplittingConfig>,
}

impl AppOptions {
    pub fn from_py_dict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut opts = Self::default();

        let code_splitting = if let Some(cs_bound) = dict.get_item("code_splitting")? {
            if let Ok(cs) = cs_bound.cast::<PyDict>() {
                let min_size = cs
                    .get_item("min_size")?
                    .and_then(|v| v.extract::<u32>().ok())
                    .unwrap_or(20_000);
                let min_imports = cs
                    .get_item("min_imports")?
                    .and_then(|v| v.extract::<u32>().ok())
                    .unwrap_or(2);
                Some(CodeSplittingConfig {
                    min_size,
                    min_imports,
                })
            } else {
                None
            }
        } else {
            None
        };
        opts.code_splitting = code_splitting;

        if let Some(v) = dict.get_item("out_dir")? {
            opts.out_dir = py_to_path_string(&v).ok();
        }

        if let Some(v) = dict.get_item("format")? {
            if let Ok(s) = v.extract::<String>() {
                opts.format = Some(s);
            }
        }

        if let Some(v) = dict.get_item("sourcemap")? {
            if let Ok(s) = v.extract::<String>() {
                opts.sourcemap = Some(normalize_string(&s));
            } else if let Ok(b) = v.extract::<bool>() {
                opts.sourcemap = Some(if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                });
            }
        }

        if let Some(v) = dict.get_item("external")? {
            match py_to_path_strings(&v) {
                Ok(paths) => opts.external = Some(paths),
                Err(_) => {
                    if let Ok(single) = py_to_path_string(&v) {
                        opts.external = Some(vec![single]);
                    }
                }
            }
        }

        if let Some(v) = dict.get_item("platform")? {
            if let Ok(s) = v.extract::<String>() {
                opts.platform = parse_platform_normalized(&s);
            }
        }

        if let Some(v) = dict.get_item("minify")? {
            opts.minify = v.extract::<bool>().ok();
        }

        if let Some(v) = dict.get_item("cwd")? {
            opts.cwd = py_to_path_string(&v).ok();
        }

        Ok(opts)
    }
}

/// Fob bundler for Python
#[pyclass]
pub struct Fob {
    bundler: CoreBundler,
}

#[pymethods]
impl Fob {
    /// Create a new bundler instance with full configuration control.
    ///
    /// Args:
    ///     config: Dictionary with bundler configuration. See BundleConfig for available options.
    ///
    /// Returns:
    ///     Fob: A new bundler instance
    ///
    /// Raises:
    ///     FobError: If configuration is invalid
    ///
    /// Example:
    ///     ```python
    ///     import fob
    ///     
    ///     bundler = fob.Fob({
    ///         "entries": ["src/index.ts"],
    ///         "output_dir": "dist",
    ///         "format": "esm",
    ///         "minify": True
    ///     })
    ///     ```
    #[new]
    fn new(config: &Bound<'_, PyDict>) -> PyResult<Self> {
        let bundle_config = BundleConfig::from_py_dict(config)?;
        let bundler = CoreBundler::new(bundle_config)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
        Ok(Self { bundler })
    }

    /// Bundle the configured entries and return detailed bundle information.
    ///
    /// Returns:
    ///     dict: Bundle result containing:
    ///         - chunks: List of chunk information
    ///         - manifest: Bundle manifest with entry mappings
    ///         - stats: Build statistics
    ///         - assets: List of static assets
    ///         - module_count: Total number of modules
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
    ///         bundler = fob.Fob({"entries": ["src/index.ts"]})
    ///         result = await bundler.bundle()
    ///         print(f"Bundled {result['module_count']} modules")
    ///     
    ///     asyncio.run(main())
    ///     ```
    fn bundle<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let bundler = self.bundler.clone();
        future_into_py(py, async move {
            let result = bundler.bundle().await.map_err(bundler_error_to_py_err)?;

            Python::attach(|py| build_result_to_py_dict(py, &result))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Build a standalone bundle (single entry, full bundling).
    ///
    /// Best for: Applications, scripts, or single-file outputs.
    ///
    /// Args:
    ///     entry: Entry file path (str or pathlib.Path)
    ///     options: Optional dictionary with build options:
    ///         - out_dir: Output directory (default: "dist")
    ///         - format: Output format - "esm", "cjs", or "iife" (default: "esm")
    ///         - sourcemap: Source map mode - True, False, "inline", "hidden", or "external"
    ///         - external: List of packages to externalize (or single string)
    ///         - platform: Target platform - "browser" or "node" (default: "browser")
    ///         - minify: Enable minification (default: False)
    ///         - cwd: Working directory for resolution
    ///
    /// Returns:
    ///     dict: Bundle result (same structure as bundle())
    ///
    /// Raises:
    ///     FobError: If bundling fails
    ///
    /// Example:
    ///     ```python
    ///     import asyncio
    ///     import fob
    ///     from pathlib import Path
    ///     
    ///     async def main():
    ///         result = await fob.Fob.bundle_entry(
    ///             Path("src/index.ts"),
    ///             {"out_dir": "dist", "minify": True}
    ///         )
    ///     
    ///     asyncio.run(main())
    ///     ```
    #[staticmethod]
    fn bundle_entry<'py>(
        py: Python<'py>,
        entry: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let entry_str = py_to_path_string(entry)?;

        let opts = if let Some(opts_dict) = options {
            BuildOptions::from_py_dict(opts_dict)?
        } else {
            BuildOptions::default()
        };

        let config = BundleConfig {
            entries: vec![entry_str],
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

        future_into_py(py, async move {
            let bundler = CoreBundler::new(config)
                .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
            let result = bundler.bundle().await.map_err(bundler_error_to_py_err)?;

            Python::attach(|py| build_result_to_py_dict(py, &result))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Build a library (single entry, externalize dependencies).
    ///
    /// Best for: npm packages, reusable libraries.
    /// Dependencies are marked as external and not bundled.
    ///
    /// Args:
    ///     entry: Entry file path (str or pathlib.Path)
    ///     options: Optional dictionary with build options (same as bundle_entry)
    ///
    /// Returns:
    ///     dict: Bundle result
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
    ///         result = await fob.Fob.library(
    ///             "src/index.ts",
    ///             {"external": ["react", "react-dom"]}
    ///         )
    ///     
    ///     asyncio.run(main())
    ///     ```
    #[staticmethod]
    fn library<'py>(
        py: Python<'py>,
        entry: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let entry_str = py_to_path_string(entry)?;

        let opts = if let Some(opts_dict) = options {
            BuildOptions::from_py_dict(opts_dict)?
        } else {
            BuildOptions::default()
        };

        let config = BundleConfig {
            entries: vec![entry_str],
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

        future_into_py(py, async move {
            let bundler = CoreBundler::new(config)
                .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
            let result = bundler.bundle().await.map_err(bundler_error_to_py_err)?;

            Python::attach(|py| build_result_to_py_dict(py, &result))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Build an app with code splitting (multiple entries, unified output)
    ///
    /// Best for: Web applications with multiple pages/routes.
    #[staticmethod]
    fn app<'py>(
        py: Python<'py>,
        entries: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let entries_vec = py_to_path_strings(entries)?;

        let opts = if let Some(opts_dict) = options {
            AppOptions::from_py_dict(opts_dict)?
        } else {
            AppOptions::default()
        };

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

        future_into_py(py, async move {
            let bundler = CoreBundler::new(config)
                .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
            let result = bundler.bundle().await.map_err(bundler_error_to_py_err)?;

            Python::attach(|py| build_result_to_py_dict(py, &result))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Build a component library (multiple entries, separate bundles)
    ///
    /// Best for: UI component libraries, design systems.
    #[staticmethod]
    fn components<'py>(
        py: Python<'py>,
        entries: &Bound<'_, PyAny>,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let entries_vec = py_to_path_strings(entries)?;

        let opts = if let Some(opts_dict) = options {
            BuildOptions::from_py_dict(opts_dict)?
        } else {
            BuildOptions::default()
        };

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

        future_into_py(py, async move {
            let bundler = CoreBundler::new(config)
                .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
            let result = bundler.bundle().await.map_err(bundler_error_to_py_err)?;

            Python::attach(|py| build_result_to_py_dict(py, &result))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}
