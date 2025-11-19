//! Transformation engine for AST manipulation
//!
//! Provides a pipeline for applying multiple transformations to code.

#[cfg(feature = "transform-engine")]
mod transform_impl {
    use crate::error::{GenError, Result};
    use crate::format::FormatOptions;
    use crate::parser::{ParseOptions, ParsedProgram};
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Program;

    /// Result of a transformation pass
    #[derive(Debug)]
    pub struct TransformResult {
        /// Whether the transformation modified the AST
        pub modified: bool,
        /// Diagnostics from the transformation
        pub diagnostics: Vec<String>,
    }

    impl TransformResult {
        pub fn success() -> Self {
            Self {
                modified: false,
                diagnostics: Vec::new(),
            }
        }

        pub fn modified() -> Self {
            Self {
                modified: true,
                diagnostics: Vec::new(),
            }
        }

        pub fn with_diagnostics(mut self, diag: String) -> Self {
            self.diagnostics.push(diag);
            self
        }
    }

    /// Trait for transformation passes
    pub trait TransformPass {
        /// Name of the transformation pass
        fn name(&self) -> &'static str;

        /// Run the transformation on a parsed program
        fn run(&self, program: &mut ParsedProgram) -> Result<TransformResult>;
    }

    /// Transformation engine that applies multiple passes
    pub struct TransformEngine<'a> {
        allocator: &'a Allocator,
        passes: Vec<Box<dyn TransformPass + 'a>>,
        parse_options: ParseOptions,
        format_options: FormatOptions,
    }

    impl<'a> TransformEngine<'a> {
        /// Create a new transformation engine
        pub fn new(allocator: &'a Allocator) -> Self {
            Self {
                allocator,
                passes: Vec::new(),
                parse_options: ParseOptions::default(),
                format_options: FormatOptions::default(),
            }
        }

        /// Set parse options
        pub fn with_parse_options(mut self, opts: ParseOptions) -> Self {
            self.parse_options = opts;
            self
        }

        /// Set format options
        pub fn with_format_options(mut self, opts: FormatOptions) -> Self {
            self.format_options = opts;
            self
        }

        /// Add a transformation pass
        pub fn add_pass<P: TransformPass + 'a>(mut self, pass: P) -> Self {
            self.passes.push(Box::new(pass));
            self
        }

        /// Transform source code
        pub fn transform(&self, source: &'a str) -> Result<TransformOutput> {
            use crate::parser::parse;

            // Parse the source
            let mut parsed = parse(self.allocator, source, self.parse_options.clone())?;

            // Apply all passes
            let mut all_diagnostics = Vec::new();
            let mut any_modified = false;

            for pass in &self.passes {
                let result = pass.run(&mut parsed)?;
                any_modified |= result.modified;
                all_diagnostics.extend(result.diagnostics);
            }

            // Generate code
            use crate::JsBuilder;
            use oxc_ast::ast::Statement;

            let js = JsBuilder::new(self.allocator);
            let statements: Vec<Statement> = parsed.program.body.iter().map(|s| *s).collect();

            let code = js.program_with_format(statements, &self.format_options)?;

            Ok(TransformOutput {
                code,
                modified: any_modified,
                diagnostics: all_diagnostics,
            })
        }
    }

    /// Output from transformation engine
    #[derive(Debug)]
    pub struct TransformOutput {
        /// Generated code
        pub code: String,
        /// Whether the code was modified
        pub modified: bool,
        /// Diagnostics from transformations
        pub diagnostics: Vec<String>,
    }
}

#[cfg(feature = "transform-engine")]
pub use transform_impl::*;
