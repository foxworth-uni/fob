use serde::{Deserialize, Serialize};

/// Top-level documentation artifact containing all modules processed in a run.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Documentation {
    /// Modules discovered during extraction.
    pub modules: Vec<ModuleDoc>,
}

impl Documentation {
    /// Returns `true` when no module contains any documented symbols.
    pub fn is_empty(&self) -> bool {
        self.modules.iter().all(|module| module.symbols.is_empty())
    }

    /// Adds a module to the documentation set.
    pub fn add_module(&mut self, module: ModuleDoc) {
        self.modules.push(module);
    }
}

/// Documentation summary for a single source module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDoc {
    /// File-system path (or virtual path) for the module.
    pub path: String,
    /// Optional free-form description derived from leading file comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Documented exported symbols.
    pub symbols: Vec<ExportedSymbol>,
}

impl ModuleDoc {
    /// Creates an empty module documentation instance.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            description: None,
            symbols: Vec::new(),
        }
    }
}

/// Documentation for an exported symbol (function, class, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSymbol {
    /// Exported identifier name.
    pub name: String,
    /// Symbol kind.
    pub kind: SymbolKind,
    /// Primary description (free-form summary).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Parameter documentation.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<ParameterDoc>,
    /// Description of the return value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    /// Deprecated message, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// Example snippets extracted from JSDoc.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<String>,
    /// Raw tags that were not mapped to dedicated fields.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<JsDocTag>,
    /// Source location for the symbol definition.
    pub location: SourceLocation,
}

impl ExportedSymbol {
    /// Creates a new exported symbol documentation object.
    pub fn new(name: impl Into<String>, kind: SymbolKind, location: SourceLocation) -> Self {
        Self {
            name: name.into(),
            kind,
            summary: None,
            parameters: Vec::new(),
            returns: None,
            deprecated: None,
            examples: Vec::new(),
            tags: Vec::new(),
            location,
        }
    }
}

/// Parameter documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDoc {
    /// Parameter name.
    pub name: String,
    /// Optional type hint extracted from JSDoc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_hint: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ParameterDoc {
    /// Creates a new parameter documentation record.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_hint: None,
            description: None,
        }
    }
}

/// Structured representation of an arbitrary JSDoc tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsDocTag {
    /// Tag identifier (e.g. `example`, `see`).
    pub tag: String,
    /// Optional identifier associated with the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional type hint captured from the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_hint: Option<String>,
    /// Arbitrary textual description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl JsDocTag {
    /// Creates a new JSDoc tag record.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            name: None,
            type_hint: None,
            description: None,
        }
    }
}

/// Enumerates exported symbol kinds supported by the documentation model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    Variable,
    DefaultExport,
    Other,
}

impl Default for SymbolKind {
    fn default() -> Self {
        Self::Other
    }
}

/// Lightweight source position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SourceLocation {
    /// One-based line index.
    pub line: u32,
    /// One-based column index.
    pub column: u32,
}

impl SourceLocation {
    /// Creates a location from one-based line/column.
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}
