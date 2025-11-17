use crate::config::FobConfig;
use crate::error::{ConfigError, Result};
use figment::{
    providers::{Env, Format as _, Json, Serialized},
    Figment,
};
use std::path::Path;

impl FobConfig {
    /// Load configuration from multiple sources.
    /// Priority: CLI args > environment variables > config file > defaults
    pub fn load(args: &crate::cli::BuildArgs, config_path: Option<&Path>) -> Result<Self> {
        let mut figment = Figment::new().merge(Serialized::defaults(Self::default_config()));

        // Load fob.config.json if it exists
        let config_file = config_path.map(|p| p.to_path_buf()).or_else(|| {
            let default_path = Path::new("fob.config.json");
            default_path.exists().then(|| default_path.to_path_buf())
        });

        if let Some(path) = config_file {
            figment = figment.merge(Json::file(path));
        }

        // Merge environment variables (FOB_FORMAT, FOB_OUT_DIR, etc.)
        figment = figment.merge(Env::prefixed("FOB_").split("_"));

        // CLI args override everything (but only merge if entry was actually provided)
        // If entry is empty from CLI, skip merging to preserve config file entry
        if !args.entry.is_empty() {
            let cli_config = Self::from_build_args(args);
            figment = figment.merge(Serialized::defaults(cli_config));
        }

        figment.extract().map_err(|e| {
            ConfigError::InvalidValue {
                field: "configuration".to_string(),
                value: e.to_string(),
                hint: "Check fob.config.json syntax and field types".to_string(),
            }
            .into()
        })
    }

    /// Convert CLI BuildArgs to FobConfig.
    fn from_build_args(args: &crate::cli::BuildArgs) -> Self {
        use crate::config::conversions::*;
        use crate::config::types::DocsLlmConfig;
        use crate::config::{defaults::*, types::*};

        Self {
            entry: args.entry.clone(),
            format: args.format.into(),
            out_dir: args.out_dir.clone(),
            dts: args.dts,
            dts_bundle: if args.dts_bundle { Some(true) } else { None },
            external: args.external.clone(),
            docs: args.docs,
            docs_format: args.docs_format.map(Into::into),
            docs_dir: args.docs_dir.clone(),
            docs_include_internal: args.docs_include_internal,
            docs_enhance: args.docs_enhance,
            docs_llm: if args.docs_enhance {
                Some(DocsLlmConfig {
                    model: args
                        .docs_llm_model
                        .clone()
                        .unwrap_or_else(default_llm_model),
                    mode: args
                        .docs_enhance_mode
                        .map(|m| match m {
                            crate::cli::DocsEnhanceMode::Missing => "missing",
                            crate::cli::DocsEnhanceMode::Incomplete => "incomplete",
                            crate::cli::DocsEnhanceMode::All => "all",
                        }
                        .to_string())
                        .unwrap_or_else(default_llm_mode),
                    cache: !args.docs_no_cache,
                    url: args.docs_llm_url.clone(),
                })
            } else {
                None
            },
            docs_write_back: args.docs_write_back,
            docs_merge_strategy: args.docs_merge_strategy.map(|s| match s {
                crate::cli::DocsMergeStrategy::Merge => "merge".to_string(),
                crate::cli::DocsMergeStrategy::Replace => "replace".to_string(),
                crate::cli::DocsMergeStrategy::Skip => "skip".to_string(),
            }),
            docs_no_backup: args.docs_no_backup,
            platform: args.platform.into(),
            sourcemap: args.sourcemap.map(Into::into),
            minify: args.minify,
            target: args.target.into(),
            global_name: args.global_name.clone(),
            bundle: args.bundle,
            splitting: args.splitting,
            no_treeshake: args.no_treeshake,
            clean: args.clean,
            cwd: args.cwd.clone(),
        }
    }

    /// Get default configuration values.
    pub(crate) fn default_config() -> Self {
        use crate::config::{defaults::*, types::*};
        use std::path::PathBuf;

        Self {
            entry: vec!["src/index.ts".to_string()],
            format: Format::Esm,
            out_dir: PathBuf::from("dist"),
            dts: false,
            dts_bundle: None,
            external: vec![],
            docs: false,
            docs_format: None,
            docs_dir: None,
            docs_include_internal: false,
            docs_enhance: false,
            docs_llm: None,
            docs_write_back: false,
            docs_merge_strategy: None,
            docs_no_backup: false,
            platform: Platform::Browser,
            sourcemap: None,
            minify: false,
            target: EsTarget::Es2020,
            global_name: None,
            bundle: true, // Bundle by default
            splitting: false,
            no_treeshake: false,
            clean: false,
            cwd: None,
        }
    }
}

