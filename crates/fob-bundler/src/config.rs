//! Build configuration types.
//!
//! This module provides the new `BuildConfig` type with a cleaner, more extensible design.
//! `BuildConfig` integrates with `DeploymentTarget` for platform-specific configuration.
//!
//! Note: `BuildOptions` remains the public API for backward compatibility.
//! `BuildConfig` is available for advanced use cases and future migration.

use crate::builders::unified::{EntryPoints, MinifyLevel};
use crate::target::{ExportConditions, NodeBuiltins};
use crate::{OutputFormat, SourceMapType};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// External package pattern for resolution
#[derive(Debug, Clone)]
pub enum ExternalPattern {
    /// Exact package name match
    Exact(String),
    /// Regex pattern match
    Pattern(String),
}

impl ExternalPattern {
    pub fn exact(name: impl Into<String>) -> Self {
        Self::Exact(name.into())
    }

    pub fn pattern(pattern: impl Into<String>) -> Self {
        Self::Pattern(pattern.into())
    }
}

/// Output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub dir: PathBuf,
    pub format: OutputFormat,
    pub sourcemap: Option<SourceMapType>,
    pub splitting: bool,
}

/// Module resolution configuration
#[derive(Debug, Clone)]
pub struct ResolutionConfig {
    /// Export conditions (from DeploymentTarget or custom)
    pub conditions: ExportConditions,

    /// Path aliases (@/ -> ./src/)
    pub aliases: FxHashMap<String, String>,

    /// External packages
    pub external: Vec<ExternalPattern>,

    /// How to handle Node.js built-ins
    pub node_builtins: NodeBuiltins,
}

/// Optimization settings
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub minify: MinifyLevel,
    pub tree_shake: bool,
    pub bundle: bool,
}

/// Build configuration
///
/// This replaces BuildOptions with a cleaner, more structured design.
/// Use the builder pattern methods for ergonomic configuration.
pub struct BuildConfig {
    /// Entry points
    pub entries: EntryPoints,

    /// Output configuration
    pub output: OutputConfig,

    /// Module resolution
    pub resolution: ResolutionConfig,

    /// Optimization settings
    pub optimization: OptimizationConfig,

    /// Working directory for module resolution
    pub cwd: Option<PathBuf>,

    /// Runtime for filesystem operations
    pub runtime: Option<Arc<dyn crate::Runtime>>,

    /// Virtual files that don't exist on disk
    pub virtual_files: FxHashMap<String, String>,

    /// Plugins to use during bundling
    pub plugins: Vec<crate::SharedPluginable>,
}

impl BuildConfig {
    /// Create a new BuildConfig with a single entry point
    pub fn new(entry: impl Into<EntryPoints>) -> Self {
        let entries = entry.into();
        Self {
            entries,
            output: OutputConfig {
                dir: PathBuf::from("dist"),
                format: OutputFormat::Esm,
                sourcemap: Some(SourceMapType::File),
                splitting: false,
            },
            resolution: ResolutionConfig {
                conditions: ExportConditions::browser(),
                aliases: FxHashMap::default(),
                external: Vec::new(),
                node_builtins: NodeBuiltins::Error,
            },
            optimization: OptimizationConfig {
                minify: MinifyLevel::None,
                tree_shake: true,
                bundle: true,
            },
            cwd: None,
            runtime: None,
            virtual_files: FxHashMap::default(),
            plugins: Vec::new(),
        }
    }

    /// Apply settings from a deployment target
    pub fn for_target(mut self, target: &dyn crate::DeploymentTarget) -> Self {
        self.resolution.conditions = target.conditions();
        self.resolution.node_builtins = target.node_builtins();
        self.resolution.external.extend(
            target
                .external_packages()
                .into_iter()
                .map(ExternalPattern::exact),
        );
        self
    }

    /// Set the output directory
    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output.dir = dir.into();
        self
    }

    /// Set the output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.output.format = format;
        self
    }

    /// Enable or disable source maps
    pub fn sourcemap(mut self, enabled: bool) -> Self {
        self.output.sourcemap = if enabled {
            Some(SourceMapType::File)
        } else {
            None
        };
        self
    }

    /// Set source map type explicitly
    pub fn sourcemap_type(mut self, sourcemap: Option<SourceMapType>) -> Self {
        self.output.sourcemap = sourcemap;
        self
    }

    /// Set the minification level
    pub fn minify(mut self, level: MinifyLevel) -> Self {
        self.optimization.minify = level;
        self
    }

    /// Enable or disable bundling
    pub fn bundle(mut self, enabled: bool) -> Self {
        self.optimization.bundle = enabled;
        self
    }

    /// Enable or disable code splitting
    pub fn splitting(mut self, enabled: bool) -> Self {
        self.output.splitting = enabled;
        self
    }

    /// Add external packages
    pub fn external(mut self, packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.resolution.external.extend(
            packages
                .into_iter()
                .map(|p| ExternalPattern::exact(p.into())),
        );
        self
    }

    /// Add a path alias
    pub fn path_alias(mut self, alias: impl Into<String>, target: impl Into<String>) -> Self {
        self.resolution.aliases.insert(alias.into(), target.into());
        self
    }

    /// Set the working directory
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set the runtime
    pub fn runtime(mut self, runtime: Arc<dyn crate::Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Add a virtual file
    pub fn virtual_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
        self.virtual_files.insert(path.into(), content.into());
        self
    }

    /// Add a plugin
    pub fn plugin(mut self, plugin: crate::SharedPluginable) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Validate the build configuration
    ///
    /// Checks for internal consistency and invalid combinations.
    pub fn validate(&self) -> crate::Result<()> {
        // Code splitting requires multiple entries and bundling
        if self.output.splitting {
            if matches!(self.entries, EntryPoints::Single(_)) {
                return Err(crate::Error::InvalidConfig(
                    "code splitting requires multiple entry points".into(),
                ));
            }
            if !self.optimization.bundle {
                return Err(crate::Error::InvalidConfig(
                    "code splitting requires bundle: true".into(),
                ));
            }
        }

        // Entry count validation (DoS protection)
        const MAX_ENTRY_POINTS: usize = 1000;
        let entry_count = match &self.entries {
            EntryPoints::Single(_) => 1,
            EntryPoints::Multiple(v) => v.len(),
            EntryPoints::Named(m) => m.len(),
        };

        if entry_count > MAX_ENTRY_POINTS {
            return Err(crate::Error::InvalidConfig(format!(
                "Too many entry points: {} (max {})",
                entry_count, MAX_ENTRY_POINTS
            )));
        }

        if entry_count == 0 {
            return Err(crate::Error::InvalidConfig(
                "At least one entry point is required".into(),
            ));
        }

        Ok(())
    }

    /// Execute the build with this configuration
    ///
    /// Validates the configuration, converts to BuildOptions, and runs the build.
    pub async fn build(self) -> crate::Result<crate::BuildResult> {
        self.validate()?;
        let options = self.into_build_options();
        crate::builders::build(options).await
    }

    /// Convert BuildConfig to BuildOptions for compatibility with existing build pipeline
    fn into_build_options(self) -> crate::BuildOptions {
        use crate::Platform;

        // Map ExportConditions to Platform for backward compatibility
        let platform = match self.resolution.conditions {
            ExportConditions::Node => Platform::Node,
            ExportConditions::Edge | ExportConditions::Browser => Platform::Browser,
        };

        crate::BuildOptions {
            entry: self.entries,
            bundle: self.optimization.bundle,
            splitting: self.output.splitting,
            external: self
                .resolution
                .external
                .into_iter()
                .map(|p| match p {
                    ExternalPattern::Exact(s) => s,
                    ExternalPattern::Pattern(s) => s,
                })
                .collect(),
            outdir: Some(self.output.dir),
            outfile: None,
            platform,
            format: self.output.format,
            sourcemap: self.output.sourcemap,
            minify_level: match self.optimization.minify {
                MinifyLevel::None => None,
                MinifyLevel::Whitespace => Some("whitespace".to_string()),
                MinifyLevel::Syntax => Some("syntax".to_string()),
                MinifyLevel::Identifiers => Some("identifiers".to_string()),
            },
            globals: FxHashMap::default(),
            plugins: self.plugins,
            virtual_files: self.virtual_files,
            path_aliases: self.resolution.aliases,
            cwd: self.cwd,
            runtime: self.runtime,
            decorator: None,
            #[cfg(feature = "dts-generation")]
            dts: None,
        }
    }
}
