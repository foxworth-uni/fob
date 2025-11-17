//! Development server configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default)]
    pub port: Option<u16>,

    #[serde(default = "default_open")]
    pub open: bool,

    #[serde(default = "default_hmr_path")]
    pub hmr_path: String,

    #[serde(default)]
    pub watch_paths: Vec<PathBuf>,

    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Directory containing static assets to serve in development
    #[serde(default = "default_static_dir")]
    pub static_dir: Option<PathBuf>,

    /// Cache-Control header for static assets
    #[serde(default)]
    pub static_cache_control: Option<String>,

    #[serde(default)]
    pub proxy: HashMap<String, ProxyConfig>,

    #[serde(default)]
    pub cors: Option<CorsConfig>,

    #[serde(default)]
    pub https: Option<HttpsConfig>,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: None,
            open: default_open(),
            hmr_path: default_hmr_path(),
            watch_paths: Vec::new(),
            debounce_ms: default_debounce_ms(),
            static_dir: default_static_dir(),
            static_cache_control: None,
            proxy: HashMap::new(),
            cors: None,
            https: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    pub target: Option<String>,

    #[serde(default)]
    pub ws: bool,

    #[serde(default)]
    pub change_origin: bool,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub rewrite: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorsConfig {
    #[serde(default)]
    pub origins: Vec<String>,

    #[serde(default)]
    pub methods: Vec<String>,

    #[serde(default)]
    pub headers: Vec<String>,

    #[serde(default)]
    pub expose_headers: Vec<String>,

    #[serde(default)]
    pub max_age: Option<u64>,

    #[serde(default)]
    pub allow_credentials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpsConfig {
    #[serde(default)]
    pub key: Option<PathBuf>,

    #[serde(default)]
    pub cert: Option<PathBuf>,

    #[serde(default)]
    pub pfx: Option<PathBuf>,

    #[serde(default)]
    pub passphrase: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".into()
}

fn default_open() -> bool {
    true
}

fn default_hmr_path() -> String {
    "/__joy/hmr".into()
}

fn default_debounce_ms() -> u64 {
    100
}

fn default_static_dir() -> Option<PathBuf> {
    Some(PathBuf::from("public"))
}
