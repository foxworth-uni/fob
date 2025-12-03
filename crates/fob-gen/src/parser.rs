//! Parser façade for reading existing JavaScript/TypeScript code
//!
//! This module provides a unified interface for parsing source code into ASTs
//! that can be manipulated and regenerated using fob-gen's builders.

#[cfg(feature = "parser")]
mod parser_impl {
    use crate::error::{GenError, Result};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    /// Parse options for reading source code
    #[derive(Debug, Clone)]
    pub struct ParseOptions {
        /// Source type (JavaScript, TypeScript, JSX, TSX)
        pub source_type: SourceType,
        /// Allow parsing errors (returns partial AST)
        pub allow_errors: bool,
    }

    impl Default for ParseOptions {
        fn default() -> Self {
            Self {
                source_type: SourceType::mjs(),
                allow_errors: false,
            }
        }
    }

    impl ParseOptions {
        /// Create parse options from file path (auto-detects source type)
        pub fn from_path(path: &str) -> Self {
            Self {
                source_type: SourceType::from_path(path).unwrap_or(SourceType::mjs()),
                allow_errors: false,
            }
        }

        /// Create parse options for TypeScript
        pub fn typescript() -> Self {
            Self {
                source_type: SourceType::ts(),
                allow_errors: false,
            }
        }

        /// Create parse options for JSX
        pub fn jsx() -> Self {
            Self {
                source_type: SourceType::jsx(),
                allow_errors: false,
            }
        }

        /// Create parse options for TSX
        pub fn tsx() -> Self {
            Self {
                source_type: SourceType::tsx(),
                allow_errors: false,
            }
        }
    }

    /// Parse diagnostic information
    #[derive(Debug, Clone)]
    pub struct ParseDiagnostic {
        /// Error message
        pub message: String,
        /// Span information (if available)
        pub span: Option<(u32, u32)>,
    }

    /// Parsed program with AST and metadata
    pub struct ParsedProgram<'a> {
        /// The parsed AST program
        pub program: oxc_ast::ast::Program<'a>,
        /// Parse diagnostics (errors/warnings)
        pub diagnostics: Vec<ParseDiagnostic>,
        /// Original source text
        pub source_text: &'a str,
        /// Allocator used for AST nodes
        pub allocator: &'a Allocator,
    }

    impl<'a> ParsedProgram<'a> {
        /// Get the program AST
        pub fn ast(&self) -> &oxc_ast::ast::Program<'a> {
            &self.program
        }

        /// Get mutable access to the program AST
        pub fn ast_mut(&mut self) -> &mut oxc_ast::ast::Program<'a> {
            &mut self.program
        }

        /// Get the allocator for creating new AST nodes
        pub fn allocator(&self) -> &'a Allocator {
            self.allocator
        }

        /// Check if parsing had errors
        pub fn has_errors(&self) -> bool {
            !self.diagnostics.is_empty()
        }
    }

    /// Parse source code into an AST
    ///
    /// # Arguments
    ///
    /// * `allocator` - Allocator for AST nodes (must outlive the returned program)
    /// * `source` - Source code to parse
    /// * `options` - Parse options
    ///
    /// # Returns
    ///
    /// Parsed program with AST and diagnostics
    pub fn parse<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        options: ParseOptions,
    ) -> Result<ParsedProgram<'a>> {
        let parser = Parser::new(allocator, source, options.source_type);
        let result = parser.parse();

        let diagnostics: Vec<ParseDiagnostic> = result
            .errors
            .iter()
            .map(|err| ParseDiagnostic {
                message: format!("{:?}", err),
                span: None, // Span extraction requires more complex handling
            })
            .collect();

        if !options.allow_errors && !diagnostics.is_empty() {
            return Err(GenError::CodegenFailed {
                context: "Parse errors".to_string(),
                reason: Some(
                    diagnostics
                        .iter()
                        .map(|d| d.message.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            });
        }

        Ok(ParsedProgram {
            program: result.program,
            diagnostics,
            source_text: source,
            allocator,
        })
    }
}

#[cfg(feature = "parser")]
pub use parser_impl::*;
