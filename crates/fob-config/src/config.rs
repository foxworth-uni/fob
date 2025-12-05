//! High-level configuration structure for Fob.
//!
//! This module provides the main `JoyConfig` struct and profile merging logic.
//! For file discovery, see the `discovery` module.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bundle::{BundleOptions, PluginOptions};
use crate::dev::DevConfig;
use crate::error::{ConfigError, Result as ConfigResult};
use crate::settings::GlobalSettings;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JoyConfig {
    #[serde(default)]
    pub bundle: BundleOptions,

    #[serde(default)]
    pub dev: Option<DevConfig>,

    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    #[serde(default)]
    pub settings: GlobalSettings,

    #[serde(default)]
    #[serde(rename = "plugins")]
    #[serde(skip_serializing)]
    extra_plugins: Vec<PluginOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default)]
    pub bundle: Value,

    #[serde(default)]
    pub dev: Value,

    #[serde(default)]
    pub settings: Value,
}

impl JoyConfig {
    /// Create from serde_json::Value (for programmatic config from DB/API)
    ///
    /// # Example
    ///
    /// ```
    /// use fob_config::JoyConfig;
    /// use serde_json::json;
    /// use std::path::PathBuf;
    ///
    /// let value = json!({
    ///     "bundle": {
    ///         "entries": ["index.mdx"],
    ///         "minify": true
    ///     }
    /// });
    ///
    /// let config = JoyConfig::from_value(value).unwrap();
    /// assert_eq!(config.bundle.entries, vec![PathBuf::from("index.mdx")]);
    /// ```
    pub fn from_value(value: Value) -> ConfigResult<Self> {
        let mut config: JoyConfig =
            serde_json::from_value(value).map_err(|e| ConfigError::InvalidValue {
                field: "config".to_string(),
                hint: Some(e.to_string()),
            })?;
        config.promote_top_level_plugins();
        Ok(config)
    }

    /// Convert to serde_json::Value
    pub fn to_value(&self) -> ConfigResult<Value> {
        serde_json::to_value(self).map_err(|e| ConfigError::InvalidValue {
            field: "config".to_string(),
            hint: Some(e.to_string()),
        })
    }
}

impl JoyConfig {
    pub fn materialize_profile(mut self, profile: Option<&str>) -> ConfigResult<Self> {
        self.promote_top_level_plugins();

        if let Some(name) = profile {
            if let Some(profile_cfg) = self.profiles.get(name) {
                if !profile_cfg.bundle.is_null() {
                    let mut base = serde_json::to_value(&self.bundle).map_err(|err| {
                        ConfigError::InvalidProfileOverride {
                            message: err.to_string(),
                        }
                    })?;
                    merge_values(&mut base, &profile_cfg.bundle);
                    self.bundle = serde_json::from_value(base).map_err(|err| {
                        ConfigError::InvalidProfileOverride {
                            message: err.to_string(),
                        }
                    })?;
                }

                if !profile_cfg.dev.is_null() {
                    let mut base = match &self.dev {
                        Some(dev) => serde_json::to_value(dev).map_err(|err| {
                            ConfigError::InvalidProfileOverride {
                                message: err.to_string(),
                            }
                        })?,
                        None => Value::Null,
                    };
                    merge_values(&mut base, &profile_cfg.dev);
                    if base.is_null() {
                        self.dev = None;
                    } else {
                        self.dev = Some(serde_json::from_value(base).map_err(|err| {
                            ConfigError::InvalidProfileOverride {
                                message: err.to_string(),
                            }
                        })?);
                    }
                }

                if !profile_cfg.settings.is_null() {
                    let mut base = serde_json::to_value(&self.settings).map_err(|err| {
                        ConfigError::InvalidProfileOverride {
                            message: err.to_string(),
                        }
                    })?;
                    merge_values(&mut base, &profile_cfg.settings);
                    self.settings = serde_json::from_value(base).map_err(|err| {
                        ConfigError::InvalidProfileOverride {
                            message: err.to_string(),
                        }
                    })?;
                }
            }

            apply_plugin_profiles(&mut self.bundle.plugins, name)?;
        }

        Ok(self)
    }

    fn promote_top_level_plugins(&mut self) {
        if self.extra_plugins.is_empty() {
            return;
        }

        self.bundle.plugins.append(&mut self.extra_plugins);
    }
}

fn merge_values(target: &mut Value, update: &Value) {
    match (target, update) {
        (Value::Object(target_map), Value::Object(update_map)) => {
            for (key, value) in update_map {
                merge_values(target_map.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (target_slot, Value::Object(update_map)) => {
            let mut new_obj = serde_json::Map::with_capacity(update_map.len());
            for (key, value) in update_map {
                new_obj.insert(key.clone(), value.clone());
            }
            *target_slot = Value::Object(new_obj);
        }
        (target_slot, Value::Array(_)) => {
            *target_slot = update.clone();
        }
        (target_slot, _) => {
            *target_slot = update.clone();
        }
    }
}

fn apply_plugin_profiles(plugins: &mut [PluginOptions], profile: &str) -> ConfigResult<()> {
    for plugin in plugins {
        let Some(overrides) = plugin.profiles.get(profile).cloned() else {
            continue;
        };

        if overrides.is_null() {
            continue;
        }

        let original_profiles = plugin.profiles.clone();
        let mut merged =
            serde_json::to_value(&*plugin).map_err(|err| ConfigError::InvalidProfileOverride {
                message: err.to_string(),
            })?;
        merge_values(&mut merged, &overrides);
        let mut updated: PluginOptions =
            serde_json::from_value(merged).map_err(|err| ConfigError::InvalidProfileOverride {
                message: err.to_string(),
            })?;
        updated.profiles = original_profiles;
        *plugin = updated;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn from_value_creates_config() {
        let value = json!({
            "bundle": {
                "entries": ["index.ts"],
                "minify": true
            }
        });

        let config = JoyConfig::from_value(value).unwrap();
        assert_eq!(config.bundle.entries, vec![PathBuf::from("index.ts")]);
        assert!(config.bundle.minify);
    }

    #[test]
    fn to_value_serializes_config() {
        let mut config = JoyConfig::default();
        config.bundle.entries = vec![PathBuf::from("index.ts")];
        config.bundle.minify = true;

        let value = config.to_value().unwrap();
        assert_eq!(value["bundle"]["minify"], json!(true));
    }

    #[test]
    fn profile_merging_works() {
        let value = json!({
            "bundle": {
                "entries": ["index.ts"],
                "minify": false,
                "code_splitting": true
            },
            "profiles": {
                "production": {
                    "bundle": {
                        "minify": true,
                        "code_splitting": false
                    }
                }
            }
        });

        let config = JoyConfig::from_value(value)
            .unwrap()
            .materialize_profile(Some("production"))
            .unwrap();

        assert!(config.bundle.minify);
        assert!(!config.bundle.code_splitting);
    }
}
