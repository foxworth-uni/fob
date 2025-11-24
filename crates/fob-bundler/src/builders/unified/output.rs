use crate::Result;
use rustc_hash::FxHashMap;
use std::path::Path;

/// Result of a build operation.
///
/// Contains the bundled output, module graph analysis, and build metadata.
pub struct BuildResult {
    /// The output bundle(s) from the build.
    pub output: BuildOutput,

    /// Module graph and dependency analysis.
    pub analysis: fob::analysis::AnalysisResult,

    /// Cache effectiveness metrics.
    pub cache: fob::analysis::CacheAnalysis,

    /// Transformation trace (when JOY_TRACE env var is set).
    pub trace: Option<fob::analysis::TransformationTrace>,

    /// Asset registry containing discovered static assets.
    pub asset_registry: Option<std::sync::Arc<crate::builders::asset_registry::AssetRegistry>>,
}

/// Output from a build operation.
pub enum BuildOutput {
    /// Single bundle output.
    Single(crate::BundleOutput),

    /// Multiple independent bundles (components mode).
    Multiple(FxHashMap<String, crate::BundleOutput>),
}

impl BuildResult {
    /// Access the module graph analysis.
    pub fn analysis(&self) -> &fob::analysis::AnalysisResult {
        &self.analysis
    }

    /// Access build statistics.
    pub fn stats(&self) -> &crate::graph::GraphStatistics {
        &self.analysis.stats
    }

    /// Get entry point module IDs.
    pub fn entry_points(&self) -> &[crate::graph::ModuleId] {
        &self.analysis.entry_points
    }

    /// Access cache effectiveness metrics.
    pub fn cache(&self) -> &fob::analysis::CacheAnalysis {
        &self.cache
    }

    /// Access transformation trace (if enabled via JOY_TRACE).
    pub fn trace(&self) -> Option<&fob::analysis::TransformationTrace> {
        self.trace.as_ref()
    }

    /// Write output files to the specified directory.
    ///
    /// Delegates to `BuildOutput::write_to`.
    pub fn write_to(&self, dir: impl AsRef<Path>, overwrite: bool) -> Result<()> {
        self.output.write_to(dir, overwrite)
    }

    /// Write output files, overwriting any existing files.
    ///
    /// Convenience method that calls `write_to` with `overwrite = true`.
    pub fn write_to_force(&self, dir: impl AsRef<Path>) -> Result<()> {
        self.output.write_to_force(dir)
    }

    /// Iterator over all output chunks (JavaScript/CSS).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_bundler::BuildOptions;
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let result = BuildOptions::library("./src/main.js").build().await?;
    /// for chunk in result.chunks() {
    ///     println!("Chunk: {} ({} bytes)", chunk.filename, chunk.code.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn chunks(&self) -> Box<dyn Iterator<Item = &crate::OutputChunk> + '_> {
        self.output.chunks()
    }

    /// Iterator over all static assets (images, fonts, etc.).
    pub fn assets(&self) -> Box<dyn Iterator<Item = &crate::OutputAsset> + '_> {
        self.output.assets()
    }

    /// Generate a manifest for HTML injection and preloading.
    ///
    /// Maps entry points to their output files and tracks chunk dependencies.
    pub fn manifest(&self) -> crate::output::BundleManifest {
        crate::output::BundleManifest::from_build_output(&self.output, &self.analysis)
    }

    /// Comprehensive build statistics.
    pub fn build_stats(&self) -> crate::output::BuildStats {
        crate::output::BuildStats {
            total_modules: self.analysis.stats.module_count,
            total_chunks: self.chunks().count(),
            total_size: self.total_size(),
            duration_ms: 0, // Could track with std::time::Instant
            cache_hit_rate: self.cache.hit_rate,
        }
    }

    /// Total output size in bytes (all chunks and assets).
    pub fn total_size(&self) -> usize {
        self.output.total_size()
    }

    /// Get all entry point chunks.
    pub fn entry_chunks(&self) -> Box<dyn Iterator<Item = &crate::OutputChunk> + '_> {
        Box::new(self.chunks().filter(|chunk| chunk.is_entry))
    }

    /// Find a specific chunk by file name.
    pub fn find_chunk(&self, filename: &str) -> Option<&crate::OutputChunk> {
        self.chunks().find(|c| c.filename == filename)
    }
}

impl BuildOutput {
    /// Get the single bundle, if this is a single-bundle build.
    pub fn as_single(&self) -> Option<&crate::BundleOutput> {
        match self {
            BuildOutput::Single(bundle) => Some(bundle),
            _ => None,
        }
    }

    /// Get the single bundle mutably.
    pub fn as_single_mut(&mut self) -> Option<&mut crate::BundleOutput> {
        match self {
            BuildOutput::Single(bundle) => Some(bundle),
            _ => None,
        }
    }

    /// Get multiple bundles, if this is a multi-bundle build.
    pub fn as_multiple(&self) -> Option<&FxHashMap<String, crate::BundleOutput>> {
        match self {
            BuildOutput::Multiple(bundles) => Some(bundles),
            _ => None,
        }
    }

    /// Get multiple bundles mutably.
    pub fn as_multiple_mut(&mut self) -> Option<&mut FxHashMap<String, crate::BundleOutput>> {
        match self {
            BuildOutput::Multiple(bundles) => Some(bundles),
            _ => None,
        }
    }

    /// Write output files to the specified directory.
    ///
    /// For single-bundle builds, writes all assets to the directory.
    /// For multi-bundle builds, creates subdirectories for each component.
    ///
    /// # Arguments
    ///
    /// * `dir` - Output directory path
    /// * `overwrite` - Whether to overwrite existing files
    ///
    /// # Errors
    ///
    /// Returns errors for invalid paths or file system errors.
    pub fn write_to(&self, dir: impl AsRef<Path>, overwrite: bool) -> Result<()> {
        use crate::output::writer::write_bundle_to;

        eprintln!("[DEBUG] BuildOutput::write_to called, dir: {}", dir.as_ref().display());
        match self {
            BuildOutput::Single(bundle) => {
                eprintln!("[DEBUG] Single bundle mode, assets: {}", bundle.assets.len());
                write_bundle_to(bundle, dir.as_ref(), overwrite)?;
            }
            BuildOutput::Multiple(bundles) => {
                eprintln!("[DEBUG] Multiple bundles mode, count: {}", bundles.len());
                let dir = dir.as_ref();
                for (name, bundle) in bundles {
                    eprintln!("[DEBUG] Writing bundle '{}' with {} assets", name, bundle.assets.len());
                    let component_dir = dir.join(name);
                    write_bundle_to(bundle, &component_dir, overwrite)?;
                }
            }
        }
        eprintln!("[DEBUG] BuildOutput::write_to completed successfully");
        Ok(())
    }

    /// Write output files, overwriting any existing files.
    ///
    /// Convenience method that calls `write_to` with `overwrite = true`.
    pub fn write_to_force(&self, dir: impl AsRef<Path>) -> Result<()> {
        self.write_to(dir, true)
    }

    /// Iterator over all chunks in the bundle(s).
    pub fn chunks(&self) -> Box<dyn Iterator<Item = &crate::OutputChunk> + '_> {
        match self {
            BuildOutput::Single(bundle) => Box::new(bundle.assets.iter().filter_map(|output| {
                if let crate::Output::Chunk(chunk) = output {
                    Some(&**chunk)
                } else {
                    None
                }
            })),
            BuildOutput::Multiple(bundles) => Box::new(bundles.values().flat_map(|bundle| {
                bundle.assets.iter().filter_map(|output| {
                    if let crate::Output::Chunk(chunk) = output {
                        Some(&**chunk)
                    } else {
                        None
                    }
                })
            })),
        }
    }

    /// Iterator over all assets.
    pub fn assets(&self) -> Box<dyn Iterator<Item = &crate::OutputAsset> + '_> {
        match self {
            BuildOutput::Single(bundle) => Box::new(bundle.assets.iter().filter_map(|output| {
                if let crate::Output::Asset(asset) = output {
                    Some(&**asset)
                } else {
                    None
                }
            })),
            BuildOutput::Multiple(bundles) => Box::new(bundles.values().flat_map(|bundle| {
                bundle.assets.iter().filter_map(|output| {
                    if let crate::Output::Asset(asset) = output {
                        Some(&**asset)
                    } else {
                        None
                    }
                })
            })),
        }
    }

    /// Total size of all outputs.
    pub fn total_size(&self) -> usize {
        match self {
            BuildOutput::Single(bundle) => calculate_bundle_size(bundle),
            BuildOutput::Multiple(bundles) => bundles.values().map(calculate_bundle_size).sum(),
        }
    }
}

/// Helper to calculate bundle size
fn calculate_bundle_size(bundle: &crate::BundleOutput) -> usize {
    bundle
        .assets
        .iter()
        .map(|output| match output {
            crate::Output::Chunk(chunk) => chunk.code.len(),
            crate::Output::Asset(asset) => asset.source.as_bytes().len(),
        })
        .sum()
}
