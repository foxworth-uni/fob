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
//! let result = fob::BuildOptions::library("./src/index.js")
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
//! ### Analyze without bundling
//!
//! ```no_run
//! use fob_bundler as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let analysis = fob::analyze(["./src/index.js"]).await?;
//! for unused in analysis.graph.unused_exports()? {
//!     println!("unused: {} from {}", unused.export.name, unused.module_id);
//! }
//! # Ok(()) }
//! ```

// Re-export everything from foundation crate
pub use fob::*;

// Bundler-specific modules
pub mod builders;
pub mod output;
pub mod plugins;

// Bundler-specific graph modules
pub mod from_rolldown;
pub mod module_collection_plugin;

// Bundler-specific analysis
pub mod analysis;

// Diagnostic extraction
pub mod diagnostics;

// Re-export core Rolldown types for library users
pub use rolldown::{
    BundleOutput, Bundler, BundlerOptions, InputItem, OutputFormat, Platform, SourceMapType,
};

// Re-export output types for detailed bundle inspection
pub use rolldown_common::{Output, OutputAsset, OutputChunk};

// Re-export TypeScript-related types from rolldown_common
pub use rolldown_common::{
    BundlerTransformOptions, IsolatedDeclarationsOptions, TypeScriptOptions,
};

// Re-export bundler APIs
pub use builders::{build, plugin, BuildOptions, BuildOutput, BuildResult, EntryPoints};

#[cfg(feature = "dts-generation")]
pub use builders::DtsOptions;

// Re-export DtsEmitPlugin from plugins module when dts-generation feature is enabled
#[cfg(feature = "dts-generation")]
#[cfg_attr(docsrs, doc(cfg(feature = "dts-generation")))]
pub use plugins::DtsEmitPlugin;

pub use output::{AppBuild, Bundle as JoyBundle, ComponentBuild, ImportMap};

pub use rolldown_plugin::{Plugin, __inner::SharedPluginable};

// Re-export AnalyzedBundle (bundler-specific analysis result)
pub use analysis::AnalyzedBundle;

// Test utilities (available in test builds for both unit and integration tests)
// Re-export from fob foundation crate
#[cfg(all(any(test, doctest), not(target_family = "wasm")))]
pub use fob::test_utils;

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
    Foundation(#[from] fob::Error),
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
