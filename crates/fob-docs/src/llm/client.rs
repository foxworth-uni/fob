//! Ollama client for LLM-enhanced documentation generation.

use super::cache::CachedResponse;
use super::config::LlmConfig;
use super::error::{LlmError, Result};
use ollama_rs::{
    generation::completion::request::GenerationRequest,
    generation::parameters::{FormatType, JsonStructure},
    Ollama,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Response from the LLM after parsing.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlmResponse {
    /// Detailed explanation of the symbol.
    pub explanation: String,

    /// Code examples.
    #[serde(default)]
    pub examples: Vec<String>,

    /// Best practices or usage tips.
    #[serde(default, rename = "bestPractices")]
    pub best_practices: Vec<String>,
}

impl From<LlmResponse> for CachedResponse {
    fn from(response: LlmResponse) -> Self {
        CachedResponse {
            explanation: response.explanation,
            examples: response.examples,
            best_practices: response.best_practices,
            timestamp: chrono::Utc::now().to_rfc3339(),
            model: "unknown".to_string(), // Will be set by caller
        }
    }
}

/// Client for interacting with Ollama.
pub struct OllamaClient {
    /// Underlying Ollama client from ollama-rs.
    client: Ollama,

    /// Configuration.
    config: LlmConfig,
}

impl OllamaClient {
    /// Creates a new Ollama client.
    pub fn new(config: LlmConfig) -> Result<Self> {
        // Validate config
        config
            .validate()
            .map_err(|msg| LlmError::GenerationFailed {
                message: format!("Invalid config: {}", msg),
                source: None,
            })?;

        // Parse URL to extract scheme, host, and port
        let url = config.url.trim_end_matches('/');

        // Determine the scheme (default to http if not specified)
        let (scheme, rest) = if let Some(stripped) = url.strip_prefix("https://") {
            ("https", stripped)
        } else if let Some(stripped) = url.strip_prefix("http://") {
            ("http", stripped)
        } else {
            // No scheme provided, default to http
            ("http", url)
        };

        // Extract host and port from the remaining part
        let (hostname, port) = if let Some((h, p)) = rest.split_once(':') {
            let port_num = p.parse::<u16>().map_err(|_| LlmError::GenerationFailed {
                message: format!("Invalid port number in URL: {}", p),
                source: None,
            })?;
            (h, port_num)
        } else {
            (rest, 11434)
        };

        // Reconstruct the host with scheme for ollama-rs
        // ollama-rs expects the full URL base like "http://localhost"
        let host = format!("{}://{}", scheme, hostname);

        let client = Ollama::new(host, port);

        Ok(Self { client, config })
    }

    /// Performs a preflight check to ensure Ollama is running and the model is available.
    ///
    /// This function:
    /// 1. Checks if Ollama is running
    /// 2. Lists available models
    /// 3. Suggests alternatives if the requested model isn't found
    pub async fn preflight_check(&self) -> Result<PreflightResult> {
        // Try to list models (tests connectivity)
        let models = match self.client.list_local_models().await {
            Ok(models) => models,
            Err(e) => {
                return Err(LlmError::OllamaNotRunning {
                    url: self.config.url.clone(),
                    source: Box::new(e),
                });
            }
        };

        let available: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

        // Check if requested model is available
        let has_model = available
            .iter()
            .any(|name| name == &self.config.model || name.starts_with(&format!("{}:", &self.config.model)));

        if !has_model {
            // Try to suggest similar models
            let suggestions: Vec<String> = available
                .iter()
                .filter(|name| {
                    name.contains("llama") || name.contains("code") || name.contains("qwen")
                })
                .cloned()
                .collect();

            return Ok(PreflightResult::ModelNotFound {
                requested: self.config.model.clone(),
                available,
                suggestions,
            });
        }

        Ok(PreflightResult::Ok {
            model: self.config.model.clone(),
            available_models: available,
        })
    }

    /// Generates documentation using the LLM.
    ///
    /// Returns a validated and parsed response.
    pub async fn generate(&self, prompt: String) -> Result<LlmResponse> {
        // Use structured output with JSON schema to force valid JSON generation
        // This constrains the LLM at the llama.cpp grammar level
        let mut request = GenerationRequest::new(self.config.model.clone(), prompt);

        // Try to use structured output if available (Ollama 0.5.0+)
        // Falls back to extraction-based parsing if this fails
        request = request.format(FormatType::StructuredJson(Box::new(
            JsonStructure::new::<LlmResponse>(),
        )));

        // Note: ollama-rs GenerationRequest doesn't have builder methods for temperature, etc.
        // These are typically set at the model level in Ollama
        // For now, we use the default settings

        // Generate response
        let response = match tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout_seconds),
            self.client.generate(request),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                return Err(LlmError::GenerationFailed {
                    message: format!("Ollama generation error: {}", e),
                    source: Some(Box::new(e)),
                });
            }
            Err(_) => {
                return Err(LlmError::Timeout {
                    seconds: self.config.timeout_seconds,
                });
            }
        };

        // Parse and validate response
        // With structured output, this should succeed 99% of the time
        // Fallback extraction handles edge cases and older Ollama versions
        self.parse_response(&response.response)
    }

    /// Parses and validates the LLM response.
    fn parse_response(&self, raw_response: &str) -> Result<LlmResponse> {
        // Try to extract JSON from the response
        // Sometimes models wrap JSON in markdown code blocks
        let json_str = Self::extract_json(raw_response);

        // Parse JSON
        let parsed: LlmResponse = serde_json::from_str(json_str).map_err(|e| {
            LlmError::InvalidResponse {
                message: format!("Failed to parse JSON: {}", e),
                raw_response: raw_response.to_string(),
            }
        })?;

        // Validate content
        if parsed.explanation.trim().is_empty() {
            return Err(LlmError::InvalidResponse {
                message: "LLM returned empty explanation".to_string(),
                raw_response: raw_response.to_string(),
            });
        }

        Ok(parsed)
    }

    /// Extracts JSON from a response that might have preambles or be wrapped in markdown.
    ///
    /// Tries multiple strategies in order:
    /// 1. Markdown code block extraction (```json...```)
    /// 2. Bracket-based extraction (find first { and last })
    /// 3. Direct parsing if already clean JSON
    ///
    /// Security: Uses simple string operations, no regex (prevents ReDoS).
    fn extract_json(text: &str) -> &str {
        let text = text.trim();

        // Strategy 1: Extract from markdown code block
        if text.contains("```json") {
            if let Some(start_idx) = text.find("```json") {
                let content_start = start_idx + 7; // length of "```json"
                if let Some(newline) = text[content_start..].find('\n') {
                    let json_start = content_start + newline + 1;
                    if let Some(end_idx) = text[json_start..].find("```") {
                        return text[json_start..json_start + end_idx].trim();
                    }
                }
            }
        } else if text.contains("```") {
            // Try generic code block
            if let Some(start_idx) = text.find("```") {
                let content_start = start_idx + 3;
                if let Some(newline) = text[content_start..].find('\n') {
                    let json_start = content_start + newline + 1;
                    if let Some(end_idx) = text[json_start..].find("```") {
                        return text[json_start..json_start + end_idx].trim();
                    }
                }
            }
        }

        // Strategy 2: Bracket-based extraction for responses with preambles
        // e.g., "Here is the JSON:\n\n{...}"
        if let Some(first_brace) = text.find('{') {
            if let Some(last_brace) = text.rfind('}') {
                if first_brace < last_brace {
                    return text[first_brace..=last_brace].trim();
                }
            }
        }

        // Strategy 3: Return as-is if no wrapping detected
        text
    }
}

/// Result of a preflight check.
#[derive(Debug)]
pub enum PreflightResult {
    /// Everything is ready.
    Ok {
        model: String,
        available_models: Vec<String>,
    },

    /// Requested model not found, but Ollama is running.
    ModelNotFound {
        requested: String,
        available: Vec<String>,
        suggestions: Vec<String>,
    },
}

impl PreflightResult {
    /// Returns true if the preflight check passed.
    pub fn is_ok(&self) -> bool {
        matches!(self, PreflightResult::Ok { .. })
    }

    /// Converts to an error if not OK.
    pub fn into_result(self) -> Result<()> {
        match self {
            PreflightResult::Ok { .. } => Ok(()),
            PreflightResult::ModelNotFound {
                requested,
                available,
                ..
            } => Err(LlmError::ModelNotFound {
                model: requested,
                available_models: available,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json() {
        let with_markdown = r#"```json
{
  "explanation": "test",
  "examples": [],
  "bestPractices": []
}
```"#;

        let extracted = OllamaClient::extract_json(with_markdown);
        assert!(extracted.starts_with('{'));
        assert!(extracted.ends_with('}'));

        let plain = r#"{"explanation": "test"}"#;
        let extracted = OllamaClient::extract_json(plain);
        assert_eq!(extracted, plain);
    }

    #[test]
    fn test_llm_response_deserialization() {
        let json = r#"{
            "explanation": "This function calculates the total",
            "examples": ["const total = calc(items);"],
            "bestPractices": ["Use with arrays", "Check for null"]
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.explanation, "This function calculates the total");
        assert_eq!(response.examples.len(), 1);
        assert_eq!(response.best_practices.len(), 2);
    }

    #[test]
    fn test_config_validation_in_client() {
        let mut config = LlmConfig::default();
        config.model = "".to_string();

        let result = OllamaClient::new(config);
        assert!(result.is_err());
    }
}
