#![cfg_attr(docsrs, feature(doc_cfg))]

//! # fob-bundler
//!
//! Fob bundler - Rolldown-based bundling on top of fob foundation.
//!
//! This crate provides full bundling capabilities using Rolldown, building on top
//! of the `fob` foundation crate for graph analysis and runtime abstraction.
//!
//! ## Quick Start
//!
//! ### Bundle a library
//!
//! ```no_run
//! use fob_bundler as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = fob::BuildOptions::new("./src/index.js")
//!     .bundle(false)  // Library mode: externalize all dependencies
//!     .platform(fob::Platform::Node)  // Uses Node.js export conditions
//!     .external(["react", "react-dom"])
//!     .sourcemap(true)
//!     .build()
//!     .await?;
//!
//! let bundle = result.output.as_single().expect("single bundle");
//! for asset in bundle.assets.iter() {
//!     std::fs::write(format!("dist/{}", asset.filename()), asset.content_as_bytes())?;
//! }
//! # Ok(()) }
//! ```
//!
//! ### Using BuildConfig (Recommended for New Code)
//!
//! `BuildConfig` provides a cleaner API with better type safety:
//!
//! ```no_run
//! use fob_bundler::BuildConfig;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = BuildConfig::new("./src/index.js")
//!     .bundle(false)
//!     .output_dir("dist")
//!     .build()
//!     .await?;
//!
//! result.write_to("dist", true)?;
//! # Ok(()) }
//! ```
//!
//! ### Analyze without bundling
//!
//! ```no_run
//! use fob_bundler as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let analysis = fob_graph::analyze(["./src/index.js"]).await?;
//! for unused in analysis.graph.unused_exports()? {
//!     println!("unused: {} from {}", unused.export.name, unused.module_id);
//! }
//! # Ok(()) }
//! ```

// Re-export everything from foundation crate
pub use fob_graph::*;

// Bundler-specific modules
pub mod builders;
pub mod config;
pub mod output;
pub mod plugins;
pub mod runtime;
pub mod target;

// Bundler-specific graph modules
pub mod from_rolldown;
pub mod module_collection_plugin;

// Bundler-specific analysis
pub mod analysis;

// Diagnostic extraction
pub mod diagnostics;

// Re-export core Rolldown types for library users
pub use rolldown::{
    BundleOutput, Bundler, BundlerBuilder, BundlerOptions, GlobalsOutputOption, InputItem,
    IsExternal, OutputFormat, Platform, RawMinifyOptions, ResolveOptions, SourceMapType,
};

// Re-export common types (CRITICAL: ModuleType for plugins)
pub use rolldown_common::{
    BundlerTransformOptions, DecoratorOptions, EmittedAsset, IsolatedDeclarationsOptions,
    LogWithoutPlugin, ModuleType, Output, OutputAsset, OutputChunk, TypeScriptOptions,
};

// Re-export plugin types (CRITICAL for plugin authors)
pub use rolldown_plugin::{
    __inner::SharedPluginable, HookGenerateBundleArgs, HookLoadArgs, HookLoadOutput,
    HookLoadReturn, HookNoopReturn, HookResolveIdArgs, HookResolveIdOutput, HookResolveIdReturn,
    HookTransformArgs, HookTransformOutput, HookTransformReturn, HookUsage, Plugin, PluginContext,
    SharedTransformPluginContext,
};

// Re-export bundler APIs
pub use builders::{
    BuildOptions, BuildOutput, BuildResult, EntryPoints, MinifyLevel, build, plugin,
};
pub use config::{
    BuildConfig, ExternalPattern, OptimizationConfig, OutputConfig, ResolutionConfig,
};
pub use plugins::{FobPlugin, PluginPhase, PluginRegistry};

#[cfg(feature = "dts-generation")]
pub use builders::DtsOptions;

// Re-export DtsEmitPlugin from plugins module when dts-generation feature is enabled
#[cfg(feature = "dts-generation")]
#[cfg_attr(docsrs, doc(cfg(feature = "dts-generation")))]
pub use plugins::DtsEmitPlugin;

// Logging utilities (optional, enabled with "logging" feature)
#[cfg(feature = "logging")]
#[cfg_attr(docsrs, doc(cfg(feature = "logging")))]
pub mod logging;

#[cfg(feature = "logging")]
#[cfg_attr(docsrs, doc(cfg(feature = "logging")))]
pub use logging::{LogLevel, init_logging, init_logging_from_env};

pub use output::{AppBuild, Bundle as JoyBundle, ComponentBuild, ImportMap};
pub use target::{DeploymentTarget, ExportConditions, NodeBuiltins, RuntimeEnvironment};

// Re-export AnalyzedBundle (bundler-specific analysis result)
pub use analysis::AnalyzedBundle;

// Test utilities (available in test builds for both unit and integration tests)
// Re-export from fob foundation crate
#[cfg(all(any(test, doctest), not(target_family = "wasm")))]
pub use fob_graph::test_utils;

/// Error types for fob-bundler operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from Rolldown bundler.
    #[error("Rolldown bundler error: {}", format_bundler_error(.0))]
    Bundler(Vec<diagnostics::ExtractedDiagnostic>),

    /// Invalid configuration provided.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid output path (e.g., directory traversal attempt).
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(String),

    /// File write operation failed.
    #[error("Write failure: {0}")]
    WriteFailure(String),

    /// Output file already exists and overwrite is disabled.
    #[error("Output exists: {0}")]
    OutputExists(String),

    /// Asset not found during resolution.
    #[error("Asset not found: {specifier} (searched from: {searched_from})")]
    AssetNotFound {
        specifier: String,
        searched_from: String,
    },

    /// Asset security violation (e.g., directory traversal attempt).
    #[error("Asset security violation: {path} - {reason}")]
    AssetSecurityViolation { path: String, reason: String },

    /// Asset file is too large.
    #[error("Asset too large: {path} ({size} bytes exceeds limit of {max_size} bytes)")]
    AssetTooLarge {
        path: String,
        size: u64,
        max_size: u64,
    },

    /// I/O error with context message.
    #[error("{message}")]
    IoError {
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// Error from foundation crate.
    #[error("Foundation error: {0}")]
    Foundation(#[from] fob_graph::Error),
}

/// Result type alias for fob-bundler operations.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a bundler error from a Rolldown error.
    ///
    /// Extracts structured diagnostics from Rolldown's error types.
    pub fn from_rolldown_batch(error: &dyn std::fmt::Debug) -> Self {
        Error::Bundler(diagnostics::extract_from_rolldown_error(error))
    }
}

/// Format bundler error diagnostics for display.
fn format_bundler_error(diagnostics: &[diagnostics::ExtractedDiagnostic]) -> String {
    if diagnostics.is_empty() {
        return "Unknown bundler error".to_string();
    }

    if diagnostics.len() == 1 {
        let diag = &diagnostics[0];
        format!("{}: {}", diag.kind, diag.message)
    } else {
        format!(
            "{} errors: {}",
            diagnostics.len(),
            diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.kind, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}

impl std::fmt::Display for diagnostics::DiagnosticKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            diagnostics::DiagnosticKind::MissingExport => write!(f, "MissingExport"),
            diagnostics::DiagnosticKind::ParseError => write!(f, "ParseError"),
            diagnostics::DiagnosticKind::CircularDependency => write!(f, "CircularDependency"),
            diagnostics::DiagnosticKind::UnresolvedEntry => write!(f, "UnresolvedEntry"),
            diagnostics::DiagnosticKind::UnresolvedImport => write!(f, "UnresolvedImport"),
            diagnostics::DiagnosticKind::InvalidOption => write!(f, "InvalidOption"),
            diagnostics::DiagnosticKind::Plugin => write!(f, "Plugin"),
            diagnostics::DiagnosticKind::Transform => write!(f, "Transform"),
            diagnostics::DiagnosticKind::Other(s) => write!(f, "{}", s),
        }
    }
}

impl miette::Diagnostic for Error {
    fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(match self {
            Error::Bundler(_) => "BUNDLER_ERROR",
            Error::InvalidConfig(_) => "INVALID_CONFIG",
            Error::Io(_) => "IO_ERROR",
            Error::InvalidOutputPath(_) => "INVALID_OUTPUT_PATH",
            Error::WriteFailure(_) => "WRITE_FAILURE",
            Error::OutputExists(_) => "OUTPUT_EXISTS",
            Error::AssetNotFound { .. } => "ASSET_NOT_FOUND",
            Error::AssetSecurityViolation { .. } => "ASSET_SECURITY_VIOLATION",
            Error::AssetTooLarge { .. } => "ASSET_TOO_LARGE",
            Error::IoError { .. } => "IO_ERROR",
            Error::Foundation(_) => "FOUNDATION_ERROR",
        }))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        match self {
            Error::InvalidConfig(msg) => Some(Box::new(format!(
                "Check your configuration file for syntax errors.\nError: {}",
                msg
            ))),
            Error::InvalidOutputPath(path) => Some(Box::new(format!(
                "The output path '{}' is invalid. Ensure it's within the project directory and doesn't contain '..' components.",
                path
            ))),
            Error::WriteFailure(msg) => Some(Box::new(format!(
                "Failed to write file. Check disk space and permissions.\nError: {}",
                msg
            ))),
            Error::OutputExists(msg) => Some(Box::new(format!(
                "Output file already exists: {}\nUse --overwrite flag to replace existing files.",
                msg
            ))),
            Error::AssetNotFound {
                specifier,
                searched_from,
            } => Some(Box::new(format!(
                "Could not find asset '{}'.\nSearched from: {}\nCheck that the file exists and the path is correct.",
                specifier, searched_from
            ))),
            Error::AssetSecurityViolation { path, reason } => Some(Box::new(format!(
                "Security violation detected for path '{}': {}\nPaths must be within the project directory.",
                path, reason
            ))),
            Error::AssetTooLarge {
                path,
                size,
                max_size,
            } => Some(Box::new(format!(
                "Asset '{}' is too large: {} bytes (max: {} bytes).\nConsider splitting large assets or increasing the size limit.",
                path, size, max_size
            ))),
            Error::Bundler(diagnostics) => {
                if diagnostics.len() == 1 {
                    diagnostics[0]
                        .help
                        .as_ref()
                        .map(|h| Box::new(h.clone()) as Box<dyn std::fmt::Display>)
                } else {
                    Some(Box::new(
                        "Multiple bundler errors occurred. See details below.".to_string(),
                    ))
                }
            }
            _ => None,
        }
    }

    fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn miette::Diagnostic> + '_>> {
        match self {
            Error::Bundler(diagnostics) if diagnostics.len() > 1 => {
                // Return all diagnostics as related errors
                // This requires converting each diagnostic, which is complex
                // For now, return None - the main error message will show the count
                None
            }
            _ => None,
        }
    }
}
