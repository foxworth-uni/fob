//! Shared collection types for module graph analysis.
//!
//! These types serve as an intermediate representation between source code
//! and the final `ModuleGraph`. They are populated by:
//!
//! 1. **Bundler mode**: `ModuleCollectionPlugin` during Rolldown traversal
//! 2. **Analysis mode**: Direct parsing via `parse_module_structure()`
//!
//! The `Collected*` types retain more information than their final `Module`
//! counterparts (e.g., local bindings) to enable flexible graph construction.
//!
//! # Security Note
//!
//! `parse_module_structure()` returns errors for malformed code rather than
//! silently accepting invalid syntax. Callers should handle parse errors
//! appropriately for their use case.

use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during module collection
#[derive(Debug, Error)]
pub enum CollectionError {
    /// Failed to parse module code
    #[error("Failed to parse module: {0}")]
    ParseError(String),

    /// Module not found in collection
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
}

/// Represents a collected module with all its metadata
#[derive(Debug, Clone)]
pub struct CollectedModule {
    pub id: String,
    pub code: Option<String>,
    pub is_entry: bool,
    pub is_external: bool,
    pub imports: Vec<CollectedImport>,
    pub exports: Vec<CollectedExport>,
    pub has_side_effects: bool,
}

/// Represents an import statement in a module
#[derive(Debug, Clone)]
pub struct CollectedImport {
    pub source: String,
    pub specifiers: Vec<CollectedImportSpecifier>,
    pub is_dynamic: bool,
    /// Resolved path to the target module (relative to cwd, same format as module IDs).
    /// None for external dependencies or unresolved imports.
    pub resolved_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CollectedImportSpecifier {
    Named { imported: String, local: String },
    Default { local: String },
    Namespace { local: String },
}

/// Represents an export declaration in a module
#[derive(Debug, Clone)]
pub enum CollectedExport {
    Named { exported: String, local: Option<String> },
    Default,
    All { source: String },
}

/// Shared state for collecting module information during bundling or analysis
#[derive(Debug, Default)]
pub struct CollectionState {
    pub modules: HashMap<String, CollectedModule>,
    pub entry_points: Vec<String>,
}

impl CollectionState {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            entry_points: Vec::new(),
        }
    }

    pub fn add_module(&mut self, id: String, module: CollectedModule) {
        self.modules.insert(id, module);
    }

    pub fn get_module(&self, id: &str) -> Option<&CollectedModule> {
        self.modules.get(id)
    }

    /// Mark a module as an entry point
    ///
    /// Note: This method allows marking modules as entry points before they are added
    /// to the collection, which is useful during initial setup. Entry points should be
    /// validated after collection is complete.
    pub fn mark_entry(&mut self, id: String) {
        if !self.entry_points.contains(&id) {
            self.entry_points.push(id);
        }
    }

    /// Validate that all entry points exist in the module collection
    ///
    /// # Errors
    ///
    /// Returns `CollectionError::ModuleNotFound` for any entry point that doesn't have
    /// a corresponding module in the collection.
    pub fn validate_entry_points(&self) -> Result<(), CollectionError> {
        for entry in &self.entry_points {
            if !self.modules.contains_key(entry) {
                return Err(CollectionError::ModuleNotFound(entry.clone()));
            }
        }
        Ok(())
    }
}

/// Parse a JavaScript/TypeScript module to extract its import/export structure
///
/// Uses fob-gen's parser for consistent parsing and better error handling.
///
/// # Returns
///
/// Returns a tuple of (imports, exports, has_side_effects) where:
/// - `imports`: List of import statements found in the module
/// - `exports`: List of export declarations found in the module
/// - `has_side_effects`: Conservative default of `true` (assumes side effects)
///
/// # Errors
///
/// Returns `CollectionError::ParseError` if the code contains syntax errors.
pub fn parse_module_structure(
    code: &str,
) -> Result<(Vec<CollectedImport>, Vec<CollectedExport>, bool), CollectionError> {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Declaration, ModuleDeclaration};
    use fob_gen::{parse, ParseOptions, QueryBuilder};

    let allocator = Allocator::default();

    // Infer source type from code patterns - use ParseOptions helpers
    let parse_opts = if code.contains("import ") || code.contains("export ") {
        if code.contains(": ") || code.contains("interface ") {
            ParseOptions::tsx() // TypeScript with JSX
        } else {
            ParseOptions::jsx() // JavaScript with JSX
        }
    } else {
        ParseOptions::default() // Plain script
    };

    // Use fob-gen's parse function
    let parsed = match parse(&allocator, code, parse_opts) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(CollectionError::ParseError(e.to_string()));
        }
    };

    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let has_side_effects = true; // Conservative default

    // Use QueryBuilder to extract imports and exports
    let query = QueryBuilder::new(&allocator, parsed.ast());
    
    // Extract imports
    let _import_query = query.find_imports(None);
    // Note: QueryBuilder doesn't expose the actual ImportDeclaration nodes yet,
    // so we still need to walk the AST manually, but we use the parsed program
    for stmt in parsed.ast().body.iter() {
        if let Some(module_decl) = stmt.as_module_declaration() {
            match module_decl {
                ModuleDeclaration::ImportDeclaration(import) => {
                    let mut specifiers = Vec::new();
                    if let Some(specs) = &import.specifiers {
                        for spec in specs {
                            match spec {
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(default_spec) => {
                                    specifiers.push(CollectedImportSpecifier::Default {
                                        local: default_spec.local.name.to_string(),
                                    });
                                }
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns_spec) => {
                                    specifiers.push(CollectedImportSpecifier::Namespace {
                                        local: ns_spec.local.name.to_string(),
                                    });
                                }
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(named_spec) => {
                                    let imported = match &named_spec.imported {
                                        oxc_ast::ast::ModuleExportName::IdentifierName(ident) => ident.name.to_string(),
                                        oxc_ast::ast::ModuleExportName::IdentifierReference(ident) => ident.name.to_string(),
                                        oxc_ast::ast::ModuleExportName::StringLiteral(lit) => lit.value.to_string(),
                                    };
                                    specifiers.push(CollectedImportSpecifier::Named {
                                        imported,
                                        local: named_spec.local.name.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    imports.push(CollectedImport {
                        source: import.source.value.to_string(),
                        specifiers,
                        is_dynamic: false,
                        resolved_path: None, // Will be populated during graph walking
                    });
                }
                ModuleDeclaration::ExportDefaultDeclaration(_) => {
                    exports.push(CollectedExport::Default);
                }
                ModuleDeclaration::ExportNamedDeclaration(named) => {
                    if let Some(src) = &named.source {
                        // Re-export
                        exports.push(CollectedExport::All {
                            source: src.value.to_string(),
                        });
                    } else if let Some(decl) = &named.declaration {
                        // Export declaration
                        match decl {
                            Declaration::FunctionDeclaration(func) => {
                                if let Some(id) = &func.id {
                                    exports.push(CollectedExport::Named {
                                        exported: id.name.to_string(),
                                        local: Some(id.name.to_string()),
                                    });
                                }
                            }
                            Declaration::VariableDeclaration(var) => {
                                for decl in &var.declarations {
                                    if let oxc_ast::ast::BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                                        exports.push(CollectedExport::Named {
                                            exported: ident.name.to_string(),
                                            local: Some(ident.name.to_string()),
                                        });
                                    }
                                }
                            }
                            Declaration::ClassDeclaration(class) => {
                                if let Some(id) = &class.id {
                                    exports.push(CollectedExport::Named {
                                        exported: id.name.to_string(),
                                        local: Some(id.name.to_string()),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                ModuleDeclaration::ExportAllDeclaration(all) => {
                    exports.push(CollectedExport::All {
                        source: all.source.value.to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    Ok((imports, exports, has_side_effects))
}

