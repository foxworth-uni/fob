use crate::config::FobConfig;
use crate::error::{ConfigError, Result};

/// Validate global name follows JavaScript identifier rules.
pub fn validate_global_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(ConfigError::InvalidValue {
            field: "globalName".to_string(),
            value: "".to_string(),
            hint: "Global name cannot be empty".to_string(),
        }
        .into());
    }

    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return Err(ConfigError::InvalidValue {
            field: "globalName".to_string(),
            value: name.to_string(),
            hint: format!(
                "Must start with letter, underscore, or dollar sign (got '{}')",
                first
            ),
        }
        .into());
    }

    for c in name.chars() {
        if !c.is_alphanumeric() && c != '_' && c != '$' {
            return Err(ConfigError::InvalidValue {
                field: "globalName".to_string(),
                value: name.to_string(),
                hint: format!("Invalid character '{}' in identifier", c),
            }
            .into());
        }
    }

    Ok(())
}

impl FobConfig {
    /// Validate configuration for logical consistency.
    pub fn validate(&self) -> Result<()> {
        if self.entry.is_empty() {
            return Err(ConfigError::MissingField {
                field: "entry".to_string(),
                hint: "Provide at least one entry point".to_string(),
            }
            .into());
        }

        if self.format == crate::config::types::Format::Iife && self.global_name.is_none() {
            return Err(ConfigError::MissingField {
                field: "globalName".to_string(),
                hint: "IIFE format requires a global variable name".to_string(),
            }
            .into());
        }

        if self.dts_bundle == Some(true) && !self.dts {
            return Err(ConfigError::InvalidValue {
                field: "dtsBundle".to_string(),
                value: "true".to_string(),
                hint: "Bundling declarations requires dts: true".to_string(),
            }
            .into());
        }

        if !self.docs {
            if self.docs_format.is_some() {
                return Err(ConfigError::InvalidValue {
                    field: "docsFormat".to_string(),
                    value: "set".to_string(),
                    hint: "Set docs: true to configure documentation format".to_string(),
                }
                .into());
            }
            if self.docs_dir.is_some() {
                return Err(ConfigError::InvalidValue {
                    field: "docsDir".to_string(),
                    value: "set".to_string(),
                    hint: "Set docs: true to configure documentation directory".to_string(),
                }
                .into());
            }
            if self.docs_include_internal {
                return Err(ConfigError::InvalidValue {
                    field: "docsIncludeInternal".to_string(),
                    value: "true".to_string(),
                    hint: "Set docs: true to include @internal symbols".to_string(),
                }
                .into());
            }
        }

        // LLM enhancement validation
        if self.docs_enhance && !self.docs {
            return Err(ConfigError::InvalidValue {
                field: "docsEnhance".to_string(),
                value: "true".to_string(),
                hint: "LLM enhancement requires docs: true".to_string(),
            }
            .into());
        }

        if self.docs_llm.is_some() && !self.docs_enhance {
            return Err(ConfigError::InvalidValue {
                field: "docsLlm".to_string(),
                value: "set".to_string(),
                hint: "docsLlm configuration requires docsEnhance: true".to_string(),
            }
            .into());
        }

        // Validate LLM mode if present
        if let Some(ref llm_config) = self.docs_llm {
            if !["missing", "incomplete", "all"].contains(&llm_config.mode.as_str()) {
                return Err(ConfigError::InvalidValue {
                    field: "docsLlm.mode".to_string(),
                    value: llm_config.mode.clone(),
                    hint: "Mode must be 'missing', 'incomplete', or 'all'".to_string(),
                }
                .into());
            }
        }

        // Code splitting requires bundling
        if self.splitting && !self.bundle {
            return Err(ConfigError::InvalidValue {
                field: "splitting".to_string(),
                value: "true".to_string(),
                hint: "Code splitting requires bundle: true".to_string(),
            }
            .into());
        }

        if let Some(ref name) = self.global_name {
            validate_global_name(name)?;
        }

        Ok(())
    }
}

