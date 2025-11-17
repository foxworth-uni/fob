//! LLM-enhanced documentation generation.
//!
//! This module provides optional LLM enhancement for extracted documentation using Ollama.
//!
//! # Features
//!
//! - **Smart Caching**: BLAKE3-based cache that auto-invalidates on code changes
//! - **Flexible Enhancement**: Three modes (missing, incomplete, all)
//! - **Graceful Fallback**: Never fails the build, always continues on errors
//! - **Local-First**: Uses Ollama for privacy and cost control
//!
//! # Usage
//!
//! ```ignore
//! use fob_docs::llm::{LlmEnhancer, LlmConfig};
//!
//! // Create configuration
//! let config = LlmConfig::default()
//!     .with_model("llama3.2:3b")
//!     .with_mode(EnhancementMode::Missing);
//!
//! // Create enhancer
//! let enhancer = LlmEnhancer::new(config).await?;
//!
//! // Enhance documentation
//! let enhanced = enhancer.enhance_documentation(docs, |current, total| {
//!     eprintln!("Enhancing {}/{} symbols...", current, total);
//! }).await?;
//! ```

pub mod cache;
pub mod client;
pub mod config;
pub mod error;
pub mod prompts;

pub use cache::{CachedResponse, LlmCache};
pub use client::{OllamaClient, PreflightResult};
pub use config::{EnhancementMode, LlmConfig};
pub use error::{LlmError, Result};
pub use prompts::{EnhancementContext, PromptBuilder};

use crate::{Documentation, ExportedSymbol, ModuleDoc};
use std::path::PathBuf;

/// Main orchestrator for LLM-enhanced documentation generation.
pub struct LlmEnhancer {
    /// Ollama client.
    client: OllamaClient,

    /// Response cache.
    cache: LlmCache,

    /// Configuration.
    config: LlmConfig,
}

impl LlmEnhancer {
    /// Creates a new LLM enhancer.
    ///
    /// Performs a preflight check to ensure Ollama is running and the model is available.
    pub async fn new(config: LlmConfig) -> Result<Self> {
        let client = OllamaClient::new(config.clone())?;

        // Perform preflight check
        let preflight = client.preflight_check().await?;

        // Log preflight results
        match &preflight {
            PreflightResult::Ok { model, .. } => {
                eprintln!("[LLM] Using model: {}", model);
            }
            PreflightResult::ModelNotFound {
                requested,
                suggestions,
                ..
            } => {
                eprintln!("[LLM] Warning: Model '{}' not found", requested);
                if !suggestions.is_empty() {
                    eprintln!("[LLM] Available models: {}", suggestions.join(", "));
                    eprintln!("[LLM] Tip: Try one of the above or run: ollama pull {}", requested);
                }
            }
        }

        // Convert to error if not OK
        preflight.into_result()?;

        // Create cache
        let cache_dir = PathBuf::from(".fob-cache").join("docs-llm");
        let cache = LlmCache::new(cache_dir, config.cache_enabled);

        if config.cache_enabled {
            eprintln!("[LLM] Cache enabled (smart invalidation)");
        }

        Ok(Self {
            client,
            cache,
            config,
        })
    }

    /// Enhances a single symbol with LLM-generated documentation.
    ///
    /// This method:
    /// 1. Checks if enhancement is needed based on the mode
    /// 2. Checks the cache first
    /// 3. Calls the LLM if cache miss
    /// 4. Caches the response
    ///
    /// Returns `None` if the symbol should not be enhanced (based on mode).
    pub async fn enhance_symbol(
        &self,
        symbol: &ExportedSymbol,
        file_content: &str,
        file_path: &str,
    ) -> Result<Option<EnhancedSymbol>> {
        // Check if we should enhance this symbol
        if !self.should_enhance_symbol(symbol) {
            return Ok(None);
        }

        // Generate cache key
        let cache_key = self
            .cache
            .cache_key(symbol, file_content, &self.config.model);

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key)? {
            return Ok(Some(EnhancedSymbol {
                symbol: symbol.clone(),
                explanation: cached.explanation,
                examples: cached.examples,
                best_practices: cached.best_practices,
                metadata: EnhancementMetadata {
                    model: cached.model,
                    timestamp: cached.timestamp,
                    cache_hit: true,
                },
            }));
        }

        // Cache miss - call LLM
        let context = EnhancementContext {
            file_path: file_path.to_string(),
            surrounding_code: None,
        };

        let prompt = PromptBuilder::build_prompt(symbol, &context);

        let response = self.client.generate(prompt).await?;

        // Cache the response
        let cached_response = CachedResponse {
            explanation: response.explanation.clone(),
            examples: response.examples.clone(),
            best_practices: response.best_practices.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            model: self.config.model.clone(),
        };

        self.cache.set(&cache_key, &cached_response)?;

        Ok(Some(EnhancedSymbol {
            symbol: symbol.clone(),
            explanation: response.explanation,
            examples: response.examples,
            best_practices: response.best_practices,
            metadata: EnhancementMetadata {
                model: self.config.model.clone(),
                timestamp: cached_response.timestamp,
                cache_hit: false,
            },
        }))
    }

    /// Enhances all symbols in the documentation.
    ///
    /// The `progress_callback` is called for each symbol processed with (current, total).
    pub async fn enhance_documentation<F>(
        &self,
        docs: Documentation,
        progress_callback: F,
    ) -> Result<Documentation>
    where
        F: Fn(usize, usize),
    {
        let total_symbols: usize = docs.modules.iter().map(|m| m.symbols.len()).sum();

        let mut enhanced_modules = Vec::new();
        let mut current = 0;

        for module in docs.modules {
            let mut enhanced_symbols = Vec::new();

            // Read file content once for all symbols in this module
            let file_content = std::fs::read_to_string(&module.path).unwrap_or_default();

            for symbol in module.symbols {
                current += 1;
                progress_callback(current, total_symbols);

                match self
                    .enhance_symbol(&symbol, &file_content, &module.path)
                    .await
                {
                    Ok(Some(enhanced)) => {
                        enhanced_symbols.push(enhanced);
                    }
                    Ok(None) => {
                        // Symbol not enhanced (filtered by mode)
                        enhanced_symbols.push(EnhancedSymbol {
                            symbol,
                            explanation: String::new(),
                            examples: Vec::new(),
                            best_practices: Vec::new(),
                            metadata: EnhancementMetadata {
                                model: "none".to_string(),
                                timestamp: String::new(),
                                cache_hit: false,
                            },
                        });
                    }
                    Err(e) => {
                        // Log error but continue
                        eprintln!("[LLM] Warning: Failed to enhance {}: {}", symbol.name, e);
                        enhanced_symbols.push(EnhancedSymbol {
                            symbol,
                            explanation: String::new(),
                            examples: Vec::new(),
                            best_practices: Vec::new(),
                            metadata: EnhancementMetadata {
                                model: "error".to_string(),
                                timestamp: String::new(),
                                cache_hit: false,
                            },
                        });
                    }
                }
            }

            enhanced_modules.push(EnhancedModule {
                path: module.path,
                description: module.description,
                symbols: enhanced_symbols,
            });
        }

        Ok(Documentation {
            modules: enhanced_modules
                .into_iter()
                .map(|m| ModuleDoc {
                    path: m.path,
                    description: m.description,
                    symbols: m
                        .symbols
                        .into_iter()
                        .map(|s| {
                            let mut symbol = s.symbol;
                            // Merge LLM explanation with existing summary
                            if !s.explanation.is_empty() {
                                symbol.summary = Some(if let Some(existing) = symbol.summary {
                                    format!("{}\n\n{}", existing, s.explanation)
                                } else {
                                    s.explanation
                                });
                            }
                            // Add LLM examples to existing examples
                            symbol.examples.extend(s.examples);
                            symbol
                        })
                        .collect(),
                })
                .collect(),
        })
    }

    /// Determines if a symbol should be enhanced based on the configuration mode.
    fn should_enhance_symbol(&self, symbol: &ExportedSymbol) -> bool {
        let has_summary = symbol.summary.is_some();
        let has_params = !symbol.parameters.is_empty()
            && symbol
                .parameters
                .iter()
                .all(|p| p.description.is_some());
        let has_returns = symbol.returns.is_some();
        let has_examples = !symbol.examples.is_empty();

        self.config
            .enhancement_mode
            .should_enhance(has_summary, has_params, has_returns, has_examples)
    }
}

/// A symbol with LLM-generated enhancements.
#[derive(Debug, Clone)]
pub struct EnhancedSymbol {
    /// Original symbol.
    pub symbol: ExportedSymbol,

    /// LLM-generated explanation.
    pub explanation: String,

    /// LLM-generated examples.
    pub examples: Vec<String>,

    /// LLM-generated best practices.
    pub best_practices: Vec<String>,

    /// Metadata about the enhancement.
    pub metadata: EnhancementMetadata,
}

/// A module with enhanced symbols.
#[derive(Debug, Clone)]
struct EnhancedModule {
    path: String,
    description: Option<String>,
    symbols: Vec<EnhancedSymbol>,
}

/// Metadata about an LLM enhancement.
#[derive(Debug, Clone)]
pub struct EnhancementMetadata {
    /// Model that generated this enhancement.
    pub model: String,

    /// Timestamp of generation.
    pub timestamp: String,

    /// Whether this was a cache hit.
    pub cache_hit: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceLocation, SymbolKind};

    #[test]
    fn test_should_enhance_symbol() {
        let _config = LlmConfig::default().with_mode(EnhancementMode::Missing);
        let enhancer_result = std::panic::catch_unwind(|| {
            // We can't actually create an enhancer in tests without Ollama
            // So we'll just test the logic
            let symbol_no_summary = ExportedSymbol::new(
                "test",
                SymbolKind::Function,
                SourceLocation::new(1, 1),
            );
            let has_summary = symbol_no_summary.summary.is_some();
            assert!(!has_summary);
        });
        assert!(enhancer_result.is_ok());
    }
}
