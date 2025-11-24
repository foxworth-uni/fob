//! Module parsing logic for graph walking.
//!
//! This module handles reading files, extracting scripts from framework files,
//! and parsing module structure to extract imports and exports.

use std::path::Path;

use fob_graph::collection::{CollectedExport, CollectedImport};
use fob::runtime::{Runtime, RuntimeError};

use crate::extractors::extract_scripts;
use crate::config::MAX_FILE_SIZE;
use super::WalkerError;

/// Module parser that reads and parses files.
pub struct ModuleParser;

impl ModuleParser {
    /// Process a module file: read, extract scripts, and parse structure.
    pub async fn process_module(
        &self,
        path: &Path,
        runtime: &dyn Runtime,
    ) -> Result<ParsedModule, WalkerError> {
        // Read the file
        let code = self.read_file(path, runtime).await?;

        // Extract scripts from framework files if needed
        let code_to_parse = self.extract_if_framework(path, &code)?;

        // Parse the module
        let (imports, exports, has_side_effects) = self.parse_module(&code_to_parse);

        Ok(ParsedModule {
            code,
            imports,
            exports,
            has_side_effects,
        })
    }

    /// Read a file from the filesystem with size validation.
    ///
    /// This method enforces MAX_FILE_SIZE to prevent DoS attacks and memory exhaustion.
    async fn read_file(&self, path: &Path, runtime: &dyn Runtime) -> Result<String, WalkerError> {
        // Check file size before reading
        if let Ok(metadata) = runtime.metadata(path).await {
            if metadata.size > MAX_FILE_SIZE as u64 {
                return Err(WalkerError::FileTooLarge {
                    path: path.to_path_buf(),
                    size: metadata.size as usize,
                    max: MAX_FILE_SIZE,
                });
            }
        }

        let bytes = runtime
            .read_file(path)
            .await
            .map_err(|e| WalkerError::ReadFile {
                path: path.to_path_buf(),
                source: e,
            })?;

        // Double-check size after reading (in case metadata was unavailable)
        if bytes.len() > MAX_FILE_SIZE {
            return Err(WalkerError::FileTooLarge {
                path: path.to_path_buf(),
                size: bytes.len(),
                max: MAX_FILE_SIZE,
            });
        }

        String::from_utf8(bytes).map_err(|e| WalkerError::ReadFile {
            path: path.to_path_buf(),
            source: RuntimeError::Other(format!("Invalid UTF-8: {}", e)),
        })
    }

    /// Extract scripts from framework files if applicable.
    ///
    /// For framework files (.astro, .svelte, .vue), extracts JavaScript/TypeScript
    /// from the component structure. For other files, returns the content as-is.
    fn extract_if_framework(&self, path: &Path, content: &str) -> Result<String, WalkerError> {
        let scripts =
            extract_scripts(path, content).map_err(|e| WalkerError::ExtractionFailed {
                path: path.to_path_buf(),
                source: e,
            })?;

        if scripts.is_empty() {
            // Not a framework file or no scripts found, return as-is
            return Ok(content.to_string());
        }

        // Combine multiple scripts with blank lines (same as plugin behavior)
        let combined: Vec<String> = scripts.iter().map(|s| s.source_text.to_string()).collect();
        Ok(combined.join("\n\n"))
    }

    /// Parse a module to extract imports and exports.
    ///
    /// Uses the existing parse_module_structure function from collection module.
    /// If parsing fails, returns empty imports/exports and assumes side effects.
    fn parse_module(&self, code: &str) -> (Vec<CollectedImport>, Vec<CollectedExport>, bool) {
        fob_graph::collection::parse_module_structure(code).unwrap_or_else(|_| (vec![], vec![], true))
    }
}

/// Result of parsing a module file.
pub struct ParsedModule {
    pub code: String,
    pub imports: Vec<CollectedImport>,
    pub exports: Vec<CollectedExport>,
    pub has_side_effects: bool,
}

