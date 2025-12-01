//! Enhanced error handling for MDX compilation
//!
//! Provides helpful error messages with file context, line/column numbers,
//! and suggestions for common issues.

use serde::{Deserialize, Serialize};
use std::fmt;

/// MDX compilation error with enhanced context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdxError {
    /// The error message
    pub message: String,
    /// Optional file path where the error occurred
    pub file: Option<String>,
    /// Line number (1-indexed)
    pub line: Option<usize>,
    /// Column number (1-indexed)
    pub column: Option<usize>,
    /// The source code context (lines around the error)
    pub context: Option<String>,
    /// Helpful suggestion to fix the error
    pub suggestion: Option<String>,
}

impl MdxError {
    /// Create a new MDX error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            file: None,
            line: None,
            column: None,
            context: None,
            suggestion: None,
        }
    }

    /// Add file information
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Add line and column information
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Add source code context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Create an ESM syntax error
    pub fn esm_syntax_error(message: String, line: usize, column: usize, context: String) -> Self {
        Self::new(message)
            .with_location(line, column)
            .with_context(context)
            .with_suggestion(
                "Check your import/export syntax. Ensure all statements are valid ES modules.",
            )
    }

    /// Create an invalid export error
    pub fn invalid_export(code: &str) -> Self {
        Self::new(format!("Invalid export statement: {}", code))
            .with_suggestion("Only named exports, re-exports, and default exports are allowed in MDX. Remove or fix the export statement.")
    }

    /// Create a parsing error
    pub fn parse_error(message: String) -> Self {
        Self::new(format!("Failed to parse MDX: {}", message))
            .with_suggestion("Check your MDX syntax. Ensure all JSX tags are properly closed and expressions are valid.")
    }

    /// Create a conversion error
    pub fn conversion_error(message: String) -> Self {
        Self::new(format!("Failed to convert MDX to JSX: {}", message)).with_suggestion(
            "This is likely an internal error. Check that your MDX content is valid.",
        )
    }

    /// Extract context lines from source code
    pub fn extract_context(source: &str, line: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(context_lines + 1);
        let end = (line + context_lines).min(lines.len());

        let mut context = String::new();
        for (i, line_text) in lines[start..end].iter().enumerate() {
            let line_num = start + i + 1;
            let marker = if line_num == line { ">" } else { " " };
            context.push_str(&format!("{} {:3} | {}\n", marker, line_num, line_text));
        }
        context
    }
}

impl fmt::Display for MdxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Error header
        write!(f, "MDX Error: {}", self.message)?;

        // File location
        if let Some(ref file) = self.file {
            write!(f, "\n  in {}", file)?;
        }

        // Line and column
        if let (Some(line), Some(col)) = (self.line, self.column) {
            write!(f, "\n  at line {}, column {}", line, col)?;
        }

        // Context
        if let Some(ref context) = self.context {
            write!(f, "\n\n{}", context)?;
        }

        // Suggestion
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n💡 Suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

impl std::error::Error for MdxError {}

impl From<anyhow::Error> for MdxError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_error() {
        let err = MdxError::new("Something went wrong");
        assert_eq!(err.message, "Something went wrong");
    }

    #[test]
    fn test_error_with_file() {
        let err = MdxError::new("Error").with_file("test.mdx");
        assert_eq!(err.file, Some("test.mdx".to_string()));
    }

    #[test]
    fn test_error_with_location() {
        let err = MdxError::new("Error").with_location(10, 5);
        assert_eq!(err.line, Some(10));
        assert_eq!(err.column, Some(5));
    }

    #[test]
    fn test_error_display() {
        let err = MdxError::new("Parse failed")
            .with_file("test.mdx")
            .with_location(5, 10)
            .with_suggestion("Check your syntax");

        let display = format!("{}", err);
        assert!(display.contains("MDX Error: Parse failed"));
        assert!(display.contains("in test.mdx"));
        assert!(display.contains("at line 5, column 10"));
        assert!(display.contains("💡 Suggestion: Check your syntax"));
    }

    #[test]
    fn test_extract_context() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";
        let context = MdxError::extract_context(source, 3, 1);

        eprintln!("Context output:\n{}", context);

        assert!(context.contains("2 | line 2"));
        assert!(context.contains(">"));
        assert!(context.contains("3 | line 3"));
        assert!(context.contains("4 | line 4"));
    }

    #[test]
    fn test_esm_syntax_error() {
        let err = MdxError::esm_syntax_error(
            "Unexpected token".to_string(),
            5,
            10,
            "> 5 | import { foo } fro './bar'".to_string(),
        );

        assert!(err.message.contains("Unexpected token"));
        assert_eq!(err.line, Some(5));
        assert!(err.suggestion.is_some());
    }

    #[test]
    fn test_invalid_export() {
        let err = MdxError::invalid_export("export var x = 1");
        assert!(err.message.contains("Invalid export statement"));
        assert!(err.suggestion.is_some());
    }
}
