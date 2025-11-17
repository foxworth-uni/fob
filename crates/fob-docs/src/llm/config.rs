//! Configuration types for LLM-enhanced documentation.

use serde::{Deserialize, Serialize};

/// Configuration for LLM-enhanced documentation generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    /// LLM provider (currently only "ollama" is supported).
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Model name to use (e.g., "llama3.2:3b", "codellama:7b").
    #[serde(default = "default_model")]
    pub model: String,

    /// Ollama server URL.
    #[serde(default = "default_url")]
    pub url: String,

    /// Whether to use caching (smart BLAKE3-based cache).
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,

    /// Enhancement mode determines which symbols to enhance.
    #[serde(default)]
    pub enhancement_mode: EnhancementMode,

    /// Temperature for generation (0.0-1.0, lower = more deterministic).
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum tokens to generate per symbol.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            url: default_url(),
            cache_enabled: default_cache_enabled(),
            enhancement_mode: EnhancementMode::default(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            timeout_seconds: default_timeout(),
        }
    }
}

impl LlmConfig {
    /// Creates a new LLM config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the Ollama server URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Sets the enhancement mode.
    pub fn with_mode(mut self, mode: EnhancementMode) -> Self {
        self.enhancement_mode = mode;
        self
    }

    /// Disables caching.
    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.model.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        if self.url.is_empty() {
            return Err("Ollama URL cannot be empty".to_string());
        }

        if !(0.0..=1.0).contains(&self.temperature) {
            return Err(format!(
                "Temperature must be between 0.0 and 1.0, got {}",
                self.temperature
            ));
        }

        if self.max_tokens == 0 {
            return Err("max_tokens must be greater than 0".to_string());
        }

        if self.timeout_seconds == 0 {
            return Err("timeout_seconds must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Determines which symbols should be enhanced with LLM-generated content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnhancementMode {
    /// Only enhance symbols that have no JSDoc summary at all.
    Missing,

    /// Enhance symbols that are missing parameters, returns, or examples.
    Incomplete,

    /// Enhance all symbols, merging LLM content with existing JSDoc.
    All,
}

impl Default for EnhancementMode {
    fn default() -> Self {
        Self::Missing
    }
}

impl EnhancementMode {
    /// Returns true if the symbol should be enhanced based on this mode.
    pub fn should_enhance(
        &self,
        has_summary: bool,
        has_params: bool,
        has_returns: bool,
        has_examples: bool,
    ) -> bool {
        match self {
            // Only enhance if there's no summary at all
            EnhancementMode::Missing => !has_summary,

            // Enhance if missing any documentation
            EnhancementMode::Incomplete => {
                !has_summary || !has_params || !has_returns || !has_examples
            }

            // Always enhance
            EnhancementMode::All => true,
        }
    }
}

impl std::fmt::Display for EnhancementMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnhancementMode::Missing => write!(f, "missing"),
            EnhancementMode::Incomplete => write!(f, "incomplete"),
            EnhancementMode::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for EnhancementMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "missing" => Ok(EnhancementMode::Missing),
            "incomplete" => Ok(EnhancementMode::Incomplete),
            "all" => Ok(EnhancementMode::All),
            _ => Err(format!(
                "Invalid enhancement mode: '{}'. Valid options: missing, incomplete, all",
                s
            )),
        }
    }
}

// Default value functions for serde
fn default_provider() -> String {
    "ollama".to_string()
}

fn default_model() -> String {
    "llama3.2:3b".to_string()
}

fn default_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_cache_enabled() -> bool {
    true
}

fn default_temperature() -> f32 {
    0.2
}

fn default_max_tokens() -> u32 {
    500
}

fn default_timeout() -> u64 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhancement_mode_should_enhance() {
        let mode = EnhancementMode::Missing;
        assert!(mode.should_enhance(false, true, true, true));
        assert!(!mode.should_enhance(true, false, false, false));

        let mode = EnhancementMode::Incomplete;
        assert!(mode.should_enhance(false, true, true, true));
        assert!(mode.should_enhance(true, false, true, true));
        assert!(mode.should_enhance(true, true, false, true));
        assert!(mode.should_enhance(true, true, true, false));
        assert!(!mode.should_enhance(true, true, true, true));

        let mode = EnhancementMode::All;
        assert!(mode.should_enhance(false, false, false, false));
        assert!(mode.should_enhance(true, true, true, true));
    }

    #[test]
    fn test_enhancement_mode_from_str() {
        assert_eq!(
            "missing".parse::<EnhancementMode>().unwrap(),
            EnhancementMode::Missing
        );
        assert_eq!(
            "incomplete".parse::<EnhancementMode>().unwrap(),
            EnhancementMode::Incomplete
        );
        assert_eq!(
            "all".parse::<EnhancementMode>().unwrap(),
            EnhancementMode::All
        );
        assert!("invalid".parse::<EnhancementMode>().is_err());
    }

    #[test]
    fn test_config_validation() {
        let config = LlmConfig::default();
        assert!(config.validate().is_ok());

        let config = LlmConfig::default().with_model("");
        assert!(config.validate().is_err());

        let config = LlmConfig {
            temperature: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
