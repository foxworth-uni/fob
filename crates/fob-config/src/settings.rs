//! Global configuration settings shared across profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalSettings {
    #[serde(default)]
    pub log_level: Option<String>,

    #[serde(default)]
    pub log_format: Option<String>,

    #[serde(default)]
    pub trace: bool,

    #[serde(default)]
    pub parallel_jobs: Option<usize>,

    #[serde(default)]
    pub environment: HashMap<String, String>,
}
