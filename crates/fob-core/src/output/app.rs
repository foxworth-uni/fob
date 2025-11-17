use std::path::Path;

use rustc_hash::FxHashMap;

use crate::graph::ModuleGraph;
use crate::Result;

use super::bundle::Bundle;
use super::metadata::BundleMetadata;

/// Result of application bundling with optional chunk metadata.
pub struct AppBuild {
    bundle: Bundle,
    chunks: FxHashMap<String, Vec<String>>,
    graph: ModuleGraph,
}

impl AppBuild {
    pub(crate) fn new(bundle: Bundle) -> Self {
        let graph = bundle.module_graph().clone();
        Self {
            bundle,
            chunks: FxHashMap::default(),
            graph,
        }
    }

    pub fn bundle(&self) -> &Bundle {
        &self.bundle
    }

    pub fn chunk_manifest(&self) -> &FxHashMap<String, Vec<String>> {
        &self.chunks
    }

    pub fn module_graph(&self) -> &ModuleGraph {
        &self.graph
    }

    /// Writes the application bundle to disk, erroring if files already exist.
    ///
    /// This is the safe default that prevents accidentally overwriting existing files.
    /// Use [`write_to_force`](Self::write_to_force) if you want to overwrite.
    ///
    /// # Atomic Guarantees
    ///
    /// Either all files are written successfully or none are written. If any operation
    /// fails, all previously written files are rolled back.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_core as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::app(["./src/main.js", "./src/dashboard.js"])
    ///     .build()
    ///     .await?;
    ///
    /// let app = result.output.as_single().expect("single bundle");
    /// // Write to dist/, error if files exist
    /// app.write_to("dist")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to(&self, dir: impl AsRef<Path>) -> Result<()> {
        self.bundle.write_to(dir)
    }

    /// Writes the application bundle to disk, overwriting existing files.
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
    /// # use fob_core as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::app(["./src/main.js", "./src/dashboard.js"])
    ///     .build()
    ///     .await?;
    ///
    /// let app = result.output.as_single().expect("single bundle");
    /// // Overwrite existing files in dist/
    /// app.write_to_force("dist")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_force(&self, dir: impl AsRef<Path>) -> Result<()> {
        self.bundle.write_to_force(dir)
    }

    /// Extracts comprehensive metadata about the application bundle.
    ///
    /// This analyzes ALL modules in the bundle to build a complete picture
    /// of exports, imports, and sizes.
    ///
    /// # Performance
    ///
    /// This operation walks the entire module graph. If you need to call it multiple
    /// times, consider caching the result.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_core as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::app(["./src/main.js", "./src/dashboard.js"])
    ///     .build()
    ///     .await?;
    ///
    /// let app = result.output.as_single().expect("single bundle");
    /// let metadata = app.metadata();
    /// println!("Total size: {} KB", metadata.total_size() / 1024);
    /// println!("Modules: {}", metadata.module_count());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn metadata(&self) -> Result<BundleMetadata> {
        self.bundle.metadata().await
    }
}
