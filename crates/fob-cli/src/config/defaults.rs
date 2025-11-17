use std::path::PathBuf;

// EsTarget is re-exported from cli module via config::types
use crate::config::types::{EsTarget, Format, Platform};

pub fn default_format() -> Format {
    Format::Esm
}

pub fn default_out_dir() -> PathBuf {
    PathBuf::from("dist")
}

pub fn default_platform() -> Platform {
    Platform::Browser
}

pub fn default_target() -> EsTarget {
    EsTarget::Es2020
}

pub fn default_bundle() -> bool {
    true // Bundle by default (most common use case)
}

pub fn default_llm_model() -> String {
    "llama3.2:3b".to_string()
}

pub fn default_llm_mode() -> String {
    "missing".to_string()
}

pub fn default_llm_cache() -> bool {
    true
}

