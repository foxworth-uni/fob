//! Type definitions for fob-ruby

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

    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "esm" => Some(Self::Esm),
            "cjs" => Some(Self::Cjs),
            "iife" => Some(Self::Iife),
            _ => None,
        }
    }
}

impl From<OutputFormat> for fob_bundler::OutputFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Esm => Self::Esm,
            OutputFormat::Cjs => Self::Cjs,
            OutputFormat::Iife => Self::Iife,
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

    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "silent" => Some(Self::Silent),
            "error" => Some(Self::Error),
            "warn" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            _ => None,
        }
    }
}

impl From<LogLevel> for fob_bundler::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Silent => fob_bundler::LogLevel::Silent,
            LogLevel::Error => fob_bundler::LogLevel::Error,
            LogLevel::Warn => fob_bundler::LogLevel::Warn,
            LogLevel::Info => fob_bundler::LogLevel::Info,
            LogLevel::Debug => fob_bundler::LogLevel::Debug,
        }
    }
}
