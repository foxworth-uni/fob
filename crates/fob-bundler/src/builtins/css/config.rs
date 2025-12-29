//! CSS processing configuration types

/// Configuration options for CSS processing
///
/// Controls how lightningcss processes CSS files including
/// bundling, minification, and browser target compatibility.
#[derive(Debug, Clone)]
pub struct CssPluginOptions {
    /// Enable CSS minification
    ///
    /// When enabled, CSS will be minified using lightningcss:
    /// - Merge longhand properties into shorthands
    /// - Remove unnecessary whitespace
    /// - Merge duplicate rules
    /// - Optimize calc() expressions
    pub minify: bool,

    /// Browser targets for vendor prefixing
    ///
    /// Accepts browserslist-style queries, e.g.:
    /// - `vec![">0.2%", "not dead"]`
    /// - `vec!["last 2 versions"]`
    /// - `vec!["chrome >= 90"]`
    ///
    /// When specified, lightningcss will automatically add
    /// vendor prefixes required for the target browsers.
    pub targets: Option<Vec<String>>,

    /// Enable source map generation
    ///
    /// When enabled, source maps will be generated for
    /// minified/transformed CSS to aid debugging.
    pub source_map: bool,

    /// Patterns to exclude from processing
    ///
    /// Glob patterns for CSS files to skip.
    /// Example: `vec!["**/*.min.css", "**/vendor/**"]`
    pub exclude: Vec<String>,

    /// Patterns to include for processing
    ///
    /// Glob patterns for CSS files to process.
    /// If empty, all `.css` files are processed.
    /// Example: `vec!["src/**/*.css"]`
    pub include: Vec<String>,
}

impl Default for CssPluginOptions {
    fn default() -> Self {
        Self {
            minify: false,
            targets: None,
            source_map: false,
            exclude: vec!["**/*.min.css".to_string()],
            include: Vec::new(),
        }
    }
}

impl CssPluginOptions {
    /// Create new options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable minification
    pub fn with_minify(mut self, enabled: bool) -> Self {
        self.minify = enabled;
        self
    }

    /// Set browser targets
    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.targets = Some(targets);
        self
    }

    /// Enable source maps
    pub fn with_source_maps(mut self, enabled: bool) -> Self {
        self.source_map = enabled;
        self
    }

    /// Add exclusion pattern
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }

    /// Add inclusion pattern
    pub fn include(mut self, pattern: impl Into<String>) -> Self {
        self.include.push(pattern.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = CssPluginOptions::default();
        assert_eq!(opts.minify, false);
        assert!(opts.targets.is_none());
        assert_eq!(opts.source_map, false);
        assert!(!opts.exclude.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let opts = CssPluginOptions::new()
            .with_minify(true)
            .with_targets(vec![">0.2%".to_string()])
            .with_source_maps(true);

        assert!(opts.minify);
        assert!(opts.targets.is_some());
        assert!(opts.source_map);
    }

    #[test]
    fn test_exclusion_patterns() {
        let opts = CssPluginOptions::new()
            .exclude("vendor/**")
            .exclude("node_modules/**");

        assert!(opts.exclude.iter().any(|p| p.contains("vendor")));
    }
}
