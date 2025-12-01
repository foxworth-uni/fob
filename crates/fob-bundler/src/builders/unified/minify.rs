//! Minification level configuration.
//!
//! Provides granular control over JavaScript minification using a string-based API
//! for configuration file compatibility.

use crate::{Error, Result};

/// Validated minification level.
///
/// Controls how aggressively JavaScript code is minified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MinifyLevel {
    /// No minification - output readable code.
    #[default]
    None,
    /// Remove whitespace and comments only.
    Whitespace,
    /// Syntax-level optimizations (property names preserved).
    Syntax,
    /// Full minification including identifier mangling.
    Identifiers,
}

impl MinifyLevel {
    /// Parse a minification level from a string.
    ///
    /// # Supported Values
    ///
    /// - `"none"` - No minification
    /// - `"whitespace"` - Remove whitespace only
    /// - `"syntax"` - Syntax-level minification
    /// - `"identifiers"` - Full minification with identifier mangling
    ///
    /// Values are case-insensitive.
    ///
    /// # Examples
    ///
    /// ```
    /// use fob_bundler::MinifyLevel;
    ///
    /// assert_eq!(MinifyLevel::parse("none").unwrap(), MinifyLevel::None);
    /// assert_eq!(MinifyLevel::parse("IDENTIFIERS").unwrap(), MinifyLevel::Identifiers);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error for unrecognized values.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" | "false" => Ok(Self::None),
            "whitespace" => Ok(Self::Whitespace),
            "syntax" => Ok(Self::Syntax),
            "identifiers" | "true" => Ok(Self::Identifiers),
            _ => Err(Error::InvalidConfig(format!(
                "Invalid minify level: '{}'. Expected: none, whitespace, syntax, identifiers",
                s
            ))),
        }
    }

    /// Convert to Rolldown's minification options.
    ///
    /// Note: Rolldown currently only supports boolean minification.
    /// All non-None levels map to `true` until Rolldown exposes granular control.
    pub(crate) fn to_rolldown_options(self) -> Option<rolldown::RawMinifyOptions> {
        match self {
            Self::None => None,
            // TODO: Map to granular Rolldown options when available
            Self::Whitespace | Self::Syntax | Self::Identifiers => {
                Some(rolldown::RawMinifyOptions::from(true))
            }
        }
    }

    /// Returns true if any minification is enabled.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::None)
    }
}

impl std::fmt::Display for MinifyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Whitespace => write!(f, "whitespace"),
            Self::Syntax => write!(f, "syntax"),
            Self::Identifiers => write!(f, "identifiers"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_levels() {
        assert_eq!(MinifyLevel::parse("none").unwrap(), MinifyLevel::None);
        assert_eq!(
            MinifyLevel::parse("whitespace").unwrap(),
            MinifyLevel::Whitespace
        );
        assert_eq!(MinifyLevel::parse("syntax").unwrap(), MinifyLevel::Syntax);
        assert_eq!(
            MinifyLevel::parse("identifiers").unwrap(),
            MinifyLevel::Identifiers
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(MinifyLevel::parse("NONE").unwrap(), MinifyLevel::None);
        assert_eq!(
            MinifyLevel::parse("Whitespace").unwrap(),
            MinifyLevel::Whitespace
        );
        assert_eq!(
            MinifyLevel::parse("IDENTIFIERS").unwrap(),
            MinifyLevel::Identifiers
        );
    }

    #[test]
    fn test_parse_bool_compat() {
        assert_eq!(
            MinifyLevel::parse("true").unwrap(),
            MinifyLevel::Identifiers
        );
        assert_eq!(MinifyLevel::parse("false").unwrap(), MinifyLevel::None);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(MinifyLevel::parse("invalid").is_err());
        assert!(MinifyLevel::parse("min").is_err());
        assert!(MinifyLevel::parse("").is_err());
    }

    #[test]
    fn test_is_enabled() {
        assert!(!MinifyLevel::None.is_enabled());
        assert!(MinifyLevel::Whitespace.is_enabled());
        assert!(MinifyLevel::Syntax.is_enabled());
        assert!(MinifyLevel::Identifiers.is_enabled());
    }

    #[test]
    fn test_display() {
        assert_eq!(MinifyLevel::None.to_string(), "none");
        assert_eq!(MinifyLevel::Whitespace.to_string(), "whitespace");
        assert_eq!(MinifyLevel::Syntax.to_string(), "syntax");
        assert_eq!(MinifyLevel::Identifiers.to_string(), "identifiers");
    }

    #[test]
    fn test_to_rolldown_options() {
        assert!(MinifyLevel::None.to_rolldown_options().is_none());
        assert!(MinifyLevel::Whitespace.to_rolldown_options().is_some());
        assert!(MinifyLevel::Syntax.to_rolldown_options().is_some());
        assert!(MinifyLevel::Identifiers.to_rolldown_options().is_some());
    }
}
