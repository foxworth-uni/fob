//! Type definitions for fob-native
//!
//! All enum-like options are exposed as strings for better JS/TS ergonomics.
//! Strings are parsed case-insensitively.

/// Convert log level string to fob-bundler LogLevel (case-insensitive)
pub fn parse_log_level(level: Option<&str>) -> fob_bundler::LogLevel {
    match level.map(|s| s.to_lowercase()).as_deref() {
        Some("silent") => fob_bundler::LogLevel::Silent,
        Some("error") => fob_bundler::LogLevel::Error,
        Some("warn") => fob_bundler::LogLevel::Warn,
        Some("info") => fob_bundler::LogLevel::Info,
        Some("debug") => fob_bundler::LogLevel::Debug,
        _ => fob_bundler::LogLevel::Info, // default
    }
}

/// Convert entry mode string to fob-bundler EntryMode (case-insensitive)
pub fn parse_entry_mode(mode: Option<&str>) -> fob_bundler::EntryMode {
    match mode.map(|s| s.to_lowercase()).as_deref() {
        Some("shared") => fob_bundler::EntryMode::Shared,
        Some("isolated") => fob_bundler::EntryMode::Isolated,
        _ => fob_bundler::EntryMode::Shared, // default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        // Lowercase
        assert!(matches!(
            parse_log_level(Some("silent")),
            fob_bundler::LogLevel::Silent
        ));
        assert!(matches!(
            parse_log_level(Some("error")),
            fob_bundler::LogLevel::Error
        ));
        assert!(matches!(
            parse_log_level(Some("debug")),
            fob_bundler::LogLevel::Debug
        ));

        // Mixed case
        assert!(matches!(
            parse_log_level(Some("INFO")),
            fob_bundler::LogLevel::Info
        ));
        assert!(matches!(
            parse_log_level(Some("Warn")),
            fob_bundler::LogLevel::Warn
        ));

        // Default
        assert!(matches!(parse_log_level(None), fob_bundler::LogLevel::Info));
    }

    #[test]
    fn test_parse_entry_mode() {
        assert!(matches!(
            parse_entry_mode(Some("shared")),
            fob_bundler::EntryMode::Shared
        ));
        assert!(matches!(
            parse_entry_mode(Some("ISOLATED")),
            fob_bundler::EntryMode::Isolated
        ));
        assert!(matches!(
            parse_entry_mode(None),
            fob_bundler::EntryMode::Shared
        ));
    }
}
