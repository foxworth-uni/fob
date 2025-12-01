//! ESM syntax validation using OXC parser

use crate::utils::offset_to_line_col;
use markdown::MdxSignal;
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

/// Validates ESM syntax (import/export statements) in MDX
///
/// This function is called by the markdown parser for each ESM block found in MDX.
/// It uses OXC parser to validate the syntax is correct JavaScript.
pub fn validate_esm_syntax(code: &str) -> MdxSignal {
    // Create allocator for OXC parser
    let allocator = Allocator::default();

    // Set source type to JavaScript module
    let source_type = SourceType::mjs();

    // Parse the code
    let ParserReturn { errors, .. } = Parser::new(&allocator, code, source_type).parse();

    // Check for parse errors
    if errors.is_empty() {
        MdxSignal::Ok
    } else {
        // Get first error for reporting
        let error = &errors[0];
        let message = format!("Invalid ESM syntax: {}", error.message);

        // Extract line and column from error
        let (line, column, context) = if let Some(labels) = &error.labels {
            if let Some(label) = labels.first() {
                // Calculate line/column from byte offset
                let offset = label.offset();
                let (line, col) = offset_to_line_col(code, offset);

                // Extract context (the line containing the error)
                let context = code
                    .lines()
                    .nth(line.saturating_sub(1))
                    .unwrap_or("")
                    .to_string();

                (line, col, context)
            } else {
                (1, 1, String::new())
            }
        } else {
            (1, 1, String::new())
        };

        // MdxSignal::Error(message, line, reason, place)
        MdxSignal::Error(
            message,
            line,
            Box::new(format!("at line {}, column {}", line, column)),
            Box::new(context),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_import() {
        let code = "import { Component } from 'react'";
        assert!(matches!(validate_esm_syntax(code), MdxSignal::Ok));
    }

    #[test]
    fn test_valid_export() {
        let code = "export const meta = { title: 'Test' }";
        assert!(matches!(validate_esm_syntax(code), MdxSignal::Ok));
    }

    #[test]
    fn test_invalid_syntax() {
        let code = "import { from";
        assert!(matches!(validate_esm_syntax(code), MdxSignal::Error(..)));
    }
}
