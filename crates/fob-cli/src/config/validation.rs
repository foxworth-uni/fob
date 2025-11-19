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
