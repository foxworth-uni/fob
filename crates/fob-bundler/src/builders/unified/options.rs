use crate::{Error, OutputFormat, Platform, Result, Runtime, SharedPluginable};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature = "dts-generation")]
use super::dts::DtsOptions;
use super::entry::EntryPoints;
use super::primitives::{CodeSplittingConfig, EntryMode, ExternalConfig, IncrementalConfig};

/// Configuration options for a build operation.
///
/// This struct controls all aspects of the bundling process using three
/// orthogonal primitives:
///
/// 1. **EntryMode**: Shared (entries can share chunks) vs Isolated (independent bundles)
/// 2. **CodeSplittingConfig**: Code splitting configuration (Option = on/off)
/// 3. **ExternalConfig**: External dependencies (None, List, FromManifest)
///
/// Use the builder pattern methods for ergonomic configuration, or construct
/// directly for full control.
///
/// # Examples
///
/// ```no_run
/// use fob_bundler::BuildOptions;
///
/// # async fn example() -> fob_bundler::Result<()> {
/// // Single entry app
/// let app = BuildOptions::new("src/index.ts")
///     .outfile("dist/index.js")
///     .build()
///     .await?;
///
/// // Multi-page app with shared chunks
/// let app = BuildOptions::new_multiple(["page1.js", "page2.js"])
///     .bundle_together()
///     .with_code_splitting()
///     .build()
///     .await?;
///
/// // Component library (each component isolated)
/// let components = BuildOptions::new_multiple(["Button.tsx", "Card.tsx"])
///     .bundle_separately()
///     .externalize(["react", "react-dom"])
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Entry point(s) for the build.
    pub entry: EntryPoints,

    /// Do entries share code?
    ///
    /// - `Shared`: Entries can share chunks (was: Unified)
    /// - `Isolated`: Each entry stands alone (was: Separate)
    pub entry_mode: EntryMode,

    /// Code splitting configuration.
    ///
    /// - `None`: No code splitting (default)
    /// - `Some(config)`: Enable code splitting with specified thresholds
    ///
    /// Only applies when `entry_mode` is `Shared`.
    pub code_splitting: Option<CodeSplittingConfig>,

    /// External dependency configuration.
    ///
    /// - `None`: Bundle everything (was: BundlingMode::Full)
    /// - `List(packages)`: Externalize specific packages
    /// - `FromManifest(path)`: Externalize dependencies from package.json
    pub external: ExternalConfig,

    /// Output directory for bundled files.
    ///
    /// Cannot be used with `outfile`. Required when code splitting is enabled.
    pub outdir: Option<PathBuf>,

    /// Output file path for single-entry builds.
    ///
    /// Cannot be used with `outdir` or when code splitting is enabled.
    /// Only valid for `EntryPoints::Single`.
    pub outfile: Option<PathBuf>,

    /// Target runtime platform (default: Browser).
    pub platform: Platform,

    /// Output module format (default: ESM).
    pub format: OutputFormat,

    /// Source map generation strategy (default: external file).
    pub sourcemap: Option<crate::SourceMapType>,

    /// Minification level as a string (default: None/disabled).
    ///
    /// Valid values: "none", "whitespace", "syntax", "identifiers"
    /// Also accepts "true"/"false" for boolean compatibility.
    pub minify_level: Option<String>,

    /// Global variable names for external packages (IIFE/UMD only).
    ///
    /// Maps package names to global variable names.
    /// Example: `{"react": "React", "react-dom": "ReactDOM"}`
    pub globals: FxHashMap<String, String>,

    /// Rolldown/Rollup plugins to apply during bundling.
    pub plugins: Vec<SharedPluginable>,

    /// Virtual files that don't exist on disk.
    ///
    /// Maps virtual paths to their content. Useful for programmatic entry points.
    pub virtual_files: FxHashMap<String, String>,

    /// Path aliases for import resolution (e.g., "@" → "src").
    ///
    /// Maps alias prefixes to their target directories. These are resolved
    /// relative to the `cwd` if not absolute.
    pub path_aliases: FxHashMap<String, String>,

    /// Working directory for module resolution (default: current directory).
    pub cwd: Option<PathBuf>,

    /// Runtime for filesystem operations (default: NativeRuntime on native).
    ///
    /// This allows the bundler to work across different platforms:
    /// - On native targets, NativeRuntime uses std::fs
    /// - On WASM targets, BrowserRuntime bridges to JavaScript
    pub runtime: Option<Arc<dyn Runtime>>,

    /// Decorator transformation options (modern/Stage 3 decorators).
    ///
    /// Enables transformation of JavaScript/TypeScript decorators.
    /// Only supports modern decorators (TC39 Stage 3 proposal).
    pub decorator: Option<crate::DecoratorOptions>,

    /// TypeScript declaration file generation options.
    #[cfg(feature = "dts-generation")]
    pub dts: Option<DtsOptions>,

    /// Build cache configuration.
    ///
    /// When enabled, build results are cached based on content hashing.
    /// On cache hits, the bundle is loaded from cache instead of running Rolldown.
    pub cache: Option<crate::cache::CacheConfig>,

    /// Incremental module graph caching configuration.
    ///
    /// When enabled, the module graph is cached between builds and only
    /// reprocessed when files change. This is complementary to `cache`:
    /// - `cache`: Caches entire build output (skips Rolldown entirely on hit)
    /// - `incremental`: Caches module graph (Rolldown still runs, graph reused)
    ///
    /// For watch mode, `incremental` is typically more useful since files
    /// change frequently. For CI, `cache` is better for identical inputs.
    pub incremental: Option<IncrementalConfig>,

    /// Maximum parallel builds for `EntryMode::Isolated` (native only).
    ///
    /// Controls concurrency to prevent thread pool contention with Rolldown's
    /// internal parallelism.
    ///
    /// - `None`: Use default (`min(num_cpus, 8)`)
    /// - `Some(1)`: Force sequential execution
    /// - `Some(n)`: Limit to n concurrent builds
    ///
    /// Ignored on WASM (builds are always sequential).
    pub max_parallel_builds: Option<usize>,
}

impl BuildOptions {
    /// Create a new BuildOptions with a single entry point.
    ///
    /// Defaults:
    /// - `entry_mode`: `Shared`
    /// - `code_splitting`: `None`
    /// - `external`: `None` (bundle everything)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let options = BuildOptions::new("./src/index.js")
    ///     .outfile("dist/bundle.js")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(entry: impl AsRef<Path>) -> Self {
        Self {
            entry: EntryPoints::Single(normalize_entry_path(entry)),
            entry_mode: EntryMode::Shared,
            code_splitting: None,
            external: ExternalConfig::None,
            outdir: None,
            outfile: None,
            platform: Platform::Browser,
            format: OutputFormat::Esm,
            sourcemap: Some(crate::SourceMapType::File),
            minify_level: None,
            globals: FxHashMap::default(),
            plugins: Vec::new(),
            virtual_files: FxHashMap::default(),
            path_aliases: FxHashMap::default(),
            cwd: None,
            runtime: None,
            decorator: None,
            #[cfg(feature = "dts-generation")]
            dts: None,
            cache: None,
            incremental: None,
            max_parallel_builds: None,
        }
    }

    /// Create BuildOptions with multiple entry points.
    ///
    /// Defaults:
    /// - `entry_mode`: `Isolated` (independent bundles per entry)
    /// - `code_splitting`: `None`
    /// - `external`: `None` (bundle everything)
    ///
    /// Use `.bundle_together().with_code_splitting()` for code splitting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// // Isolated bundles (default)
    /// let options = BuildOptions::new_multiple(["./src/a.js", "./src/b.js"])
    ///     .bundle_separately()
    ///     .outdir("dist")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_multiple<P, I>(entries: I) -> Self
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let normalized = entries.into_iter().map(normalize_entry_path).collect();

        Self {
            entry: EntryPoints::Multiple(normalized),
            entry_mode: EntryMode::Isolated,
            code_splitting: None,
            external: ExternalConfig::None,
            outdir: None,
            outfile: None,
            platform: Platform::Browser,
            format: OutputFormat::Esm,
            sourcemap: Some(crate::SourceMapType::File),
            minify_level: None,
            globals: FxHashMap::default(),
            plugins: Vec::new(),
            virtual_files: FxHashMap::default(),
            path_aliases: FxHashMap::default(),
            cwd: None,
            runtime: None,
            decorator: None,
            #[cfg(feature = "dts-generation")]
            dts: None,
            cache: None,
            incremental: None,
            max_parallel_builds: None,
        }
    }

    /// Set entries to share code (was: Unified).
    ///
    /// Entries can share chunks and use code splitting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let opts = BuildOptions::new_multiple(["a.js", "b.js"])
    ///     .bundle_together()
    ///     .with_code_splitting();
    /// ```
    pub fn bundle_together(mut self) -> Self {
        self.entry_mode = EntryMode::Shared;
        self
    }

    /// Set entries to be isolated (was: Separate).
    ///
    /// Each entry produces an independent bundle.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let opts = BuildOptions::new_multiple(["Button.tsx", "Card.tsx"])
    ///     .bundle_separately();
    /// ```
    pub fn bundle_separately(mut self) -> Self {
        self.entry_mode = EntryMode::Isolated;
        self.code_splitting = None; // Isolated mode doesn't support code splitting
        self
    }

    /// Enable code splitting with default settings (20KB, min_imports 2).
    ///
    /// Requires multiple entry points and `entry_mode: Shared`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let opts = BuildOptions::new_multiple(["page1.js", "page2.js"])
    ///     .bundle_together()
    ///     .with_code_splitting();
    /// ```
    pub fn with_code_splitting(mut self) -> Self {
        self.code_splitting = Some(CodeSplittingConfig::default());
        self
    }

    /// Configure code splitting with custom thresholds.
    ///
    /// # Arguments
    ///
    /// * `min_size` - Minimum chunk size in bytes
    /// * `min_imports` - Minimum entry points importing the same module (must be >= 2)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let opts = BuildOptions::new_multiple(["page1.js", "page2.js"])
    ///     .bundle_together()
    ///     .with_code_splitting_config(
    ///         fob_bundler::CodeSplittingConfig::new(50_000, 3)
    ///     );
    /// ```
    pub fn with_code_splitting_config(mut self, config: CodeSplittingConfig) -> Self {
        self.code_splitting = Some(config);
        self
    }

    /// Externalize specific packages (explicit list).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let opts = BuildOptions::new("src/index.js")
    ///     .externalize(["react", "react-dom"]);
    /// ```
    pub fn externalize<I, S>(mut self, packages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let packages_vec: Vec<String> = packages.into_iter().map(|s| s.into()).collect();
        self.external = ExternalConfig::List(packages_vec);
        self
    }

    /// Externalize dependencies from package.json manifest.
    ///
    /// Reads `dependencies` and `peerDependencies` from the specified file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let opts = BuildOptions::new("src/lib.ts")
    ///     .externalize_from("package.json");
    /// ```
    pub fn externalize_from(mut self, manifest_path: impl Into<std::path::PathBuf>) -> Self {
        self.external = ExternalConfig::FromManifest(manifest_path.into());
        self
    }

    /// Set the output directory.
    pub fn outdir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.outdir = Some(dir.into());
        self
    }

    /// Set the output file (single entry only).
    pub fn outfile(mut self, file: impl Into<PathBuf>) -> Self {
        self.outfile = Some(file.into());
        self
    }

    /// Set the target platform.
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// Set the output format.
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable or disable source maps.
    pub fn sourcemap(mut self, enabled: bool) -> Self {
        self.sourcemap = if enabled {
            Some(crate::SourceMapType::File)
        } else {
            None
        };
        self
    }

    /// Generate inline source maps.
    pub fn sourcemap_inline(mut self) -> Self {
        self.sourcemap = Some(crate::SourceMapType::Inline);
        self
    }

    /// Generate hidden source maps.
    pub fn sourcemap_hidden(mut self) -> Self {
        self.sourcemap = Some(crate::SourceMapType::Hidden);
        self
    }

    /// Set the minification level.
    ///
    /// # Supported Values
    ///
    /// - `"none"` - No minification
    /// - `"whitespace"` - Remove whitespace only
    /// - `"syntax"` - Syntax-level optimizations
    /// - `"identifiers"` - Full minification with identifier mangling
    pub fn minify_level(mut self, level: impl Into<String>) -> Self {
        self.minify_level = Some(level.into());
        self
    }

    /// Set global variable mappings for external packages.
    pub fn globals_map<I, K, V>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in entries {
            self.globals.insert(k.into(), v.into());
        }
        self
    }

    /// Add a Rolldown plugin.
    pub fn plugin<P>(mut self, plugin: P) -> Self
    where
        P: crate::builders::common::IntoPlugin,
    {
        self.plugins.push(plugin.into_plugin());
        self
    }

    /// Add a virtual file.
    pub fn virtual_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
        self.virtual_files.insert(path.into(), content.into());
        self
    }

    /// Add a path alias for import resolution.
    pub fn path_alias(mut self, alias: impl Into<String>, target: impl Into<String>) -> Self {
        self.path_aliases.insert(alias.into(), target.into());
        self
    }

    /// Add multiple path aliases at once.
    pub fn path_aliases(mut self, aliases: FxHashMap<String, String>) -> Self {
        self.path_aliases = aliases;
        self
    }

    /// Set the working directory for module resolution.
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set the runtime for filesystem operations.
    pub fn runtime(mut self, runtime: Arc<dyn Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Enable modern decorator transformation.
    pub fn decorators(mut self, enabled: bool) -> Self {
        if enabled {
            self.decorator = Some(crate::DecoratorOptions {
                legacy: Some(false),
                emit_decorator_metadata: None,
            });
        } else {
            self.decorator = None;
        }
        self
    }

    /// Enable TypeScript declaration file generation.
    #[cfg(feature = "dts-generation")]
    pub fn emit_dts(mut self, enabled: bool) -> Self {
        let mut dts = self.dts.unwrap_or_default();
        dts.emit = Some(enabled);
        self.dts = Some(dts);
        self
    }

    /// Set the output directory for declaration files.
    #[cfg(feature = "dts-generation")]
    pub fn dts_outdir(mut self, dir: impl Into<PathBuf>) -> Self {
        let mut dts = self.dts.unwrap_or_default();
        dts.outdir = Some(dir.into());
        self.dts = Some(dts);
        self
    }

    /// Strip @internal JSDoc tags from declarations.
    #[cfg(feature = "dts-generation")]
    pub fn strip_internal(mut self, enabled: bool) -> Self {
        let mut dts = self.dts.unwrap_or_default();
        dts.strip_internal = enabled;
        self.dts = Some(dts);
        self
    }

    /// Generate source maps for declaration files.
    #[cfg(feature = "dts-generation")]
    pub fn declaration_map(mut self, enabled: bool) -> Self {
        let mut dts = self.dts.unwrap_or_default();
        dts.sourcemap = enabled;
        self.dts = Some(dts);
        self
    }

    /// Enable build caching with the specified directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let result = BuildOptions::new("src/index.ts")
    ///     .cache_dir(".cache/fob")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn cache_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.cache = Some(crate::cache::CacheConfig::new(dir));
        self
    }

    /// Set the cache configuration directly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::{BuildOptions, CacheConfig};
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let cache = CacheConfig::new(".cache/fob")
    ///     .with_env_vars(["NODE_ENV"]);
    ///
    /// let result = BuildOptions::new("src/index.ts")
    ///     .cache(cache)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn cache(mut self, config: crate::cache::CacheConfig) -> Self {
        self.cache = Some(config);
        self
    }

    /// Force rebuild even if cached result exists.
    ///
    /// The cache will still be updated after the build completes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let result = BuildOptions::new("src/index.ts")
    ///     .cache_dir(".cache/fob")
    ///     .force_rebuild()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn force_rebuild(mut self) -> Self {
        if let Some(cache) = &mut self.cache {
            cache.force_rebuild = true;
        }
        self
    }

    /// Enable incremental module graph caching with default directory.
    ///
    /// Uses `.fob-cache` as the cache directory by default.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let result = BuildOptions::new("src/index.ts")
    ///     .incremental()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn incremental(mut self) -> Self {
        self.incremental = Some(IncrementalConfig::default());
        self
    }

    /// Enable incremental caching with custom directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let result = BuildOptions::new("src/index.ts")
    ///     .incremental_dir(".cache/incremental")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn incremental_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.incremental = Some(IncrementalConfig::new(dir.as_ref()));
        self
    }

    /// Set maximum parallel builds for `EntryMode::Isolated`.
    ///
    /// Controls concurrency to prevent thread pool contention with Rolldown's
    /// internal parallelism.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum concurrent builds. `1` forces sequential execution.
    ///
    /// # Notes
    ///
    /// - Default is `min(num_cpus, 8)` on native platforms
    /// - Ignored on WASM (builds are always sequential)
    /// - Only applies when `entry_mode` is `Isolated`
    pub fn max_parallel_builds(mut self, max: usize) -> Self {
        self.max_parallel_builds = Some(max);
        self
    }

    /// Validate the build options for internal consistency.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<()> {
        // Entry count validations
        const MAX_ENTRY_POINTS: usize = 1000;
        let entry_count = match &self.entry {
            EntryPoints::Single(_) => 1,
            EntryPoints::Multiple(v) => v.len(),
            EntryPoints::Named(m) => m.len(),
        };

        if entry_count > MAX_ENTRY_POINTS {
            return Err(Error::InvalidConfig(format!(
                "Too many entry points: {} (max {})",
                entry_count, MAX_ENTRY_POINTS
            )));
        }

        if entry_count == 0 {
            return Err(Error::InvalidConfig(
                "At least one entry point is required".into(),
            ));
        }

        // Entry mode validations
        match self.entry_mode {
            EntryMode::Shared => {
                // Shared is always valid
            }
            EntryMode::Isolated => {
                // Isolated requires multiple entries
                if matches!(self.entry, EntryPoints::Single(_)) {
                    return Err(Error::InvalidConfig(
                        "EntryMode::Isolated requires multiple entry points. \
                         Use EntryMode::Shared for single entry builds."
                            .into(),
                    ));
                }

                // Isolated + code splitting is invalid
                if self.code_splitting.is_some() {
                    return Err(Error::InvalidConfig(
                        "EntryMode::Isolated cannot use code splitting (bundles are independent). \
                         Use EntryMode::Shared for code splitting."
                            .into(),
                    ));
                }
            }
        }

        // Code splitting validations
        if let Some(config) = &self.code_splitting {
            // Code splitting requires multiple entries
            if matches!(self.entry, EntryPoints::Single(_)) {
                return Err(Error::InvalidConfig(
                    "Code splitting requires multiple entry points.".into(),
                ));
            }

            if config.min_size == 0 {
                return Err(Error::InvalidConfig(
                    "min_size must be greater than 0".into(),
                ));
            }

            if config.min_imports < 2 {
                return Err(Error::InvalidConfig(
                    "min_imports must be at least 2 for shared chunks".into(),
                ));
            }
        }

        // outfile validations
        if self.outfile.is_some() {
            if !matches!(self.entry, EntryPoints::Single(_)) {
                return Err(Error::InvalidConfig(
                    "outfile can only be used with a single entry point. Use outdir for multiple entries.".into(),
                ));
            }
            if self.code_splitting.is_some() {
                return Err(Error::InvalidConfig(
                    "outfile cannot be used with code splitting. Use outdir instead.".into(),
                ));
            }
        }

        // outdir and outfile are mutually exclusive
        if self.outdir.is_some() && self.outfile.is_some() {
            return Err(Error::InvalidConfig(
                "outdir and outfile cannot both be specified".into(),
            ));
        }

        Ok(())
    }

    /// Execute the build with these options.
    ///
    /// This is a convenience method that calls `build(self)`.
    pub async fn build(self) -> Result<super::output::BuildResult> {
        super::build(self).await
    }
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self::new(".")
    }
}

/// Normalize an entry path by cleaning redundant `.` / `..` segments.
fn normalize_entry_path(entry: impl AsRef<Path>) -> String {
    crate::builders::common::normalize_entry_path(entry)
}
