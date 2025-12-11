//! Utility functions for PHP API

use ext_php_rs::types::{ZendHashTable, Zval};

/// Extract string from Zval
pub fn zval_to_string(zval: &Zval) -> Option<String> {
    zval.str().map(|s| s.to_string())
}

/// Extract string array from ZendHashTable
pub fn array_to_strings(arr: &ZendHashTable) -> Vec<String> {
    arr.iter().filter_map(|(_, v)| zval_to_string(v)).collect()
}

/// Normalize string (lowercase, trim)
pub fn normalize_string(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Parse format string with normalization
pub fn parse_format_normalized(s: &str) -> Option<crate::types::OutputFormat> {
    crate::types::OutputFormat::from_str(&normalize_string(s))
}

/// Parse platform string with normalization
pub fn parse_platform_normalized(s: &str) -> Option<String> {
    let normalized = normalize_string(s);
    match normalized.as_str() {
        "browser" | "web" => Some("browser".to_string()),
        "node" | "nodejs" => Some("node".to_string()),
        _ => None,
    }
}

/// Parse entry mode string with normalization
pub fn parse_entry_mode_normalized(s: &str) -> Option<crate::api::primitives::EntryMode> {
    crate::api::primitives::EntryMode::from_str(&normalize_string(s))
}

/// Extract integer from Zval
pub fn zval_to_int(zval: &Zval) -> Option<i64> {
    zval.long()
}

/// Extract boolean from Zval
pub fn zval_to_bool(zval: &Zval) -> Option<bool> {
    zval.bool()
}
