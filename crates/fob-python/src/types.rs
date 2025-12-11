//! Type definitions for fob-python

use pyo3::Bound;
use pyo3::prelude::*;

/// Output format for bundled code
#[derive(Clone, Debug)]
pub enum OutputFormat {
    Esm,
    Cjs,
    Iife,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "esm" => Some(Self::Esm),
            "cjs" => Some(Self::Cjs),
            "iife" => Some(Self::Iife),
            _ => None,
        }
    }
}

/// Log level for fob output
#[derive(Clone, Debug, Default)]
pub enum LogLevel {
    Silent,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "silent" => Some(Self::Silent),
            "error" => Some(Self::Error),
            "warn" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            _ => None,
        }
    }

    pub fn to_bundler_level(&self) -> fob_bundler::LogLevel {
        match self {
            Self::Silent => fob_bundler::LogLevel::Silent,
            Self::Error => fob_bundler::LogLevel::Error,
            Self::Warn => fob_bundler::LogLevel::Warn,
            Self::Info => fob_bundler::LogLevel::Info,
            Self::Debug => fob_bundler::LogLevel::Debug,
        }
    }
}

pub fn register_types(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
