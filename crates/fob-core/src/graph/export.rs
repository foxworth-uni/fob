use serde::{Deserialize, Serialize};

use super::SourceSpan;

/// Export declaration kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportKind {
    Named,
    Default,
    ReExport,
    /// Star re-export: `export * from './module'`
    ///
    /// This re-exports all named exports from the source module.
    /// Unlike `ReExport`, this doesn't specify individual export names.
    StarReExport,
    TypeOnly,
}

/// Complete metadata describing a module export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Export {
    pub name: String,
    pub kind: ExportKind,
    pub is_used: bool,
    pub is_type_only: bool,
    pub re_exported_from: Option<String>,
    pub is_framework_used: bool,
    /// True if this export came from a CommonJS module.
    ///
    /// Important for CJS/ESM interop detection.
    pub came_from_commonjs: bool,
    pub span: SourceSpan,
    /// Number of times this export is imported across the entire module graph.
    ///
    /// - `None` means usage count hasn't been computed yet
    /// - `Some(0)` means the export is confirmed unused
    /// - `Some(n)` where n > 0 means the export is used n times
    ///
    /// This is populated by `ModuleGraph::compute_export_usage_counts()`.
    pub usage_count: Option<usize>,
}

impl Export {
    /// Construct a new export metadata record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        kind: ExportKind,
        is_used: bool,
        is_type_only: bool,
        re_exported_from: Option<String>,
        is_framework_used: bool,
        came_from_commonjs: bool,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            is_used,
            is_type_only,
            re_exported_from,
            is_framework_used,
            came_from_commonjs,
            span,
            usage_count: None,
        }
    }

    /// Marks the export as used by another module.
    pub fn mark_used(&mut self) {
        self.is_used = true;
    }

    /// Marks the export as unused.
    pub fn mark_unused(&mut self) {
        self.is_used = false;
    }

    /// Marks the export as used by framework conventions (React hooks, etc.).
    pub fn mark_framework_used(&mut self) {
        self.is_framework_used = true;
        self.is_used = true;
    }

    /// Convenience check for default exports.
    pub fn is_default(&self) -> bool {
        matches!(self.kind, ExportKind::Default)
    }

    /// Returns true if the export re-exports from another module.
    pub fn is_re_export(&self) -> bool {
        matches!(self.kind, ExportKind::ReExport | ExportKind::StarReExport)
    }

    /// Returns true if this is a star re-export (`export * from './module'`).
    pub fn is_star_re_export(&self) -> bool {
        matches!(self.kind, ExportKind::StarReExport)
    }

    /// Returns true if this export is marked as used by framework conventions.
    ///
    /// Framework-used exports include React hooks, Next.js data fetching functions,
    /// Vue composables, etc. These exports appear unused in static analysis but are
    /// consumed by framework magic.
    pub fn is_framework_used(&self) -> bool {
        self.is_framework_used
    }

    /// Sets the usage count for this export.
    ///
    /// This should be called by `ModuleGraph::compute_export_usage_counts()`.
    pub fn set_usage_count(&mut self, count: usize) {
        self.usage_count = Some(count);
    }

    /// Increments the usage count by 1.
    ///
    /// If the count hasn't been initialized yet (is None), sets it to 1.
    pub fn increment_usage_count(&mut self) {
        self.usage_count = Some(self.usage_count.unwrap_or(0) + 1);
    }

    /// Returns the usage count for this export.
    ///
    /// - `None` means the count hasn't been computed yet
    /// - `Some(0)` means the export is confirmed unused
    /// - `Some(n)` where n > 0 means the export is used n times
    pub fn usage_count(&self) -> Option<usize> {
        self.usage_count
    }

    /// Resets the usage count to None.
    ///
    /// Used when the module graph is modified and counts need to be recomputed.
    pub fn reset_usage_count(&mut self) {
        self.usage_count = None;
    }
}
