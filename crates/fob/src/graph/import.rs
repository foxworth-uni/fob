use serde::{Deserialize, Serialize};

use super::{ModuleId, SourceSpan};

/// Individual import binding from a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportSpecifier {
    /// `import { foo } from 'mod'`
    Named(String),
    /// `import foo from 'mod'`
    Default,
    /// `import * as foo from 'mod'`
    Namespace(String),
}

/// Mechanism used to load the dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportKind {
    /// Static `import` declaration.
    Static,
    /// Dynamic `import()` expression.
    Dynamic,
    /// CommonJS `require()` call.
    Require,
    /// TypeScript `import type` declaration removed at runtime.
    TypeOnly,
    /// `export { foo } from 'mod'` style re-export.
    ReExport,
}

impl ImportKind {
    /// Returns `true` for imports that execute at runtime.
    pub fn is_runtime(&self) -> bool {
        !matches!(self, Self::TypeOnly)
    }

    /// Returns `true` for static, eagerly-resolved imports.
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static | Self::Require | Self::ReExport)
    }
}

/// Complete analysis of a dependency edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub source: String,
    pub specifiers: Vec<ImportSpecifier>,
    pub kind: ImportKind,
    pub resolved_to: Option<ModuleId>,
    pub span: SourceSpan,
}

impl Import {
    /// Convenience constructor for building imports in tests/fixtures.
    pub fn new(
        source: impl Into<String>,
        specifiers: Vec<ImportSpecifier>,
        kind: ImportKind,
        resolved_to: Option<ModuleId>,
        span: SourceSpan,
    ) -> Self {
        Self {
            source: source.into(),
            specifiers,
            kind,
            resolved_to,
            span,
        }
    }

    /// Returns `true` for side-effect-only imports (`import 'polyfill'`).
    pub fn is_side_effect_only(&self) -> bool {
        self.specifiers.is_empty() && self.kind.is_runtime()
    }

    /// Returns `true` when the import references a package on npm (no relative prefix).
    pub fn is_external(&self) -> bool {
        !self.source.is_empty()
            && !self.source.starts_with('.')
            && !self.source.starts_with('/')
            && !self.source.starts_with('\\')
            && !self.source.starts_with("virtual:")
    }

    /// Returns `true` if the import only contributes types.
    pub fn is_type_only(&self) -> bool {
        matches!(self.kind, ImportKind::TypeOnly)
    }

    /// Check if this is a namespace import (`import * as foo`).
    pub fn is_namespace_import(&self) -> bool {
        self.specifiers
            .iter()
            .any(|spec| matches!(spec, ImportSpecifier::Namespace(_)))
    }

    /// Get the namespace binding name if this is a namespace import.
    ///
    /// Returns `Some(name)` for `import * as name from 'mod'`, otherwise `None`.
    pub fn namespace_name(&self) -> Option<&str> {
        self.specifiers.iter().find_map(|spec| {
            if let ImportSpecifier::Namespace(name) = spec {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    /// Check if this import contributes to runtime execution.
    ///
    /// This returns `true` for:
    /// - Side-effect imports (`import 'polyfill'`)
    /// - Regular runtime imports
    ///
    /// Returns `false` for type-only imports.
    pub fn has_runtime_effect(&self) -> bool {
        self.kind.is_runtime()
    }
}
