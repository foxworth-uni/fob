//! Core bundler implementation (no PyO3 dependencies)

use crate::api::config::BundleConfig;
use crate::conversion::format::convert_format;
use crate::conversion::sourcemap::convert_sourcemap_mode;
use crate::runtime::NativeRuntime;
use fob_bundler::{ExternalConfig, Runtime};
use fob_plugin_mdx::FobMdxPlugin;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

/// Core bundler (no PyO3 dependencies)
#[derive(Clone)]
pub struct CoreBundler {
    config: BundleConfig,
    runtime: Arc<NativeRuntime>,
}

impl CoreBundler {
    /// Create a new core bundler instance
    pub fn new(config: BundleConfig) -> Result<Self, String> {
        let cwd = config
            .cwd
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Failed to determine working directory".to_string())?;

        let runtime = Arc::new(
            NativeRuntime::new(cwd.clone())
                .map_err(|e| format!("Failed to create runtime: {}", e))?,
        );

        Ok(Self { config, runtime })
    }

    /// Check if MDX should be enabled (explicit config or auto-detect from entries)
    fn should_enable_mdx(config: &BundleConfig) -> bool {
        // Explicitly configured
        if config.mdx.is_some() {
            return true;
        }
        // Auto-detect: any entry ends with .mdx
        config.entries.iter().any(|e| e.ends_with(".mdx"))
    }

    /// Create MDX plugin from config options
    fn create_mdx_plugin(
        mdx_opts: Option<&crate::api::config::MdxOptions>,
        runtime: Arc<NativeRuntime>,
    ) -> FobMdxPlugin {
        let runtime_dyn: Arc<dyn fob_bundler::Runtime> = runtime;
        let mut plugin = FobMdxPlugin::new(runtime_dyn);

        if let Some(opts) = mdx_opts {
            if let Some(gfm) = opts.gfm {
                plugin.gfm = gfm;
            }
            if let Some(footnotes) = opts.footnotes {
                plugin.footnotes = footnotes;
            }
            if let Some(math) = opts.math {
                plugin.math = math;
            }
            if let Some(ref jsx_runtime) = opts.jsx_runtime {
                plugin.jsx_runtime = jsx_runtime.clone();
            }
            if let Some(use_default) = opts.use_default_plugins {
                plugin.use_default_plugins = use_default;
            }
        }

        plugin
    }

    /// Create BuildOptions using the new composable primitives.
    fn create_build_options(&self) -> fob_bundler::BuildOptions {
        let entries = &self.config.entries;

        // Determine entry mode (default: Shared for single entry, Isolated for multiple)
        let entry_mode = self
            .config
            .entry_mode
            .clone()
            .unwrap_or(if entries.len() == 1 {
                crate::api::primitives::EntryMode::Shared
            } else {
                crate::api::primitives::EntryMode::Isolated
            });

        // Convert entry mode to core type
        let core_entry_mode: fob_bundler::EntryMode = entry_mode.into();

        // Create base options based on entry count
        let mut options = if entries.len() == 1 {
            fob_bundler::BuildOptions::new(&entries[0])
        } else {
            fob_bundler::BuildOptions::new_multiple(entries)
        };

        // Set entry mode
        options.entry_mode = core_entry_mode;

        // Set code splitting
        if let Some(code_splitting) = &self.config.code_splitting {
            options.code_splitting = Some(code_splitting.clone().into());
        }

        // Set external configuration
        if let Some(true) = self.config.external_from_manifest {
            // Externalize from package.json
            let cwd = self
                .runtime
                .get_cwd()
                .unwrap_or_else(|_| PathBuf::from("."));
            options.external = ExternalConfig::FromManifest(cwd.join("package.json"));
        } else if let Some(external_packages) = &self.config.external {
            if !external_packages.is_empty() {
                // Externalize specific packages
                options.external = ExternalConfig::List(external_packages.clone());
            }
        }
        // Otherwise external stays as ExternalConfig::None (bundle everything)

        options
    }

    /// Bundle the configured entries
    pub async fn bundle(&self) -> Result<fob_bundler::BuildResult, fob_bundler::Error> {
        // Validation
        if self.config.entries.is_empty() {
            return Err(fob_bundler::Error::InvalidConfig(
                "No entries provided".to_string(),
            ));
        }

        if self.config.entries.len() > 1000 {
            return Err(fob_bundler::Error::InvalidConfig(
                "Too many entries (max 1000)".to_string(),
            ));
        }

        for entry in &self.config.entries {
            if entry.len() > 4096 {
                return Err(fob_bundler::Error::InvalidConfig(format!(
                    "Entry path too long (max 4096 chars): {}",
                    &entry[..50]
                )));
            }
        }

        let format = convert_format(self.config.format.clone());
        let cwd = self.runtime.get_cwd().map_err(|e| {
            fob_bundler::Error::Io(io::Error::other(format!("Failed to get cwd: {}", e)))
        })?;

        // Validate and normalize output directory
        let out_dir = if let Some(output_dir) = &self.config.output_dir {
            let path = PathBuf::from(output_dir);
            // If absolute path, use it directly; otherwise join with cwd
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        } else {
            cwd.join("dist")
        };

        // Validate entry paths exist (skip virtual files)
        for entry in &self.config.entries {
            // Skip validation for virtual file paths
            if entry.starts_with("virtual:") {
                continue;
            }

            let entry_path = PathBuf::from(entry);
            let full_path = if entry_path.is_absolute() {
                entry_path
            } else {
                cwd.join(&entry_path)
            };

            if !full_path.exists() {
                return Err(fob_bundler::Error::InvalidConfig(format!(
                    "Entry file does not exist: {}",
                    entry
                )));
            }
        }

        // Build
        let build_result = {
            // Determine build constructor based on primitives
            let mut options = self.create_build_options();

            // Add virtual files (for inline content support)
            if let Some(ref virtual_files) = self.config.virtual_files {
                for (path, content) in virtual_files {
                    options = options.virtual_file(path, content);
                }
            }

            options = options
                .cwd(cwd)
                .format(format)
                .runtime(self.runtime.clone());

            // Set sourcemap based on mode
            options = convert_sourcemap_mode(options, self.config.sourcemap.clone())
                .map_err(fob_bundler::Error::InvalidConfig)?;

            // External packages are already set in create_build_options()

            // Set platform
            if let Some(platform_str) = &self.config.platform {
                let platform = match platform_str.as_str() {
                    "node" => fob_bundler::Platform::Node,
                    "browser" => fob_bundler::Platform::Browser,
                    other => {
                        return Err(fob_bundler::Error::InvalidConfig(format!(
                            "Invalid platform '{}'. Expected: browser, node",
                            other
                        )));
                    }
                };
                options = options.platform(platform);
            }

            // Set minify
            if let Some(true) = self.config.minify {
                options = options.minify_level("identifiers");
            }

            // Auto-inject MDX plugin if needed
            if Self::should_enable_mdx(&self.config) {
                let mdx_plugin =
                    Self::create_mdx_plugin(self.config.mdx.as_ref(), self.runtime.clone());
                options = options.plugin(Arc::new(mdx_plugin));
            }

            options.build().await?
        };

        // Write files to disk
        build_result.write_to_force(&out_dir)?;

        Ok(build_result)
    }
}
