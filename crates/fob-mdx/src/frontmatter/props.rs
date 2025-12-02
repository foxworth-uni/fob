//! Prop definition types for MDX data providers
//!
//! These types represent parsed prop expressions from MDX frontmatter.
//! Frameworks like gumbo use these to resolve data at build/request/client time.

use serde::{Deserialize, Serialize};

/// Resolution strategy for when data should be fetched
///
/// Determined by the `@refresh` and `@client` options in prop expressions:
/// - No options → `BuildTime` (fetch once during build)
/// - `@refresh=60s` → `RequestTime` (fetch on server with TTL cache)
/// - `@refresh=60s @client` → `ClientTime` (fetch on client with polling)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RefreshStrategy {
    /// Fetch data once at build time (default, static)
    BuildTime,
    /// Fetch data on each request with TTL cache
    RequestTime {
        /// Time-to-live in seconds before refetching
        ttl_seconds: u64,
    },
    /// Fetch data on the client with polling
    ClientTime {
        /// Polling interval in seconds
        interval_seconds: u64,
    },
}

impl Default for RefreshStrategy {
    fn default() -> Self {
        Self::BuildTime
    }
}

impl RefreshStrategy {
    /// Returns true if this is a build-time strategy
    pub fn is_build_time(&self) -> bool {
        matches!(self, Self::BuildTime)
    }

    /// Returns true if this requires server-side fetching
    pub fn is_server_side(&self) -> bool {
        matches!(self, Self::BuildTime | Self::RequestTime { .. })
    }

    /// Returns true if this requires client-side fetching
    pub fn is_client_side(&self) -> bool {
        matches!(self, Self::ClientTime { .. })
    }

    /// Get the TTL/interval in seconds, if applicable
    pub fn interval_seconds(&self) -> Option<u64> {
        match self {
            Self::BuildTime => None,
            Self::RequestTime { ttl_seconds } => Some(*ttl_seconds),
            Self::ClientTime { interval_seconds } => Some(*interval_seconds),
        }
    }
}

/// A parsed prop definition from frontmatter
///
/// Represents expressions like: `github.repo("owner/name").stargazers_count @refresh=60s`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropDefinition {
    /// Variable name in MDX scope (e.g., "stars")
    pub name: String,

    /// Provider identifier (e.g., "github", "notion")
    pub provider: String,

    /// Method to call on provider (e.g., "repo", "database")
    pub method: String,

    /// Arguments to the method
    pub args: Vec<PropArg>,

    /// Field path to extract (e.g., ["stargazers_count"])
    pub fields: Vec<String>,

    /// Options like refresh interval
    pub options: PropOptions,

    /// Original raw expression for debugging
    pub raw: String,
}

/// Argument types for prop methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropArg {
    /// String argument
    String(String),
    /// Numeric argument
    Number(f64),
}

impl From<String> for PropArg {
    fn from(s: String) -> Self {
        PropArg::String(s)
    }
}

impl From<&str> for PropArg {
    fn from(s: &str) -> Self {
        PropArg::String(s.to_string())
    }
}

impl From<f64> for PropArg {
    fn from(n: f64) -> Self {
        PropArg::Number(n)
    }
}

/// Options that modify prop behavior
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PropOptions {
    /// Resolution strategy (build-time, request-time, or client-time)
    #[serde(default)]
    pub strategy: RefreshStrategy,

    /// Raw refresh string for debugging (e.g., "60s", "1h")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_raw: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prop_definition_serialization() {
        let prop = PropDefinition {
            name: "stars".to_string(),
            provider: "github".to_string(),
            method: "repo".to_string(),
            args: vec![PropArg::String("owner/repo".to_string())],
            fields: vec!["stargazers_count".to_string()],
            options: PropOptions {
                strategy: RefreshStrategy::RequestTime { ttl_seconds: 60 },
                refresh_raw: Some("60s".to_string()),
            },
            raw: "github.repo(\"owner/repo\").stargazers_count @refresh=60s".to_string(),
        };

        let json = serde_json::to_string(&prop).unwrap();
        assert!(json.contains("\"name\":\"stars\""));
        assert!(json.contains("\"provider\":\"github\""));
        assert!(json.contains("\"request_time\""));

        let parsed: PropDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, prop);
    }

    #[test]
    fn test_prop_options_default() {
        let opts = PropOptions::default();
        assert!(opts.strategy.is_build_time());
        assert!(opts.refresh_raw.is_none());
    }

    #[test]
    fn test_refresh_strategy_variants() {
        // Build time (default)
        let build = RefreshStrategy::BuildTime;
        assert!(build.is_build_time());
        assert!(build.is_server_side());
        assert!(!build.is_client_side());
        assert_eq!(build.interval_seconds(), None);

        // Request time
        let request = RefreshStrategy::RequestTime { ttl_seconds: 60 };
        assert!(!request.is_build_time());
        assert!(request.is_server_side());
        assert!(!request.is_client_side());
        assert_eq!(request.interval_seconds(), Some(60));

        // Client time
        let client = RefreshStrategy::ClientTime { interval_seconds: 30 };
        assert!(!client.is_build_time());
        assert!(!client.is_server_side());
        assert!(client.is_client_side());
        assert_eq!(client.interval_seconds(), Some(30));
    }

    #[test]
    fn test_refresh_strategy_serialization() {
        let build = RefreshStrategy::BuildTime;
        let json = serde_json::to_string(&build).unwrap();
        assert_eq!(json, r#"{"type":"build_time"}"#);

        let request = RefreshStrategy::RequestTime { ttl_seconds: 120 };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("request_time"));
        assert!(json.contains("120"));

        let client = RefreshStrategy::ClientTime { interval_seconds: 30 };
        let json = serde_json::to_string(&client).unwrap();
        assert!(json.contains("client_time"));
        assert!(json.contains("30"));
    }
}
