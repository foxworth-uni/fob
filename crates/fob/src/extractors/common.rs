//! Common types and traits for framework extractors.
//!
//! This module defines the unified interface for extracting JavaScript/TypeScript
//! from framework-specific file formats (Astro, Svelte, Vue).

/// Represents JavaScript/TypeScript source code extracted from a framework file.
///
/// This is a unified type that replaces framework-specific `JavaScriptSource` types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedScript<'a> {
    /// The extracted JavaScript/TypeScript source code
    pub source_text: &'a str,

    /// Byte offset from the start of the original file
    ///
    /// This offset points to the beginning of the script content.
    /// Used to translate parse errors back to the original file location.
    pub source_offset: usize,

    /// Context information about this script block
    pub context: ScriptContext,

    /// Language identifier (js, ts, jsx, tsx)
    pub lang: &'a str,
}

impl<'a> ExtractedScript<'a> {
    /// Creates a new extracted script with the given parameters.
    pub fn new(
        source_text: &'a str,
        source_offset: usize,
        context: ScriptContext,
        lang: &'a str,
    ) -> Self {
        Self {
            source_text,
            source_offset,
            context,
            lang,
        }
    }
}

/// Context information about an extracted script block.
///
/// Different frameworks use different contexts to distinguish script blocks:
/// - Astro: Frontmatter vs script tags
/// - Svelte: Module context vs component instance
/// - Vue: Setup script vs regular script
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptContext {
    /// Astro frontmatter block (runs on server during build/SSR)
    AstroFrontmatter,
    /// Astro script tag (runs in browser)
    AstroScript,
    /// Svelte module context block (runs once when module is imported)
    SvelteModule,
    /// Svelte component instance script (runs for each component instance)
    SvelteComponent,
    /// Vue setup script (Composition API with compile-time sugar)
    VueSetup,
    /// Vue regular script (Options API or general setup)
    VueRegular,
}

/// Unified error type for all extractors.
#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    /// File exceeds maximum allowed size
    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge {
        /// Actual file size in bytes
        size: usize,
        /// Maximum allowed size in bytes
        max: usize,
    },

    /// Too many script tags found in the file
    #[error("Too many script tags: {count} found (max: {max} allowed)")]
    TooManyScriptTags {
        /// Number of script tags found
        count: usize,
        /// Maximum allowed script tags
        max: usize,
    },

    /// Script tag opened but never closed
    #[error("Unclosed script tag starting at byte position {position}")]
    UnclosedScriptTag {
        /// Byte position where the unclosed tag begins
        position: usize,
    },

    /// Frontmatter opened but never closed (Astro-specific)
    #[error("Unclosed frontmatter starting at byte position {position}")]
    UnclosedFrontmatter {
        /// Byte position where the unclosed frontmatter begins
        position: usize,
    },

    /// File contains invalid UTF-8
    #[error("Invalid UTF-8 encoding in file")]
    InvalidUtf8,

    /// I/O error reading the file
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Trait for framework-specific extractors.
///
/// Each framework (Astro, Svelte, Vue) implements this trait to provide
/// script extraction capabilities.
pub trait Extractor {
    /// Extract JavaScript/TypeScript scripts from the given source code.
    ///
    /// # Arguments
    ///
    /// * `source` - The complete file content
    ///
    /// # Returns
    ///
    /// A vector of extracted scripts, or an error if extraction fails.
    fn extract<'a>(&self, source: &'a str) -> Result<Vec<ExtractedScript<'a>>, ExtractorError>;

    /// Get the file extension this extractor handles (e.g., ".astro", ".svelte", ".vue").
    fn file_extension(&self) -> &'static str;
}

/// Maximum file size in bytes (10 MB)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of script tags to process
pub const MAX_SCRIPT_TAGS: usize = 100;

