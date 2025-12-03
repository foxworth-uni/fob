//! Incremental program builder for streaming code generation

use crate::error::Result;
use crate::format::FormatOptions;
use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_ast::ast::*;
use oxc_codegen::Codegen;
use oxc_span::{SPAN, SourceType};
use std::io::Write;

/// Incremental program builder for streaming code generation
///
/// Allows building programs incrementally without requiring all statements
/// upfront. Useful for generating large manifests or streaming output.
pub struct ProgramBuilder<'a> {
    ast: AstBuilder<'a>,
    body: Vec<Statement<'a>>,
    source_type: SourceType,
}

impl<'a> ProgramBuilder<'a> {
    /// Create a new program builder
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            body: Vec::new(),
            source_type: SourceType::mjs(),
        }
    }

    /// Create a new program builder with specific source type
    pub fn with_source_type(allocator: &'a Allocator, source_type: SourceType) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            body: Vec::new(),
            source_type,
        }
    }

    /// Get the underlying AST builder for creating nodes
    pub fn ast(&self) -> &AstBuilder<'a> {
        &self.ast
    }

    /// Get mutable access to the underlying AST builder
    pub fn ast_mut(&mut self) -> &mut AstBuilder<'a> {
        &mut self.ast
    }

    /// Add a statement to the program
    pub fn push(&mut self, stmt: Statement<'a>) {
        self.body.push(stmt);
    }

    /// Add multiple statements to the program
    pub fn extend(&mut self, stmts: impl IntoIterator<Item = Statement<'a>>) {
        self.body.extend(stmts);
    }

    /// Get the current number of statements
    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// Check if the builder is empty
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// Write the program to a writer with formatting options
    ///
    /// Consumes the builder since statements are moved into the program.
    pub fn write_to<W: Write>(self, writer: &mut W, _opts: &FormatOptions) -> Result<()> {
        let body_vec = self.ast.vec_from_iter(self.body);
        let program = self.ast.program(
            SPAN,
            self.source_type,
            "",
            self.ast.vec(), // imports/exports
            None,           // hashbang
            self.ast.vec(), // directives
            body_vec,
        );

        let codegen = Codegen::new();
        let result = codegen.build(&program);

        writer
            .write_all(result.code.as_bytes())
            .map_err(|e| crate::error::GenError::CodegenFailed {
                context: "Write error".to_string(),
                reason: Some(e.to_string()),
            })?;

        Ok(())
    }

    /// Generate the complete program as a string
    ///
    /// Consumes the builder since statements are moved into the program.
    pub fn generate(self, _opts: &FormatOptions) -> Result<String> {
        let body_vec = self.ast.vec_from_iter(self.body);
        let program = self.ast.program(
            SPAN,
            self.source_type,
            "",
            self.ast.vec(), // imports/exports
            None,           // hashbang
            self.ast.vec(), // directives
            body_vec,
        );

        let codegen = Codegen::new();
        let result = codegen.build(&program);

        Ok(result.code)
    }

    /// Build the program AST (for advanced usage)
    ///
    /// Consumes the builder since statements are moved into the program.
    pub fn build_program(self) -> Program<'a> {
        let body_vec = self.ast.vec_from_iter(self.body);
        self.ast.program(
            SPAN,
            self.source_type,
            "",
            self.ast.vec(), // imports/exports
            None,           // hashbang
            self.ast.vec(), // directives
            body_vec,
        )
    }
}
