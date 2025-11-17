//! Write LLM-enhanced documentation back to source files.

use crate::{Documentation, ExportedSymbol, ModuleDoc, ParameterDoc};
use anyhow::{Context, Result};
use std::fs;

/// Strategy for handling existing JSDoc when writing back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Replace existing JSDoc entirely with LLM output.
    Replace,
    /// Merge LLM output with existing JSDoc (preserves custom tags).
    Merge,
    /// Skip symbols that already have JSDoc.
    Skip,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::Merge
    }
}

/// Configuration for writing documentation back to source files.
#[derive(Debug, Clone)]
pub struct DocsWriteback {
    /// Whether to create .bak backup files before modifying.
    create_backups: bool,
    /// Whether to skip files in node_modules directories.
    skip_node_modules: bool,
    /// Strategy for handling existing JSDoc comments.
    merge_strategy: MergeStrategy,
}

impl DocsWriteback {
    /// Creates a new writeback configuration.
    pub fn new(
        create_backups: bool,
        skip_node_modules: bool,
        merge_strategy: MergeStrategy,
    ) -> Self {
        Self {
            create_backups,
            skip_node_modules,
            merge_strategy,
        }
    }

    /// Writes enhanced documentation back to source files.
    pub fn write_documentation(&self, docs: &Documentation) -> Result<WritebackReport> {
        let mut report = WritebackReport::default();

        for module in &docs.modules {
            match self.write_module(module) {
                Ok(module_report) => {
                    report.files_modified += module_report.files_modified;
                    report.symbols_updated += module_report.symbols_updated;
                    report.files_backed_up += module_report.files_backed_up;
                    report.symbols_skipped += module_report.symbols_skipped;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to write module {}: {}", module.path, e);
                    report.errors.push(format!("{}: {}", module.path, e));
                }
            }
        }

        Ok(report)
    }

    /// Writes documentation for a single module (file).
    fn write_module(&self, module: &ModuleDoc) -> Result<WritebackReport> {
        let mut report = WritebackReport::default();

        // Skip files we shouldn't modify
        if self.should_skip_file(&module.path) {
            return Ok(report);
        }

        // Read the original file content
        let original_content = fs::read_to_string(&module.path)
            .with_context(|| format!("Failed to read file: {}", module.path))?;

        let mut content = original_content.clone();
        let mut symbols_modified = 0;

        // Process symbols in reverse order (bottom-up) to maintain line numbers
        let mut symbols: Vec<_> = module.symbols.iter().collect();
        symbols.sort_by(|a, b| b.location.line.cmp(&a.location.line));

        for symbol in symbols {
            match self.insert_jsdoc_for_symbol(&mut content, symbol) {
                Ok(modified) => {
                    if modified {
                        symbols_modified += 1;
                    } else {
                        report.symbols_skipped += 1;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to update symbol {} in {}: {}",
                        symbol.name, module.path, e
                    );
                }
            }
        }

        // Only write if we modified something
        if symbols_modified > 0 {
            // Create backup if requested
            if self.create_backups {
                self.create_backup(&module.path)?;
                report.files_backed_up += 1;
            }

            // Write the modified content
            fs::write(&module.path, &content)
                .with_context(|| format!("Failed to write file: {}", module.path))?;

            // Validate the written file (basic check)
            if let Err(e) = self.validate_written_file(&module.path, &content) {
                // Restore from backup on error
                if self.create_backups {
                    self.restore_backup(&module.path)?;
                }
                return Err(e);
            }

            report.files_modified += 1;
            report.symbols_updated += symbols_modified;
        }

        Ok(report)
    }

    /// Inserts or updates JSDoc for a single symbol.
    /// Returns true if the content was modified.
    fn insert_jsdoc_for_symbol(
        &self,
        content: &mut String,
        symbol: &ExportedSymbol,
    ) -> Result<bool> {
        let lines: Vec<&str> = content.lines().collect();

        if symbol.location.line == 0 || symbol.location.line as usize > lines.len() {
            return Ok(false); // Invalid location
        }

        // Line numbers in locations are 1-based, convert to 0-based index
        let symbol_line_idx = (symbol.location.line - 1) as usize;

        // Check if there's existing JSDoc above the symbol
        let (existing_jsdoc, jsdoc_start_idx) = self.find_existing_jsdoc(&lines, symbol_line_idx);

        // Apply merge strategy
        match (existing_jsdoc.as_ref(), self.merge_strategy) {
            (Some(_), MergeStrategy::Skip) => {
                // Skip symbols that already have JSDoc
                return Ok(false);
            }
            _ => {}
        }

        // Generate new JSDoc comment
        let new_jsdoc = self.generate_jsdoc_comment(symbol);

        // Merge or replace based on strategy
        let final_jsdoc = match (existing_jsdoc, self.merge_strategy) {
            (Some(existing), MergeStrategy::Merge) => self.merge_jsdoc(&existing, &new_jsdoc),
            _ => new_jsdoc,
        };

        // Get indentation from the symbol line
        let symbol_line = lines[symbol_line_idx];
        let indentation = self.get_indentation(symbol_line);

        // Format JSDoc with proper indentation
        let formatted_jsdoc = self.format_jsdoc_with_indentation(&final_jsdoc, &indentation);

        // Reconstruct the file content
        let mut new_lines = Vec::new();

        if let Some(start_idx) = jsdoc_start_idx {
            // Replace existing JSDoc
            new_lines.extend_from_slice(&lines[..start_idx]);
            new_lines.push(&formatted_jsdoc);
            new_lines.extend_from_slice(&lines[symbol_line_idx..]);
        } else {
            // Insert new JSDoc before the symbol
            new_lines.extend_from_slice(&lines[..symbol_line_idx]);
            new_lines.push(&formatted_jsdoc);
            new_lines.extend_from_slice(&lines[symbol_line_idx..]);
        }

        *content = new_lines.join("\n");
        if !content.ends_with('\n') {
            content.push('\n');
        }

        Ok(true)
    }

    /// Finds existing JSDoc comment above a symbol.
    /// Returns (jsdoc_content, start_line_index) if found.
    fn find_existing_jsdoc(
        &self,
        lines: &[&str],
        symbol_line_idx: usize,
    ) -> (Option<String>, Option<usize>) {
        if symbol_line_idx == 0 {
            return (None, None);
        }

        // Look backwards for JSDoc end marker (*/)
        let mut end_idx = None;
        for i in (0..symbol_line_idx).rev() {
            let line = lines[i].trim();
            if line.ends_with("*/") {
                end_idx = Some(i);
                break;
            }
            if !line.is_empty() && !line.starts_with('*') && !line.starts_with("//") {
                // Found non-comment content, stop looking
                break;
            }
        }

        let end_idx = match end_idx {
            Some(idx) => idx,
            None => return (None, None),
        };

        // Look backwards for JSDoc start marker (/**)
        let mut start_idx = None;
        for i in (0..=end_idx).rev() {
            let line = lines[i].trim();
            if line.starts_with("/**") {
                start_idx = Some(i);
                break;
            }
        }

        match start_idx {
            Some(start) => {
                let jsdoc_lines: Vec<&str> = lines[start..=end_idx].to_vec();
                let jsdoc_content = jsdoc_lines.join("\n");
                (Some(jsdoc_content), Some(start))
            }
            None => (None, None),
        }
    }

    /// Generates a JSDoc comment string from an ExportedSymbol.
    fn generate_jsdoc_comment(&self, symbol: &ExportedSymbol) -> String {
        let mut lines = Vec::new();
        lines.push("/**".to_string());

        // Add summary/description
        if let Some(summary) = &symbol.summary {
            for line in summary.lines() {
                lines.push(format!(" * {}", line));
            }
            lines.push(" *".to_string());
        }

        // Add parameters
        for param in &symbol.parameters {
            let param_doc = self.format_parameter_doc(param);
            lines.push(format!(" * {}", param_doc));
        }

        // Add returns
        if let Some(returns) = &symbol.returns {
            lines.push(format!(" * @returns {{{}}}", returns));
        }

        // Add examples
        for example in &symbol.examples {
            lines.push(" * @example".to_string());
            for line in example.lines() {
                lines.push(format!(" * {}", line));
            }
        }

        lines.push(" */".to_string());
        lines.join("\n")
    }

    /// Formats a parameter for JSDoc.
    fn format_parameter_doc(&self, param: &ParameterDoc) -> String {
        let type_str = param.type_hint.as_deref().unwrap_or("*");
        let desc = param.description.as_deref().unwrap_or("");

        if desc.is_empty() {
            format!("@param {{{}}} {}", type_str, param.name)
        } else {
            format!("@param {{{}}} {} - {}", type_str, param.name, desc)
        }
    }

    /// Merges existing JSDoc with LLM-generated JSDoc.
    fn merge_jsdoc(&self, existing: &str, new: &str) -> String {
        // For MVP, prefer new content but preserve @deprecated and custom tags
        // TODO: Smarter merging that preserves more context

        let mut custom_tags = Vec::new();

        // Extract custom tags from existing JSDoc
        for line in existing.lines() {
            let trimmed = line.trim().trim_start_matches('*').trim();
            if trimmed.starts_with("@deprecated")
                || trimmed.starts_with("@internal")
                || trimmed.starts_with("@private")
                || trimmed.starts_with("@beta")
                || trimmed.starts_with("@alpha")
            {
                custom_tags.push(trimmed.to_string());
            }
        }

        if custom_tags.is_empty() {
            return new.to_string();
        }

        // Insert custom tags before the closing */
        let mut lines: Vec<String> = new.lines().map(|s| s.to_string()).collect();
        if let Some(last_idx) = lines.iter().position(|l| l.trim() == "*/") {
            for tag in custom_tags {
                lines.insert(last_idx, format!(" * {}", tag));
            }
        }

        lines.join("\n")
    }

    /// Gets the indentation (leading whitespace) from a line.
    fn get_indentation(&self, line: &str) -> String {
        line.chars()
            .take_while(|c| c.is_whitespace())
            .collect()
    }

    /// Formats JSDoc with proper indentation.
    fn format_jsdoc_with_indentation(&self, jsdoc: &str, indentation: &str) -> String {
        jsdoc
            .lines()
            .map(|line| format!("{}{}", indentation, line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Checks if a file should be skipped.
    fn should_skip_file(&self, path: &str) -> bool {
        if self.skip_node_modules {
            if path.contains("/node_modules/") || path.contains("\\node_modules\\") {
                return true;
            }
        }

        // Skip TypeScript declaration files
        if path.ends_with(".d.ts") {
            return true;
        }

        false
    }

    /// Creates a backup file.
    fn create_backup(&self, file_path: &str) -> Result<()> {
        let backup_path = format!("{}.bak", file_path);
        fs::copy(file_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path))?;
        Ok(())
    }

    /// Restores a file from its backup.
    fn restore_backup(&self, file_path: &str) -> Result<()> {
        let backup_path = format!("{}.bak", file_path);
        fs::copy(&backup_path, file_path)
            .with_context(|| format!("Failed to restore from backup: {}", backup_path))?;
        Ok(())
    }

    /// Validates that the written file has valid syntax.
    fn validate_written_file(&self, _file_path: &str, content: &str) -> Result<()> {
        // Basic validation: ensure file is readable and has content
        // For MVP, we skip OXC parsing validation to keep it simple
        // OXC parsing can be added later for stricter validation

        if content.trim().is_empty() {
            anyhow::bail!("Written file is empty");
        }

        // Check that JSDoc comments are balanced
        let open_count = content.matches("/**").count();
        let close_count = content.matches("*/").count();
        if open_count != close_count {
            anyhow::bail!("Unbalanced JSDoc comments after write");
        }

        Ok(())
    }
}

/// Report of writeback operations.
#[derive(Debug, Default)]
pub struct WritebackReport {
    /// Number of files successfully modified.
    pub files_modified: usize,
    /// Number of symbols updated with JSDoc.
    pub symbols_updated: usize,
    /// Number of backup files created.
    pub files_backed_up: usize,
    /// Number of symbols skipped (already had JSDoc).
    pub symbols_skipped: usize,
    /// Errors encountered during writeback.
    pub errors: Vec<String>,
}
