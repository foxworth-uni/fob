use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::bundle::helpers::default_true;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable persistent cache layer
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Custom cache directory (native targets)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,

    /// Maximum cache size in bytes (0 = unlimited)
    #[serde(default)]
    pub max_size: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: None,
            max_size: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOptions {
    /// Optional friendly name for the plugin
    #[serde(default)]
    pub name: Option<String>,

    /// Backend implementation used to execute the plugin
    /// Note: Extism support has been removed. Plugins are now registered directly in code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<PluginBackend>,

    /// Path to the plugin artifact
    pub path: PathBuf,

    /// Plugin-specific configuration forwarded during `Plugin::init`
    #[serde(default)]
    pub config: Value,

    /// Execution order (lower values run earlier)
    #[serde(default)]
    pub order: i32,

    /// Whether the plugin should be loaded
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Override the default pool size (per plugin instances)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_size: Option<usize>,

    /// Override the default memory ceiling (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_bytes: Option<usize>,

    /// Override the default timeout per invocation (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Environment-specific overrides (resolved via Figment profiles)
    #[serde(default)]
    pub profiles: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginBackend {
    // Extism support has been removed
    // Future plugin backends can be added here (e.g., Boa, Native)
}

