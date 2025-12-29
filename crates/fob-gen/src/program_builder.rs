//! Comprehensive program builder for JavaScript code generation

use crate::error::Result;
use crate::format::FormatOptions;
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast::{AstBuilder, NONE};
use oxc_codegen::Codegen;
use oxc_span::{Atom, SPAN, SourceType};
use std::io::Write;

/// Comprehensive program builder for JavaScript code generation.
///
/// This builder provides a high-level, ergonomic API for generating JavaScript code,
/// combining statement-level construction with whole-program generation.
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

    /// Get the underlying allocator (for advanced usage)
    pub fn allocator(&self) -> &'a Allocator {
        self.ast.allocator
    }

    /// Get the underlying AstBuilder (for advanced usage)
    pub fn ast(&self) -> &AstBuilder<'a> {
        &self.ast
    }

    // ===== CORE PRIMITIVES =====

    /// Create an identifier expression: `name`
    pub fn ident(&self, name: impl Into<Atom<'a>>) -> Expression<'a> {
        self.ast.expression_identifier(SPAN, name)
    }

    /// Create a string literal: `"value"`
    pub fn string(&self, value: impl Into<Atom<'a>>) -> Expression<'a> {
        self.ast.expression_string_literal(SPAN, value, None)
    }

    /// Create a number literal: `42`
    pub fn number(&self, value: f64) -> Expression<'a> {
        self.ast
            .expression_numeric_literal(SPAN, value, None, NumberBase::Decimal)
    }

    /// Create null: `null`
    pub fn null(&self) -> Expression<'a> {
        self.ast.expression_null_literal(SPAN)
    }

    /// Create boolean: `true` or `false`
    pub fn bool(&self, value: bool) -> Expression<'a> {
        self.ast.expression_boolean_literal(SPAN, value)
    }

    // ===== COMPLEX EXPRESSIONS =====

    /// Create a member expression: `obj.prop`
    pub fn member(&self, object: Expression<'a>, property: impl Into<Atom<'a>>) -> Expression<'a> {
        let prop_name = self.ast.identifier_name(SPAN, property);
        let member = self
            .ast
            .member_expression_static(SPAN, object, prop_name, false);
        Expression::from(member)
    }

    /// Create a computed member expression: `obj[expr]`
    ///
    /// This is critical for array indexing: `arr[0]`, `pageRoutes[idx]`, etc.
    pub fn computed_member(&self, object: Expression<'a>, index: Expression<'a>) -> Expression<'a> {
        let member = self
            .ast
            .member_expression_computed(SPAN, object, index, false);
        Expression::from(member)
    }

    /// Create a call expression: `callee(args...)`
    pub fn call(&self, callee: Expression<'a>, args: Vec<Argument<'a>>) -> Expression<'a> {
        let args_vec = self.ast.vec_from_iter(args);
        let call = self
            .ast
            .call_expression(SPAN, callee, NONE, args_vec, false);
        Expression::CallExpression(self.ast.alloc(call))
    }

    /// Create an argument from an expression
    pub fn arg(&self, expr: Expression<'a>) -> Argument<'a> {
        Argument::from(expr)
    }

    /// Create an object expression: `{ key1: value1, key2: value2 }`
    pub fn object(&self, props: Vec<ObjectPropertyKind<'a>>) -> Expression<'a> {
        let props_vec = self.ast.vec_from_iter(props);
        self.ast.expression_object(SPAN, props_vec)
    }

    /// Create an object property: `key: value`
    pub fn prop(&self, key: impl Into<Atom<'a>>, value: Expression<'a>) -> ObjectPropertyKind<'a> {
        let key_name = self.ast.identifier_name(SPAN, key);
        let property = self.ast.object_property(
            SPAN,
            PropertyKind::Init,
            PropertyKey::StaticIdentifier(self.ast.alloc(key_name)),
            value,
            false,
            false,
            false,
        );
        ObjectPropertyKind::ObjectProperty(self.ast.alloc(property))
    }

    /// Create an array expression: `[elem1, elem2, ...]`
    pub fn array(&self, elements: Vec<Expression<'a>>) -> Expression<'a> {
        let array_elements: Vec<_> = elements
            .into_iter()
            .map(ArrayExpressionElement::from)
            .collect();
        let elements_vec = self.ast.vec_from_iter(array_elements);
        self.ast.expression_array(SPAN, elements_vec)
    }

    /// Create an arrow function with expression body: `(params) => expr`
    pub fn arrow_fn(&self, params: Vec<&'a str>, body: Expression<'a>) -> Expression<'a> {
        let param_items: Vec<_> = params
            .into_iter()
            .map(|name| {
                let pattern = self.ast.binding_pattern(
                    self.ast.binding_pattern_kind_binding_identifier(SPAN, name),
                    NONE,
                    false,
                );
                self.ast
                    .formal_parameter(SPAN, self.ast.vec(), pattern, None, false, false)
            })
            .collect();

        let items_vec = self.ast.vec_from_iter(param_items);
        let formal_params = self.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            items_vec,
            NONE,
        );

        // Create a FunctionBody with just an expression (return statement)
        let return_stmt = self.ast.statement_return(SPAN, Some(body));
        let stmts = self.ast.vec1(return_stmt);
        let function_body = self.ast.function_body(SPAN, self.ast.vec(), stmts);

        self.ast.expression_arrow_function(
            SPAN,
            false, // expression = false because we're using a block
            false, // async
            NONE,
            formal_params,
            NONE,
            self.ast.alloc(function_body),
        )
    }

    /// Create an arrow function with block body: `(params) => { stmts }`
    pub fn arrow_fn_block(
        &self,
        params: Vec<&'a str>,
        stmts: Vec<Statement<'a>>,
    ) -> Expression<'a> {
        let param_items: Vec<_> = params
            .into_iter()
            .map(|name| {
                let pattern = self.ast.binding_pattern(
                    self.ast.binding_pattern_kind_binding_identifier(SPAN, name),
                    NONE,
                    false,
                );
                self.ast
                    .formal_parameter(SPAN, self.ast.vec(), pattern, None, false, false)
            })
            .collect();

        let items_vec = self.ast.vec_from_iter(param_items);
        let formal_params = self.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            items_vec,
            NONE,
        );

        let body_stmts = self.ast.vec_from_iter(stmts);
        let function_body = self.ast.function_body(SPAN, self.ast.vec(), body_stmts);

        self.ast.expression_arrow_function(
            SPAN,
            false, // block body (not expression)
            false, // async
            Option::<TSTypeParameterDeclaration>::None,
            formal_params,
            Option::<TSTypeAnnotation>::None,
            self.ast.alloc(function_body),
        )
    }

    // ===== STATEMENTS =====

    /// Create a const declaration: `const name = init;`
    pub fn const_decl(&self, name: impl Into<Atom<'a>>, init: Expression<'a>) -> Statement<'a> {
        let pattern = self.ast.binding_pattern(
            self.ast.binding_pattern_kind_binding_identifier(SPAN, name),
            NONE,
            false,
        );
        let declarator = self.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            pattern,
            Some(init),
            false,
        );
        let declarations = self.ast.vec1(declarator);
        let var_decl = self.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Const,
            declarations,
            false,
        );
        Statement::VariableDeclaration(self.ast.alloc(var_decl))
    }

    /// Create a let declaration: `let name = init;`
    pub fn let_decl(
        &self,
        name: impl Into<Atom<'a>>,
        init: Option<Expression<'a>>,
    ) -> Statement<'a> {
        let pattern = self.ast.binding_pattern(
            self.ast.binding_pattern_kind_binding_identifier(SPAN, name),
            NONE,
            false,
        );
        let declarator =
            self.ast
                .variable_declarator(SPAN, VariableDeclarationKind::Let, pattern, init, false);
        let declarations = self.ast.vec1(declarator);
        let var_decl =
            self.ast
                .variable_declaration(SPAN, VariableDeclarationKind::Let, declarations, false);
        Statement::VariableDeclaration(self.ast.alloc(var_decl))
    }

    /// Create an expression statement
    pub fn expr_stmt(&self, expr: Expression<'a>) -> Statement<'a> {
        self.ast.statement_expression(SPAN, expr)
    }

    /// Create an if statement: `if (test) { consequent } else { alternate }`
    pub fn if_stmt(
        &self,
        test: Expression<'a>,
        consequent: Vec<Statement<'a>>,
        alternate: Option<Vec<Statement<'a>>>,
    ) -> Statement<'a> {
        let consequent_stmts = self.ast.vec_from_iter(consequent);
        let consequent_block = self.ast.statement_block(SPAN, consequent_stmts);

        let alternate_block = alternate.map(|stmts| {
            let alt_stmts = self.ast.vec_from_iter(stmts);
            self.ast.statement_block(SPAN, alt_stmts)
        });

        Statement::IfStatement(self.ast.alloc(IfStatement {
            span: SPAN,
            test,
            consequent: consequent_block,
            alternate: alternate_block,
        }))
    }

    /// Create a return statement: `return expr;`
    pub fn return_stmt(&self, expr: Option<Expression<'a>>) -> Statement<'a> {
        Statement::ReturnStatement(self.ast.alloc(ReturnStatement {
            span: SPAN,
            argument: expr,
        }))
    }

    /// Create a throw statement: `throw expr;`
    pub fn throw(&self, expr: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.ast.alloc(ThrowStatement {
            span: SPAN,
            argument: expr,
        }))
    }

    // ===== IMPORTS & EXPORTS =====

    /// Create default import: `import local from 'source';`
    pub fn import_default(
        &self,
        local: impl Into<Atom<'a>>,
        source: impl Into<Atom<'a>>,
    ) -> ModuleDeclaration<'a> {
        let binding = self.ast.binding_identifier(SPAN, local);
        let specifier = self.ast.import_default_specifier(SPAN, binding);
        let specifiers = self
            .ast
            .vec1(ImportDeclarationSpecifier::ImportDefaultSpecifier(
                self.ast.alloc(specifier),
            ));
        let source_literal = self.ast.string_literal(SPAN, source, None);
        self.ast.module_declaration_import_declaration(
            SPAN,
            Some(specifiers),
            source_literal,
            None, // phase
            NONE, // with_clause
            ImportOrExportKind::Value,
        )
    }

    /// Create side-effect import: `import 'source';`
    pub fn import_side_effect(&self, source: impl Into<Atom<'a>>) -> ModuleDeclaration<'a> {
        let source_literal = self.ast.string_literal(SPAN, source, None);
        self.ast.module_declaration_import_declaration(
            SPAN,
            None, // no specifiers for side-effect imports
            source_literal,
            None, // phase
            NONE, // with_clause
            ImportOrExportKind::Value,
        )
    }

    /// Create named imports: `import { name1, name2 } from 'source';`
    pub fn import_named(
        &self,
        names: Vec<impl Into<Atom<'a>>>,
        source: impl Into<Atom<'a>>,
    ) -> ModuleDeclaration<'a> {
        let specifiers: Vec<_> = names
            .into_iter()
            .map(|name| {
                let atom = name.into();
                let imported_name = self.ast.identifier_name(SPAN, atom);
                let local_binding = self.ast.binding_identifier(SPAN, atom);
                let specifier = self.ast.import_specifier(
                    SPAN,
                    ModuleExportName::IdentifierName(imported_name),
                    local_binding,
                    ImportOrExportKind::Value,
                );
                ImportDeclarationSpecifier::ImportSpecifier(self.ast.alloc(specifier))
            })
            .collect();

        let specifiers_vec = self.ast.vec_from_iter(specifiers);
        let source_literal = self.ast.string_literal(SPAN, source, None);
        self.ast.module_declaration_import_declaration(
            SPAN,
            Some(specifiers_vec),
            source_literal,
            None, // phase
            NONE, // with_clause
            ImportOrExportKind::Value,
        )
    }

    /// Create default export: `export default expr;`
    pub fn export_default(&self, expr: Expression<'a>) -> ModuleDeclaration<'a> {
        // ExportDefaultDeclarationKind inherits Expression variants, so we can convert
        let kind: ExportDefaultDeclarationKind = expr.into();
        self.ast
            .module_declaration_export_default_declaration(SPAN, kind)
    }

    /// Create named export of variable: `export const name = init;`
    pub fn export_const(
        &self,
        name: impl Into<Atom<'a>>,
        init: Expression<'a>,
    ) -> ModuleDeclaration<'a> {
        let decl = self.const_decl(name, init);
        let declaration = match decl {
            Statement::VariableDeclaration(var_decl) => Declaration::VariableDeclaration(var_decl),
            _ => unreachable!(),
        };
        ModuleDeclaration::ExportNamedDeclaration(self.ast.alloc(ExportNamedDeclaration {
            span: SPAN,
            declaration: Some(declaration),
            specifiers: self.ast.vec(),
            source: None,
            export_kind: ImportOrExportKind::Value,
            with_clause: None,
        }))
    }

    // ===== PROGRAM-LEVEL OPERATIONS =====

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

        writer.write_all(result.code.as_bytes()).map_err(|e| {
            crate::error::GenError::CodegenFailed {
                context: "Write error".to_string(),
                reason: Some(e.to_string()),
            }
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

    // ===== HELPER METHODS =====

    /// Unary not operator: `!expr`
    pub fn not(&self, expr: Expression<'a>) -> Expression<'a> {
        self.ast
            .expression_unary(SPAN, UnaryOperator::LogicalNot, expr)
    }

    /// Binary expression: `left op right`
    pub fn binary(
        &self,
        left: Expression<'a>,
        op: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        self.ast.expression_binary(SPAN, left, op, right)
    }

    /// Logical expression: `left && right` or `left || right`
    pub fn logical(
        &self,
        left: Expression<'a>,
        op: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        self.ast.expression_logical(SPAN, left, op, right)
    }

    /// Conditional/ternary expression: `test ? consequent : alternate`
    pub fn conditional(
        &self,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a> {
        self.ast
            .expression_conditional(SPAN, test, consequent, alternate)
    }

    /// New expression: `new Ctor(args)`
    pub fn new_expr(&self, callee: Expression<'a>, args: Vec<Argument<'a>>) -> Expression<'a> {
        let args_vec = self.ast.vec_from_iter(args);
        self.ast.expression_new(SPAN, callee, NONE, args_vec)
    }

    /// Template literal: backtick string with expressions
    /// For simple strings without interpolation, use `string()` instead
    ///
    /// # Example
    /// ```ignore
    /// // Generates: `Hello, ${name}!`
    /// js.template_literal(
    ///     vec!["Hello, ", "!"],
    ///     vec![js.ident("name")],
    /// )
    /// ```
    pub fn template_literal(
        &self,
        parts: Vec<impl Into<Atom<'a>>>,
        expressions: Vec<Expression<'a>>,
    ) -> Expression<'a> {
        // In `a${x}b${y}c`, parts = ["a", "b", "c"], expressions = [x, y]
        // The last quasi (index == expressions.len()) has tail = true
        let quasis: Vec<_> = parts
            .into_iter()
            .enumerate()
            .map(|(i, part)| {
                let atom = part.into();
                let tail = i == expressions.len(); // Last quasi element
                let value = TemplateElementValue {
                    raw: atom,
                    cooked: Some(atom),
                };
                TemplateElement {
                    span: SPAN,
                    value,
                    tail,
                    lone_surrogates: false,
                }
            })
            .collect();

        let quasis_vec = self.ast.vec_from_iter(quasis);
        let expressions_vec = self.ast.vec_from_iter(expressions);

        self.ast
            .expression_template_literal(SPAN, quasis_vec, expressions_vec)
    }

    /// Spread element for arrays or function calls: `...expr`
    ///
    /// # Example
    /// ```ignore
    /// // In array: [...items]
    /// js.array(vec![js.spread(js.ident("items"))])
    /// ```
    pub fn spread(&self, expr: Expression<'a>) -> SpreadElement<'a> {
        SpreadElement {
            span: SPAN,
            argument: expr,
        }
    }
}
