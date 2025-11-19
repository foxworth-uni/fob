use std::path::Path;

use rolldown_common::Output;

use fob::analysis::{AnalysisResult, CacheAnalysis, TransformationTrace};
use fob::graph::{GraphStatistics, ModuleGraph, ModuleId};
use crate::analysis::AnalyzedBundle;
use crate::{BundleOutput, Result};

use super::metadata::BundleMetadata;
use super::writer::write_bundle_to;

/// High-level wrapper around `AnalyzedBundle` with ergonomic accessors.
pub struct Bundle {
    inner: AnalyzedBundle,
}

impl Bundle {
    /// Raw Rolldown bundle output (assets and warnings).
    pub fn output(&self) -> &BundleOutput {
        &self.inner.bundle
    }

    /// Pre-bundling analysis results.
    pub fn analysis(&self) -> &AnalysisResult {
        &self.inner.analysis
    }

    /// Dependency graph captured during analysis.
    pub fn module_graph(&self) -> &ModuleGraph {
        &self.inner.analysis.graph
    }

    /// Entry points analysed for this bundle.
    pub fn entry_points(&self) -> &[ModuleId] {
        &self.inner.analysis.entry_points
    }

    /// Aggregate statistics for the analysed module graph.
    pub fn stats(&self) -> &GraphStatistics {
        &self.inner.analysis.stats
    }

    /// Cache metrics gathered during bundling.
    pub fn cache(&self) -> &CacheAnalysis {
        &self.inner.cache
    }

    /// Transformation trace if `JOY_TRACE=1`.
    pub fn trace(&self) -> Option<&TransformationTrace> {
        self.inner.trace.as_ref()
    }

    /// Warnings discovered during static analysis.
    pub fn analysis_warnings(&self) -> &[String] {
        &self.inner.analysis.warnings
    }

    /// Errors discovered during static analysis.
    pub fn analysis_errors(&self) -> &[String] {
        &self.inner.analysis.errors
    }

    /// Consume the bundle and return the underlying analysed payload.
    pub fn into_inner(self) -> AnalyzedBundle {
        self.inner
    }

    /// Writes the bundle to disk, erroring if files already exist.
    ///
    /// This is the safe default that prevents accidentally overwriting existing files.
    /// Use [`write_to_force`](Self::write_to_force) if you want to overwrite.
    ///
    /// # Atomic Guarantees
    ///
    /// Either all files are written successfully or none are written. If any operation
    /// fails, all previously written files are rolled back.
    ///
    /// # Security
    ///
    /// - Validates all paths to prevent directory traversal attacks
    /// - Creates parent directories automatically
    /// - Uses atomic writes (temp file + rename)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_bundler as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::library("./src/index.js")
    ///     .build()
    ///     .await?;
    ///
    /// let bundle = result.output.as_single().expect("single bundle");
    /// // Write to dist/ directory, error if files exist
    /// bundle.write_to("dist")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to(&self, dir: impl AsRef<Path>) -> Result<()> {
        write_bundle_to(self.output(), dir.as_ref(), false)
    }

    /// Writes the bundle to disk, overwriting existing files.
    ///
    /// Use this when you want to force overwrite existing files. For the safe default
    /// that errors on conflicts, use [`write_to`](Self::write_to).
    ///
    /// # Atomic Guarantees
    ///
    /// Either all files are written successfully or none are written. If any operation
    /// fails, all previously written files are rolled back.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_bundler as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::library("./src/index.js")
    ///     .build()
    ///     .await?;
    ///
    /// let bundle = result.output.as_single().expect("single bundle");
    /// // Overwrite existing files in dist/
    /// bundle.write_to_force("dist")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_force(&self, dir: impl AsRef<Path>) -> Result<()> {
        write_bundle_to(self.output(), dir.as_ref(), true)
    }

    /// Extracts comprehensive metadata about the bundle's contents.
    ///
    /// This analyzes ALL modules in the bundle (not just entry points) to build
    /// a complete picture of exports, imports, and sizes.
    ///
    /// # Performance
    ///
    /// This operation walks the entire module graph. If you need to call it multiple
    /// times, consider caching the result.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_bundler as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::library("./src/index.js")
    ///     .build()
    ///     .await?;
    ///
    /// let bundle = result.output.as_single().expect("single bundle");
    /// let metadata = bundle.metadata();
    /// println!("Exports: {}", metadata.exports().len());
    /// println!("Total size: {} bytes", metadata.total_size());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn metadata(&self) -> Result<BundleMetadata> {
        let mut total_size = 0;
        for output in &self.output().assets {
            match output {
                Output::Asset(asset) => total_size += asset.source.as_bytes().len(),
                Output::Chunk(chunk) => total_size += chunk.code.len(),
            }
        }
        let asset_count = self.output().assets.len();

        BundleMetadata::from_graph(self.module_graph(), total_size, asset_count).await
    }

    /// Checks if the bundle has a default export.
    ///
    /// This is useful for determining import patterns:
    /// - Default export: `import Bundle from './bundle.js'`
    /// - Named exports only: `import { Component } from './bundle.js'`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_bundler as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::library("./src/index.js")
    ///     .build()
    ///     .await?;
    ///
    /// let bundle = result.output.as_single().expect("single bundle");
    /// if bundle.has_default_export() {
    ///     println!("Use: import Bundle from './bundle.js'");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_default_export(&self) -> Result<bool> {
        Ok(self.metadata().await?.has_default_export())
    }

    /// Returns the total size of all assets in bytes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_bundler as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::library("./src/index.js")
    ///     .build()
    ///     .await?;
    ///
    /// let bundle = result.output.as_single().expect("single bundle");
    /// println!("Bundle size: {} KB", bundle.total_size() / 1024);
    /// # Ok(())
    /// # }
    /// ```
    pub fn total_size(&self) -> usize {
        let mut total = 0;
        for output in &self.output().assets {
            match output {
                Output::Asset(asset) => total += asset.source.as_bytes().len(),
                Output::Chunk(chunk) => total += chunk.code.len(),
            }
        }
        total
    }
}
