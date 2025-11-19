use crate::{Error, OutputFormat, Platform, Result, Runtime, SharedPluginable};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature = "dts-generation")]
use super::dts::DtsOptions;
use super::entry::EntryPoints;

/// Configuration options for a build operation.
///
/// This struct controls all aspects of the bundling process. Use the builder
/// pattern methods for ergonomic configuration, or construct directly for
/// full control.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Entry point(s) for the build.
    pub entry: EntryPoints,

    /// Whether to bundle dependencies into the output (default: true).
    ///
    /// - `true`: Include node_modules in the bundle (app/component mode)
    /// - `false`: Externalize all dependencies (library mode)
    pub bundle: bool,

    /// Enable code splitting for shared chunks (default: false).
    ///
    /// Only applies when `entry` is `Multiple` or `Named` and `bundle` is `true`.
    /// Extracts shared dependencies into separate chunks.
    pub splitting: bool,

    /// Packages to treat as external (not bundled).
    ///
    /// Only applies when `bundle: true`. When `bundle: false`, all
    /// dependencies are automatically externalized.
    pub external: Vec<String>,

    /// Output directory for bundled files.
    ///
    /// Cannot be used with `outfile`. Required when `splitting: true`.
    pub outdir: Option<PathBuf>,

    /// Output file path for single-entry builds.
    ///
    /// Cannot be used with `outdir` or when `splitting: true`.
    /// Only valid for `EntryPoints::Single`.
    pub outfile: Option<PathBuf>,

    /// Target runtime platform (default: Browser).
    pub platform: Platform,

    /// Output module format (default: ESM).
    pub format: OutputFormat,

    /// Source map generation strategy (default: external file).
    pub sourcemap: Option<crate::SourceMapType>,

    /// Enable JavaScript minification (default: false).
    pub minify: bool,

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
    /// Example: `{"@": "./src"}` allows `import "@/components/Button"` to resolve to `./src/components/Button`
    pub path_aliases: FxHashMap<String, String>,

    /// Working directory for module resolution (default: current directory).
    pub cwd: Option<PathBuf>,

    /// Runtime for filesystem operations (default: NativeRuntime on native, must be provided for WASM).
    ///
    /// This allows the bundler to work across different platforms:
    /// - On native targets, NativeRuntime uses std::fs
    /// - On WASM targets, BrowserRuntime bridges to JavaScript
    pub runtime: Option<Arc<dyn Runtime>>,

    /// TypeScript declaration file generation options.
    #[cfg(feature = "dts-generation")]
    pub dts: Option<DtsOptions>,
}

impl BuildOptions {
    /// Create a new BuildOptions with a single entry point.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let options = BuildOptions::new("./src/index.js");
    /// ```
    pub fn new(entry: impl AsRef<Path>) -> Self {
        Self {
            entry: EntryPoints::Single(normalize_entry_path(entry)),
            bundle: true,
            splitting: false,
            external: Vec::new(),
            outdir: None,
            outfile: None,
            platform: Platform::Browser,
            format: OutputFormat::Esm,
            sourcemap: Some(crate::SourceMapType::File),
            minify: false,
            globals: FxHashMap::default(),
            plugins: Vec::new(),
            virtual_files: FxHashMap::default(),
            path_aliases: FxHashMap::default(),
            cwd: None,
            runtime: None,
            #[cfg(feature = "dts-generation")]
            dts: None,
        }
    }

    /// Create BuildOptions with multiple entry points.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let options = BuildOptions::new_multiple(["./src/a.js", "./src/b.js"]);
    /// ```
    pub fn new_multiple<P, I>(entries: I) -> Self
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let normalized = entries.into_iter().map(normalize_entry_path).collect();

        let mut opts = Self::new("."); // Dummy entry
        opts.entry = EntryPoints::Multiple(normalized);
        opts
    }

    /// Preset: library build configuration.
    ///
    /// Optimized for npm packages:
    /// - `bundle: false` (externalize all dependencies)
    /// - `platform: Node`
    /// - Auto-generates .d.ts for TypeScript entries (if dts-generation feature enabled)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// # async fn example() -> fob_bundler::Result<()> {
    /// let result = BuildOptions::library("./src/index.ts")
    ///     .external(["react"])  // Optional: explicitly externalize
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn library(entry: impl AsRef<Path>) -> Self {
        let mut opts = Self::new(entry);
        opts.bundle = false;
        opts.platform = Platform::Node;

        #[cfg(feature = "dts-generation")]
        {
            opts.dts = Some(DtsOptions::default());
        }

        opts
    }

    /// Preset: components build (independent islands).
    ///
    /// Bundles each component separately without shared chunks:
    /// - `bundle: true`
    /// - `splitting: false`
    /// - Creates one bundle per entry
    pub fn components<P, I>(entries: I) -> Self
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let mut opts = Self::new_multiple(entries);
        opts.bundle = true;
        opts.splitting = false;
        opts
    }

    /// Preset: application build with code splitting.
    ///
    /// Optimized for web apps:
    /// - `bundle: true`
    /// - `splitting: true`
    /// - Shared dependencies extracted to chunks
    pub fn app<P, I>(entries: I) -> Self
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let mut opts = Self::new_multiple(entries);
        opts.bundle = true;
        opts.splitting = true;
        opts
    }

    /// Set whether to bundle dependencies.
    pub fn bundle(mut self, enabled: bool) -> Self {
        self.bundle = enabled;
        self
    }

    /// Set whether to enable code splitting.
    pub fn splitting(mut self, enabled: bool) -> Self {
        self.splitting = enabled;
        self
    }

    /// Add external packages that should not be bundled.
    pub fn external<I, S>(mut self, packages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for pkg in packages {
            let value = pkg.into();
            if !self.external.contains(&value) {
                self.external.push(value);
            }
        }
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

    /// Enable or disable minification.
    pub fn minify(mut self, enabled: bool) -> Self {
        self.minify = enabled;
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    ///
    /// let options = BuildOptions::new("./src/index.ts")
    ///     .path_alias("@", "./src")           // @/components/Button -> ./src/components/Button
    ///     .path_alias("~", "./lib");          // ~/utils -> ./lib/utils
    /// ```
    pub fn path_alias(mut self, alias: impl Into<String>, target: impl Into<String>) -> Self {
        self.path_aliases.insert(alias.into(), target.into());
        self
    }

    /// Add multiple path aliases at once.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_bundler::BuildOptions;
    /// use rustc_hash::FxHashMap;
    ///
    /// let mut aliases = FxHashMap::default();
    /// aliases.insert("@".to_string(), "./src".to_string());
    /// aliases.insert("~".to_string(), "./lib".to_string());
    ///
    /// let options = BuildOptions::new("./src/index.ts")
    ///     .path_aliases(aliases);
    /// ```
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
    ///
    /// This is required for WASM builds to bridge to JavaScript filesystem APIs.
    /// On native platforms, NativeRuntime is used by default if not specified.
    pub fn runtime(mut self, runtime: Arc<dyn Runtime>) -> Self {
        self.runtime = Some(runtime);
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

    /// Validate the build options for internal consistency.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid, such as:
    /// - Using `outfile` with multiple entry points
    /// - Using `outfile` with `splitting` enabled
    /// - Using both `outdir` and `outfile`
    /// - Using `splitting` with a single entry point
    /// - Using `splitting` without `bundle`
    /// - Too many entry points (DoS protection)
    pub fn validate(&self) -> Result<()> {
        // outfile requires single entry
        if self.outfile.is_some() {
            if !matches!(self.entry, EntryPoints::Single(_)) {
                return Err(Error::InvalidConfig(
                    "outfile can only be used with a single entry point".into(),
                ));
            }
            if self.splitting {
                return Err(Error::InvalidConfig(
                    "outfile cannot be used with code splitting enabled".into(),
                ));
            }
        }

        // splitting requires multiple entries and bundle: true
        if self.splitting {
            if matches!(self.entry, EntryPoints::Single(_)) {
                return Err(Error::InvalidConfig(
                    "code splitting requires multiple entry points".into(),
                ));
            }
            if !self.bundle {
                return Err(Error::InvalidConfig(
                    "code splitting requires bundle: true".into(),
                ));
            }
        }

        // outdir and outfile are mutually exclusive
        if self.outdir.is_some() && self.outfile.is_some() {
            return Err(Error::InvalidConfig(
                "outdir and outfile cannot both be specified".into(),
            ));
        }

        // DoS protection: limit entry point count
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
