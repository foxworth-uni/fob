//! Framework script extractors.
//!
//! This module provides extractors for extracting JavaScript/TypeScript from
//! framework-specific file formats (Astro, Svelte, Vue).
//!
//! # Usage
//!
//! ```rust,no_run
//! use fob::extractors::{extract_scripts, ExtractedScript};
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let path = Path::new("component.astro");
//! let content = std::fs::read_to_string(path)?;
//! let scripts = extract_scripts(path, &content)?;
//!
//! for script in scripts {
//!     println!("Found script: {}", script.source_text);
//! }
//! # Ok(()) }
//! ```

mod astro;
mod common;
mod svelte;
mod vue;

pub use astro::AstroExtractor;
pub use common::{ExtractedScript, Extractor, ExtractorError, ScriptContext};
pub use svelte::SvelteExtractor;
pub use vue::VueExtractor;

use std::path::Path;

/// Extract scripts from a framework file, auto-detecting the framework by extension.
///
/// # Arguments
///
/// * `path` - The file path (used to determine framework by extension)
/// * `content` - The file content
///
/// # Returns
///
/// A vector of extracted scripts, or an error if extraction fails.
///
/// # Supported Extensions
///
/// - `.astro` - Astro components
/// - `.svelte` - Svelte components
/// - `.vue` - Vue Single File Components
///
/// # Example
///
/// ```rust,no_run
/// use fob::extractors::extract_scripts;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let path = Path::new("component.astro");
/// let content = r#"---
/// const title = 'My Page'
/// ---
/// <script>
///   console.log(title)
/// </script>"#;
///
/// let scripts = extract_scripts(path, content)?;
/// assert_eq!(scripts.len(), 2); // Frontmatter + script tag
/// # Ok(()) }
/// ```
pub fn extract_scripts<'a>(
    path: &Path,
    content: &'a str,
) -> Result<Vec<ExtractedScript<'a>>, ExtractorError> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
        "astro" => AstroExtractor.extract(content),
        "svelte" => SvelteExtractor.extract(content),
        "vue" => VueExtractor.extract(content),
        _ => Ok(vec![]), // Not a framework file, return empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scripts_astro() {
        let path = Path::new("test.astro");
        let content = r#"---
const x = 1
---
<script>console.log(x)</script>"#;
        let scripts = extract_scripts(path, content).unwrap();
        assert_eq!(scripts.len(), 2);
    }

    #[test]
    fn test_extract_scripts_svelte() {
        let path = Path::new("test.svelte");
        let content = r#"<script>let x = 1</script>"#;
        let scripts = extract_scripts(path, content).unwrap();
        assert_eq!(scripts.len(), 1);
    }

    #[test]
    fn test_extract_scripts_vue() {
        let path = Path::new("test.vue");
        let content = r#"<script setup>const x = 1</script>"#;
        let scripts = extract_scripts(path, content).unwrap();
        assert_eq!(scripts.len(), 1);
    }

    #[test]
    fn test_extract_scripts_unknown_extension() {
        let path = Path::new("test.js");
        let content = "const x = 1";
        let scripts = extract_scripts(path, content).unwrap();
        assert_eq!(scripts.len(), 0);
    }
}
