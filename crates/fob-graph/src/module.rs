use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::symbol::SymbolTable;
use super::{Export, Import, ModuleId};

/// Resolved module metadata used by graph algorithms and builders.
///
/// Heavy collections (imports, exports, symbol_table) are wrapped in Arc
/// to make cloning cheap when returning modules from the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: ModuleId,
    pub path: PathBuf,
    pub source_type: SourceType,
    #[serde(with = "arc_vec_serde")]
    pub imports: Arc<Vec<Import>>,
    #[serde(with = "arc_vec_serde")]
    pub exports: Arc<Vec<Export>>,
    pub has_side_effects: bool,
    pub is_entry: bool,
    pub is_external: bool,
    pub original_size: usize,
    pub bundled_size: Option<usize>,
    /// Symbol table from semantic analysis (intra-file dead code detection)
    #[serde(with = "arc_symbol_table_serde")]
    pub symbol_table: Arc<SymbolTable>,
    /// Module format (ESM vs CJS) from rolldown analysis
    pub module_format: ModuleFormat,
    /// Export structure kind (ESM, CJS, or None)
    pub exports_kind: ExportsKind,
    /// True if module has star re-exports (`export * from`)
    pub has_star_exports: bool,
    /// Execution order in module graph (topological sort)
    pub execution_order: Option<u32>,
}

// Serde helper for Arc<Vec<T>>
mod arc_vec_serde {
    use super::*;
    use serde::de::Deserializer;
    use serde::ser::Serializer;

    pub fn serialize<S, T>(value: &Arc<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        value.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Arc<Vec<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Vec::deserialize(deserializer).map(Arc::new)
    }
}

// Serde helper for Arc<SymbolTable>
mod arc_symbol_table_serde {
    use super::*;
    use serde::de::Deserializer;
    use serde::ser::Serializer;

    pub fn serialize<S>(value: &Arc<SymbolTable>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<SymbolTable>, D::Error>
    where
        D: Deserializer<'de>,
    {
        SymbolTable::deserialize(deserializer).map(Arc::new)
    }
}

impl Module {
    /// Create a new module builder with sensible defaults.
    pub fn builder(id: ModuleId, path: PathBuf, source_type: SourceType) -> ModuleBuilder {
        ModuleBuilder {
            module: Self {
                id,
                path,
                source_type,
                imports: Arc::new(Vec::new()),
                exports: Arc::new(Vec::new()),
                has_side_effects: false,
                is_entry: false,
                is_external: false,
                original_size: 0,
                bundled_size: None,
                symbol_table: Arc::new(SymbolTable::new()),
                module_format: ModuleFormat::Unknown,
                exports_kind: ExportsKind::None,
                has_star_exports: false,
                execution_order: None,
            },
        }
    }

    /// Mark the module as an entry module.
    pub fn mark_entry(&mut self) {
        self.is_entry = true;
    }

    /// Mark the module as an external dependency.
    pub fn mark_external(&mut self) {
        self.is_external = true;
    }

    /// Toggle side-effect tracking on the module.
    pub fn set_side_effects(&mut self, has_side_effects: bool) {
        self.has_side_effects = has_side_effects;
    }

    /// Update bundled size information (if available).
    pub fn set_bundled_size(&mut self, size: Option<usize>) {
        self.bundled_size = size;
    }

    /// Get an iterator over imports.
    pub fn imports_iter(&self) -> impl Iterator<Item = &Import> {
        self.imports.iter()
    }

    /// Get an iterator over exports.
    pub fn exports_iter(&self) -> impl Iterator<Item = &Export> {
        self.exports.iter()
    }

    /// Get mutable access to exports (for external tools like framework rules).
    ///
    /// This uses Arc::make_mut to create a mutable copy only when needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use fob_graph::Module;
    /// # use fob_graph::{ModuleId, SourceType};
    /// # use std::path::PathBuf;
    /// # let mut module = Module::builder(ModuleId::new_virtual("test.ts"), PathBuf::from("test.ts"), SourceType::TypeScript).build();
    /// for export in module.exports_mut() {
    ///     if export.name.starts_with("use") {
    ///         // export.mark_framework_used(); // Method might not exist on Export, check usage
    ///     }
    /// }
    /// ```
    pub fn exports_mut(&mut self) -> &mut Vec<Export> {
        Arc::make_mut(&mut self.exports)
    }

    /// Get imports that reference a specific module.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_graph::ModuleId;
    /// # use fob_graph::Module;
    /// # use fob_graph::SourceType;
    /// # use std::path::PathBuf;
    ///
    /// # let module = Module::builder(ModuleId::new_virtual("test.ts"), PathBuf::from("test.ts"), SourceType::TypeScript).build();
    /// let react_id = ModuleId::new("node_modules/react/index.js")?;
    /// let imports = module.imports_from(&react_id);
    /// assert_eq!(imports.len(), 0);
    /// # Ok::<(), fob_graph::ModuleIdError>(())
    /// ```
    pub fn imports_from(&self, target: &ModuleId) -> Vec<&Import> {
        self.imports
            .iter()
            .filter(|imp| imp.resolved_to.as_ref() == Some(target))
            .collect()
    }

    /// Check if this module imports from a specific source specifier.
    ///
    /// This is useful for framework detection - checking if a module imports
    /// from "react", "vue", "svelte", etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use fob_graph::Module;
    /// # use fob_graph::{ModuleId, SourceType};
    /// # use std::path::PathBuf;
    /// # let module = Module::builder(ModuleId::new_virtual("test.ts"), PathBuf::from("test.ts"), SourceType::TypeScript).build();
    /// if module.has_import_from("react") {
    ///     // This is a React module
    /// }
    /// ```
    pub fn has_import_from(&self, source: &str) -> bool {
        self.imports.iter().any(|imp| imp.source == source)
    }

    /// Get all import sources (for dependency analysis).
    ///
    /// Returns a vector of source specifiers that this module imports from.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use fob_graph::Module;
    /// # use fob_graph::{ModuleId, SourceType};
    /// # use std::path::PathBuf;
    /// # let module = Module::builder(ModuleId::new_virtual("test.ts"), PathBuf::from("test.ts"), SourceType::TypeScript).build();
    /// let sources = module.import_sources();
    /// // sources = ["react", "lodash", "./utils"]
    /// ```
    pub fn import_sources(&self) -> Vec<&str> {
        self.imports.iter().map(|imp| imp.source.as_str()).collect()
    }
}

/// Builder for `Module` to avoid long argument lists in constructors.
pub struct ModuleBuilder {
    module: Module,
}

impl ModuleBuilder {
    pub fn imports(mut self, imports: Vec<Import>) -> Self {
        self.module.imports = Arc::new(imports);
        self
    }

    pub fn exports(mut self, exports: Vec<Export>) -> Self {
        self.module.exports = Arc::new(exports);
        self
    }

    pub fn side_effects(mut self, has_side_effects: bool) -> Self {
        self.module.has_side_effects = has_side_effects;
        self
    }

    pub fn entry(mut self, is_entry: bool) -> Self {
        self.module.is_entry = is_entry;
        self
    }

    pub fn external(mut self, is_external: bool) -> Self {
        self.module.is_external = is_external;
        self
    }

    pub fn original_size(mut self, original_size: usize) -> Self {
        self.module.original_size = original_size;
        self
    }

    pub fn bundled_size(mut self, bundled_size: Option<usize>) -> Self {
        self.module.bundled_size = bundled_size;
        self
    }

    pub fn symbol_table(mut self, symbol_table: SymbolTable) -> Self {
        self.module.symbol_table = Arc::new(symbol_table);
        self
    }

    pub fn module_format(mut self, module_format: ModuleFormat) -> Self {
        self.module.module_format = module_format;
        self
    }

    pub fn exports_kind(mut self, exports_kind: ExportsKind) -> Self {
        self.module.exports_kind = exports_kind;
        self
    }

    pub fn has_star_exports(mut self, has_star_exports: bool) -> Self {
        self.module.has_star_exports = has_star_exports;
        self
    }

    pub fn execution_order(mut self, execution_order: Option<u32>) -> Self {
        self.module.execution_order = execution_order;
        self
    }

    pub fn build(self) -> Module {
        self.module
    }
}

/// Module definition format (ESM vs CJS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleFormat {
    /// ECMAScript Module (.mjs or "type": "module")
    EsmMjs,
    /// ECMAScript Module (package.json "type": "module")
    EsmPackageJson,
    /// ECMAScript Module (regular .js with ESM syntax)
    Esm,
    /// CommonJS (package.json "type": "commonjs")
    CjsPackageJson,
    /// CommonJS (regular require/module.exports)
    Cjs,
    /// Unknown format
    Unknown,
}

/// Export structure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportsKind {
    /// Module uses ESM exports
    Esm,
    /// Module uses CommonJS exports
    CommonJs,
    /// No exports detected
    None,
}

/// Resolved module source type derived from file extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    JavaScript,
    TypeScript,
    Jsx,
    Tsx,
    Json,
    Css,
    Unknown,
}

impl SourceType {
    /// Derive the source type from a file extension string.
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "jsx" => Self::Jsx,
            "tsx" => Self::Tsx,
            "json" => Self::Json,
            "css" => Self::Css,
            _ => Self::Unknown,
        }
    }

    /// Attempt to infer the source type from a file path.
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map_or(Self::Unknown, Self::from_extension)
    }

    /// Returns true if the file is JavaScript/TypeScript based.
    pub fn is_javascript_like(&self) -> bool {
        matches!(
            self,
            Self::JavaScript | Self::TypeScript | Self::Jsx | Self::Tsx
        )
    }
}
