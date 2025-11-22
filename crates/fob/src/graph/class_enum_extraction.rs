//! Class and enum member extraction from AST.
//!
//! This module provides functionality for extracting detailed information about
//! class members (properties, methods, getters, setters) and enum members from
//! JavaScript/TypeScript AST nodes.

// Use re-exported OXC types from the fob crate for version consistency
use crate::oxc::{
    ast::{
        Class, ClassElement, Expression, MethodDefinition, MethodDefinitionKind,
        PropertyDefinition, PropertyKey, TSAccessibility, TSEnumDeclaration, TSEnumMemberName,
    },
    Visit,
};
use oxc_ast_visit::walk;

use super::symbol::{
    ClassMemberMetadata, EnumMemberMetadata, EnumMemberValue, Symbol, SymbolKind, SymbolMetadata,
    SymbolSpan, SymbolTable, Visibility,
};

/// Extract class member symbols from an AST program
///
/// This uses an AST visitor to find all class declarations and extract
/// their members (properties, methods, etc.) with visibility and metadata.
pub fn extract_class_and_enum_members(
    program: &oxc_ast::ast::Program,
    source_text: &str,
    table: &mut SymbolTable,
) {
    let mut visitor = ClassEnumExtractor {
        source_text,
        table,
        current_class_name: None,
    };

    visitor.visit_program(program);
}

/// AST visitor for extracting class and enum members
struct ClassEnumExtractor<'a, 'table> {
    source_text: &'a str,
    table: &'table mut SymbolTable,
    current_class_name: Option<String>,
}

impl<'a, 'table> ClassEnumExtractor<'a, 'table> {
    /// Extract members from a class declaration
    fn extract_class_members(&mut self, class: &Class, class_name: &str) {
        for element in &class.body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    self.extract_class_property(prop, class_name);
                }
                ClassElement::MethodDefinition(method) => {
                    self.extract_class_method(method, class_name);
                }
                ClassElement::AccessorProperty(accessor) => {
                    // For accessor properties, extract them as properties with special flag
                    if let Some(key) = accessor.key.static_name() {
                        let visibility = determine_visibility_from_key(&accessor.key, None);
                        let (line, column) = get_line_column(self.source_text, accessor.span.start);
                        let span = SymbolSpan::new(line, column, accessor.span.start);

                        let mut metadata = ClassMemberMetadata::new(
                            visibility,
                            accessor.r#static,
                            class_name.to_string(),
                        );
                        metadata.is_accessor = true;

                        let symbol = Symbol::with_metadata(
                            key.to_string(),
                            SymbolKind::ClassProperty,
                            span,
                            0, // scope_id - not critical for class members
                            SymbolMetadata::ClassMember(metadata),
                        );

                        self.table.add_symbol(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    /// Extract a class property
    fn extract_class_property(&mut self, prop: &PropertyDefinition, class_name: &str) {
        if let Some(name) = extract_property_key_name(&prop.key) {
            let visibility = determine_visibility_from_key(&prop.key, prop.accessibility);
            let (line, column) = get_line_column(self.source_text, prop.span.start);
            let span = SymbolSpan::new(line, column, prop.span.start);

            let mut metadata =
                ClassMemberMetadata::new(visibility, prop.r#static, class_name.to_string());
            metadata.is_readonly = prop.readonly;

            let symbol = Symbol::with_metadata(
                name,
                SymbolKind::ClassProperty,
                span,
                0, // scope_id
                SymbolMetadata::ClassMember(metadata),
            );

            self.table.add_symbol(symbol);
        }
    }

    /// Extract a class method
    fn extract_class_method(&mut self, method: &MethodDefinition, class_name: &str) {
        if let Some(name) = extract_property_key_name(&method.key) {
            let visibility = determine_visibility_from_key(&method.key, method.accessibility);
            let (line, column) = get_line_column(self.source_text, method.span.start);
            let span = SymbolSpan::new(line, column, method.span.start);

            let kind = match method.kind {
                MethodDefinitionKind::Constructor => SymbolKind::ClassConstructor,
                MethodDefinitionKind::Get => SymbolKind::ClassGetter,
                MethodDefinitionKind::Set => SymbolKind::ClassSetter,
                MethodDefinitionKind::Method => SymbolKind::ClassMethod,
            };

            let mut metadata =
                ClassMemberMetadata::new(visibility, method.r#static, class_name.to_string());
            metadata.is_accessor =
                matches!(kind, SymbolKind::ClassGetter | SymbolKind::ClassSetter);

            let symbol = Symbol::with_metadata(
                name,
                kind,
                span,
                0, // scope_id
                SymbolMetadata::ClassMember(metadata),
            );

            self.table.add_symbol(symbol);
        }
    }

    /// Extract enum members from a TypeScript enum declaration
    fn extract_enum_members(&mut self, enum_decl: &TSEnumDeclaration) {
        let enum_name = enum_decl.id.name.to_string();

        for member in &enum_decl.body.members {
            let member_name = match &member.id {
                TSEnumMemberName::Identifier(ident) => Some(ident.name.to_string()),
                TSEnumMemberName::String(lit) => Some(lit.value.to_string()),
                _ => None, // Skip computed property names
            };

            if let Some(member_name) = member_name {
                let value = member.initializer.as_ref().map(|init| match init {
                    Expression::NumericLiteral(lit) => EnumMemberValue::Number(lit.value as i64),
                    Expression::StringLiteral(lit) => {
                        EnumMemberValue::String(lit.value.to_string())
                    }
                    _ => EnumMemberValue::Computed,
                });

                let (line, column) = get_line_column(self.source_text, member.span.start);
                let span = SymbolSpan::new(line, column, member.span.start);

                let metadata = EnumMemberMetadata {
                    enum_name: enum_name.clone(),
                    value,
                };

                let symbol = Symbol::with_metadata(
                    member_name,
                    SymbolKind::EnumMember,
                    span,
                    0, // scope_id
                    SymbolMetadata::EnumMember(metadata),
                );

                self.table.add_symbol(symbol);
            }
        }
    }
}

impl<'a, 'table, 'ast> Visit<'ast> for ClassEnumExtractor<'a, 'table> {
    fn visit_class(&mut self, class: &Class<'ast>) {
        // Get the class name from the id if it exists
        if let Some(id) = &class.id {
            let class_name = id.name.to_string();
            self.current_class_name = Some(class_name.clone());
            self.extract_class_members(class, &class_name);
        }

        // Continue visiting child nodes
        walk::walk_class(self, class);
        self.current_class_name = None;
    }

    fn visit_ts_enum_declaration(&mut self, enum_decl: &TSEnumDeclaration<'ast>) {
        self.extract_enum_members(enum_decl);
        walk::walk_ts_enum_declaration(self, enum_decl);
    }
}

/// Determine visibility from property key and TypeScript accessibility
fn determine_visibility_from_key(
    key: &PropertyKey,
    accessibility: Option<TSAccessibility>,
) -> Visibility {
    // JavaScript private fields start with #
    if matches!(key, PropertyKey::PrivateIdentifier(_)) {
        return Visibility::Private;
    }

    // TypeScript explicit accessibility
    match accessibility {
        Some(TSAccessibility::Private) => Visibility::Private,
        Some(TSAccessibility::Protected) => Visibility::Protected,
        Some(TSAccessibility::Public) | None => Visibility::Public,
    }
}

/// Extract the name from a property key
fn extract_property_key_name(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
        PropertyKey::PrivateIdentifier(ident) => Some(format!("#{}", ident.name)),
        PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
        PropertyKey::NumericLiteral(lit) => Some(lit.value.to_string()),
        // Computed property keys - use placeholder
        _ => Some("[computed]".to_string()),
    }
}

/// Calculate line and column from byte offset in source text.
///
/// Returns (line, column) where line is 1-indexed and column is 0-indexed.
fn get_line_column(source: &str, offset: u32) -> (u32, u32) {
    let offset = offset as usize;
    if offset > source.len() {
        return (0, 0);
    }

    let mut line = 1u32;
    let mut column = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }

        current_offset += ch.len_utf8();
    }

    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::{Parser, ParserReturn};
    use oxc_span::SourceType;

    #[test]
    fn test_extract_class_members() {
        let source = r#"
            class Example {
                public publicField: string;
                private privateField: number;
                #privateJsField = 42;

                constructor() {}

                private privateMethod() {}
                public publicMethod() {}

                get value() { return 1; }
                set value(v) {}
            }
        "#;

        let allocator = Allocator::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source, SourceType::tsx()).parse();

        let mut table = SymbolTable::new();
        extract_class_and_enum_members(&program, source, &mut table);

        // Should find multiple class members
        let class_members: Vec<_> = table
            .symbols
            .iter()
            .filter(|s| matches!(s.metadata, SymbolMetadata::ClassMember(_)))
            .collect();

        assert!(!class_members.is_empty(), "Should find class members");

        // Check for private members
        let private_members: Vec<_> = class_members
            .iter()
            .filter(|s| {
                if let SymbolMetadata::ClassMember(meta) = &s.metadata {
                    matches!(meta.visibility, Visibility::Private)
                } else {
                    false
                }
            })
            .collect();

        assert!(!private_members.is_empty(), "Should find private members");
    }

    #[test]
    fn test_extract_enum_members() {
        let source = r#"
            enum Status {
                Active = 1,
                Inactive = 2,
                Pending
            }
        "#;

        let allocator = Allocator::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source, SourceType::tsx()).parse();

        let mut table = SymbolTable::new();
        extract_class_and_enum_members(&program, source, &mut table);

        let enum_members: Vec<_> = table
            .symbols
            .iter()
            .filter(|s| matches!(s.kind, SymbolKind::EnumMember))
            .collect();

        assert_eq!(enum_members.len(), 3, "Should find 3 enum members");

        // Check that they all belong to "Status" enum
        for member in &enum_members {
            assert_eq!(member.enum_name(), Some("Status"));
        }
    }
}
