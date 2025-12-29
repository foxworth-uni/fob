use rustc_hash::FxHashMap;

/// Lightweight import map representation for component bundles.
#[derive(Debug, Clone, Default)]
pub struct ImportMap {
    entries: FxHashMap<String, String>,
}

impl ImportMap {
    pub fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, specifier: impl Into<String>, path: impl Into<String>) {
        self.entries.insert(specifier.into(), path.into());
    }

    pub fn as_map(&self) -> &FxHashMap<String, String> {
        &self.entries
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.entries).unwrap_or_else(|_| "{}".to_string())
    }

    /// Inject import map into HTML using fob-gen's HtmlBuilder
    ///
    /// This delegates to HtmlBuilder for consistent HTML generation.
    pub fn inject_html(&self, html: &str) -> String {
        use fob_gen::{Allocator, HtmlBuilder};

        let allocator = Allocator::default();
        let html_builder = HtmlBuilder::new(allocator);
        html_builder.inject_import_map(html, &self.to_json())
    }
}
