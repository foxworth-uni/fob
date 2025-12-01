//! Core bundler implementation (no NAPI dependencies)

use crate::api::config::{BundleConfig, MdxOptions};
use crate::conversion::format::convert_format;
use crate::conversion::sourcemap::convert_sourcemap_mode;
use crate::core::validator::validate_path;
use crate::runtime::NativeRuntime;
use fob_bundler::{BuildOptions, Runtime};
use fob_plugin_mdx::FobMdxPlugin;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Core bundler (no NAPI dependencies)
pub struct CoreBundler {
    config: BundleConfig,
    runtime: Arc<dyn Runtime>,
    cwd: PathBuf,
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

        let runtime: Arc<dyn Runtime> = Arc::new(
            NativeRuntime::new(cwd.clone())
                .map_err(|e| format!("Failed to create runtime: {}", e))?,
        );

        Ok(Self {
            config,
            runtime,
            cwd,
        })
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
    fn create_mdx_plugin(mdx_opts: Option<&MdxOptions>, cwd: &Path) -> FobMdxPlugin {
        let mut plugin = FobMdxPlugin::new(cwd.to_path_buf());

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

    /// Bundle the configured entries
    pub async fn bundle(
        &self,
    ) -> Result<crate::conversion::result::BundleResult, fob_bundler::Error> {
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
            validate_path(&cwd, &path, "output_dir")
                .map_err(|e| fob_bundler::Error::InvalidOutputPath(e.to_string()))?
        } else {
            cwd.join("dist")
        };

        // Validate entry paths
        for entry in &self.config.entries {
            let entry_path = PathBuf::from(entry);
            validate_path(&cwd, &entry_path, "entry")
                .map_err(|e| fob_bundler::Error::InvalidConfig(e.to_string()))?;
        }

        // Build
        let build_result = {
            let mut options = if self.config.entries.len() == 1 {
                BuildOptions::library(self.config.entries[0].clone())
            } else {
                BuildOptions::components(self.config.entries.clone())
            };

            options = options
                .cwd(cwd)
                .format(format)
                .runtime(self.runtime.clone());

            // Set sourcemap based on mode
            options = convert_sourcemap_mode(options, self.config.sourcemap.clone())
                .map_err(fob_bundler::Error::InvalidConfig)?;

            // Set external packages
            if let Some(external) = &self.config.external {
                options = options.external(external.clone());
            }

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
                let mdx_plugin = Self::create_mdx_plugin(self.config.mdx.as_ref(), &self.cwd);
                options = options.plugin(Arc::new(mdx_plugin));
            }

            options.build().await?
        };

        // Write files to disk
        build_result.write_to_force(&out_dir)?;

        // Convert to NAPI result
        Ok(crate::conversion::result::BundleResult::from(build_result))
    }
}
