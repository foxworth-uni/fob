use rolldown_plugin::{
    HookResolveIdArgs, HookResolveIdReturn, HookUsage,
    HookTransformArgs, HookTransformReturn, Plugin, PluginContext,
    TransformPluginContext,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ModuleDeclaration,
};
use fob_gen::{parse, ParseOptions, QueryBuilder};

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
    pub specifiers: Vec<ImportSpecifier>,
    pub is_dynamic: bool,
    /// Resolved path to the target module (relative to cwd, same format as module IDs).
    /// None for external dependencies or unresolved imports.
    pub resolved_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ImportSpecifier {
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

/// Shared state for collecting module information during bundling
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

    pub fn mark_entry(&mut self, id: String) {
        if !self.entry_points.contains(&id) {
            self.entry_points.push(id);
        }
    }
}

/// Plugin that collects module information during the bundling process
#[derive(Debug)]
pub struct ModuleCollectionPlugin {
    state: Arc<Mutex<CollectionState>>,
}

impl ModuleCollectionPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CollectionState::new())),
        }
    }

    pub fn state(&self) -> Arc<Mutex<CollectionState>> {
        Arc::clone(&self.state)
    }

    pub fn take_data(&self) -> CollectionState {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut *state)
    }
}

impl Plugin for ModuleCollectionPlugin {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "module-collection-plugin".into()
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId | HookUsage::Transform
    }

    fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs,
    ) -> impl std::future::Future<Output = HookResolveIdReturn> + Send {
        let state = Arc::clone(&self.state);
        let specifier = args.specifier.to_string();
        let is_entry = args.importer.is_none();

        async move {
            // Track module resolution - we'll collect full info in load/transform
            if is_entry {
                let mut state = state.lock().unwrap();
                state.mark_entry(specifier);
            }

            // Let Rolldown handle the actual resolution
            Ok(None)
        }
    }

    fn transform(
        &self,
        _ctx: Arc<TransformPluginContext>,
        args: &HookTransformArgs,
    ) -> impl std::future::Future<Output = HookTransformReturn> + Send {
        let state = Arc::clone(&self.state);
        let code = args.code.to_string();
        let id = args.id.to_string();

        async move {
            // Parse the module to extract imports/exports
            let (imports, exports, has_side_effects) = parse_module_structure(&code);

            let is_entry = {
                let state = state.lock().unwrap();
                state.entry_points.contains(&id)
            };

            let module = CollectedModule {
                id: id.clone(),
                code: Some(code),
                is_entry,
                is_external: false, // External modules won't go through transform
                imports,
                exports,
                has_side_effects,
            };

            let mut state = state.lock().unwrap();
            state.add_module(id, module);

            // Don't modify the code
            Ok(None)
        }
    }
}

impl Default for ModuleCollectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a JavaScript/TypeScript module to extract its import/export structure
///
/// Uses fob-gen's parser for consistent parsing and better error handling.
/// For now, we assume modules have side effects by default since the bundler
/// will handle the proper analysis.
pub fn parse_module_structure(
    code: &str,
) -> (Vec<CollectedImport>, Vec<CollectedExport>, bool) {
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
        Err(_) => {
            // If parsing fails, assume the module has side effects
            return (vec![], vec![], true);
        }
    };

    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let has_side_effects = true; // Conservative default

    // Use QueryBuilder to extract imports and exports
    let query = QueryBuilder::new(&allocator, parsed.ast());
    
    // Extract imports
    let import_query = query.find_imports(None);
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
                                    specifiers.push(ImportSpecifier::Default {
                                        local: default_spec.local.name.to_string(),
                                    });
                                }
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns_spec) => {
                                    specifiers.push(ImportSpecifier::Namespace {
                                        local: ns_spec.local.name.to_string(),
                                    });
                                }
                                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(named_spec) => {
                                    let imported = match &named_spec.imported {
                                        oxc_ast::ast::ModuleExportName::IdentifierName(ident) => ident.name.to_string(),
                                        oxc_ast::ast::ModuleExportName::IdentifierReference(ident) => ident.name.to_string(),
                                        oxc_ast::ast::ModuleExportName::StringLiteral(lit) => lit.value.to_string(),
                                    };
                                    specifiers.push(ImportSpecifier::Named {
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

    (imports, exports, has_side_effects)
}
