use crate::config::FobConfig;
use crate::error::{ConfigError, Result};
use figment::{
    providers::{Env, Format as _, Json, Serialized},
    Error as FigmentError, Figment,
};
use std::path::Path;

impl FobConfig {
    /// Load configuration from multiple sources.
    /// Priority: CLI args > environment variables > config file > defaults
    pub fn load(args: &crate::cli::BuildArgs, config_path: Option<&Path>) -> Result<Self> {
        let defaults = Self::default_config();
        let default_entry = defaults.entry.clone();
        let mut figment = Figment::new().merge(Serialized::defaults(defaults));

        // Load fob.config.json if it exists
        let mut config_file = config_path
            .map(|p| p.to_path_buf())
            .or_else(|| {
                args.cwd.as_ref().and_then(|cwd| {
                    let cwd_path = if cwd.is_absolute() {
                        cwd.clone()
                    } else {
                        std::env::current_dir().ok()?.join(cwd)
                    };
                    let candidate = cwd_path.join("fob.config.json");
                    candidate.exists().then_some(candidate)
                })
            })
            .or_else(|| {
                let default_path = Path::new("fob.config.json");
                default_path.exists().then_some(default_path.to_path_buf())
            });
        let has_config_file = config_file.is_some();

        if let Some(path) = config_file.take() {
            figment = figment.merge(Json::file(path));
        }

        // Merge environment variables (FOB_FORMAT, FOB_OUT_DIR, etc.)
        let env_provider = Env::prefixed("FOB_")
            .map(|key| env_key_to_camel_case(key.as_str()).into())
            .lowercase(false);
        figment = figment.merge(env_provider);

        // Preserve existing entry when CLI explicitly omits entries so other CLI flags still apply.
        let base_entry =
            if args.entry.is_none() || args.entry.as_ref().map_or(false, |e| e.is_empty()) {
                let base: Self = figment.clone().extract().map_err(convert_figment_error)?;
                Some(base.entry)
            } else {
                None
            };

        let cli_config = Self::from_build_args(args);
        figment = figment.merge(Serialized::defaults(cli_config));

        let mut config: Self = figment.extract().map_err(convert_figment_error)?;

        if let Some(entry) = base_entry {
            config.entry = entry;
        }

        if args.entry.is_none()
            && !has_config_file
            && config.entry == default_entry
            && !std::env::vars().any(|(key, _)| key == "FOB_ENTRY" || key.starts_with("FOB_ENTRY_"))
        {
            return Err(ConfigError::MissingField {
                field: "entry".to_string(),
                hint: "Specify at least one entry point with --entry or fob.config.json"
                    .to_string(),
            }
            .into());
        }

        Ok(config)
    }

    /// Convert CLI BuildArgs to FobConfig.
    fn from_build_args(args: &crate::cli::BuildArgs) -> Self {
        Self {
            entry: args.entry.clone().unwrap_or_default(),
            format: args.format.into(),
            out_dir: args.out_dir.clone(),
            dts: args.dts,
            dts_bundle: if args.dts_bundle { Some(true) } else { None },
            external: args.external.clone(),
            platform: args.platform.into(),
            sourcemap: args.sourcemap.map(Into::into),
            minify: args.minify,
            target: args.target,
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
        use crate::config::types::*;
        use std::path::PathBuf;

        Self {
            entry: vec!["src/index.ts".to_string()],
            format: Format::Esm,
            out_dir: PathBuf::from("dist"),
            dts: false,
            dts_bundle: None,
            external: vec![],
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

fn convert_figment_error(e: FigmentError) -> crate::error::CliError {
    ConfigError::InvalidValue {
        field: "configuration".to_string(),
        value: e.to_string(),
        hint: "Check fob.config.json syntax and field types".to_string(),
    }
    .into()
}

fn env_key_to_camel_case(key: &str) -> String {
    let mut parts = key.split('_').filter(|part| !part.is_empty());
    let mut result = String::new();
    if let Some(first) = parts.next() {
        result.push_str(&first.to_ascii_lowercase());
    }

    for part in parts {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            result.push(first_char.to_ascii_uppercase());
            for c in chars {
                result.push(c.to_ascii_lowercase());
            }
        }
    }

    result
}
