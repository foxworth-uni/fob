//! Module resolution for standalone analysis.
//!
//! Implements Node.js-style module resolution algorithm without requiring
//! the full bundler infrastructure.

use std::path::{Path, PathBuf};

use path_clean::PathClean;
use crate::runtime::{Runtime, RuntimeError};
use super::types::{AnalyzerConfig, ResolveResult};

/// Module resolver for standalone analysis.
pub struct ModuleResolver {
    config: AnalyzerConfig,
}

impl ModuleResolver {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    /// Resolve a module specifier from a given file.
    ///
    /// Implements Node.js resolution algorithm:
    /// 1. Check if it's a relative/absolute path
    /// 2. Try path aliases
    /// 3. Try extensions (.ts, .tsx, .js, .jsx)
    /// 4. Try index files
    /// 5. Check if it's external (npm package)
    pub async fn resolve(
        &self,
        specifier: &str,
        from: &Path,
        runtime: &dyn Runtime,
    ) -> Result<ResolveResult, RuntimeError> {
        // Check if it's explicitly external
        if self.is_external(specifier) {
            return Ok(ResolveResult::External(specifier.to_string()));
        }

        // Check path aliases first
        if let Some(resolved) = self.resolve_path_alias(specifier, from) {
            // Path aliases are resolved relative to cwd
            let cwd = self.get_cwd(runtime)?;
            let base = if resolved.starts_with('/') {
                Path::new("")
            } else {
                cwd.as_path()
            };
            let candidate = base.join(&resolved).clean();
            
            // Try with extensions
            let extensions = ["ts", "tsx", "js", "jsx", "mjs", "json"];
            
            // First, try the path as-is
            if runtime.exists(&candidate) {
                if let Ok(metadata) = runtime.metadata(&candidate).await {
                    if metadata.is_file {
                        return Ok(ResolveResult::Local(candidate));
                    }
                }
            }
            
            // Try with each extension
            for ext in &extensions {
                let with_ext = candidate.with_extension(ext);
                if runtime.exists(&with_ext) {
                    if let Ok(metadata) = runtime.metadata(&with_ext).await {
                        if metadata.is_file {
                            return Ok(ResolveResult::Local(with_ext));
                        }
                    }
                }
            }
            
            // Try as directory with index files
            if runtime.exists(&candidate) {
                if let Ok(metadata) = runtime.metadata(&candidate).await {
                    if metadata.is_dir {
                        for ext in &extensions {
                            let index = candidate.join(format!("index.{}", ext));
                            if runtime.exists(&index) {
                                return Ok(ResolveResult::Local(index));
                            }
                        }
                    }
                }
            }
        }

        // Try direct resolution
        if specifier.starts_with('.') || specifier.starts_with('/') {
            // Relative or absolute path
            return self.resolve_local(specifier, from, runtime).await;
        }

        // Must be an external package (bare import)
        Ok(ResolveResult::External(specifier.to_string()))
    }

    /// Check if a specifier is explicitly marked as external.
    fn is_external(&self, specifier: &str) -> bool {
        // Check exact match
        if self.config.external.iter().any(|ext| ext == specifier) {
            return true;
        }

        // Check if specifier starts with any external prefix
        // e.g., "react" matches external "react"
        // e.g., "react-dom" matches external "react"
        for ext in &self.config.external {
            if specifier == ext || specifier.starts_with(&format!("{ext}/")) {
                return true;
            }
        }

        false
    }

    /// Resolve a path alias (e.g., "@/components" → "./src/components").
    fn resolve_path_alias(&self, specifier: &str, _from: &Path) -> Option<String> {
        for (alias, target) in &self.config.path_aliases {
            if specifier.starts_with(alias) {
                let rest = &specifier[alias.len()..];
                // Remove leading slash if present
                let rest = rest.strip_prefix('/').unwrap_or(rest);
                
                // Build resolved path - ensure it starts with ./ for relative resolution
                let resolved = if target.starts_with('/') {
                    // Absolute path - shouldn't happen but handle it
                    format!("{target}/{rest}")
                } else if target.starts_with('.') {
                    // Already relative
                    if rest.is_empty() {
                        target.clone()
                    } else {
                        format!("{target}/{rest}")
                    }
                } else {
                    // Make it relative
                    if rest.is_empty() {
                        format!("./{target}")
                    } else {
                        format!("./{target}/{rest}")
                    }
                };
                
                return Some(resolved);
            }
        }
        None
    }

    /// Resolve a local file path (relative or absolute).
    async fn resolve_local(
        &self,
        specifier: &str,
        from: &Path,
        runtime: &dyn Runtime,
    ) -> Result<ResolveResult, RuntimeError> {
        let base = if specifier.starts_with('/') {
            // Absolute path - use cwd as base if available
            self.config.cwd.as_ref().map(|cwd| cwd.as_path())
                .unwrap_or_else(|| from.parent().unwrap_or(Path::new("")))
        } else {
            // Relative path
            from.parent().unwrap_or(Path::new(""))
        };

        // Join and normalize the path to handle . and .. components
        let candidate = base.join(specifier).clean();
        
        // Try with extensions
        let extensions = ["ts", "tsx", "js", "jsx", "mjs", "json"];
        
        // First, try the path as-is (might already have extension)
        if runtime.exists(&candidate) {
            // Check if it's a file (not a directory)
            if let Ok(metadata) = runtime.metadata(&candidate).await {
                if metadata.is_file {
                    return Ok(ResolveResult::Local(candidate));
                }
            }
        }

        // Try with each extension
        for ext in &extensions {
            let with_ext = candidate.with_extension(ext);
            
            if runtime.exists(&with_ext) {
                if let Ok(metadata) = runtime.metadata(&with_ext).await {
                    if metadata.is_file {
                        return Ok(ResolveResult::Local(with_ext));
                    }
                }
            }
        }

        // Try as directory with index files
        if runtime.exists(&candidate) {
            if let Ok(metadata) = runtime.metadata(&candidate).await {
                if metadata.is_dir {
                    for ext in &extensions {
                        let index = candidate.join(format!("index.{}", ext));
                        if runtime.exists(&index) {
                            return Ok(ResolveResult::Local(index));
                        }
                    }
                }
            }
        }

        // Could not resolve
        Ok(ResolveResult::Unresolved(specifier.to_string()))
    }

    /// Get the current working directory, preferring config, then runtime.
    pub fn get_cwd(&self, runtime: &dyn Runtime) -> Result<PathBuf, RuntimeError> {
        if let Some(ref cwd) = self.config.cwd {
            Ok(cwd.clone())
        } else {
            runtime.get_cwd()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::RuntimeError;
    use std::path::PathBuf;

    // Mock runtime for testing
    #[derive(Debug)]
    struct MockRuntime {
        files: Vec<PathBuf>,
    }

    #[cfg(not(target_family = "wasm"))]
    #[async_trait::async_trait]
    impl Runtime for MockRuntime {
        fn exists(&self, path: &Path) -> bool {
            self.files.iter().any(|f| f == path)
        }

        async fn read_file(&self, _path: &Path) -> Result<Vec<u8>, RuntimeError> {
            Err(RuntimeError::FileNotFound(PathBuf::new()))
        }

        async fn write_file(&self, _path: &Path, _content: &[u8]) -> Result<(), RuntimeError> {
            Ok(())
        }

        async fn metadata(&self, _path: &Path) -> Result<crate::runtime::FileMetadata, RuntimeError> {
            Err(RuntimeError::FileNotFound(PathBuf::new()))
        }

        fn resolve(&self, _specifier: &str, _from: &Path) -> Result<PathBuf, RuntimeError> {
            Err(RuntimeError::FileNotFound(PathBuf::new()))
        }

        async fn create_dir(&self, _path: &Path, _recursive: bool) -> Result<(), RuntimeError> {
            Ok(())
        }

        async fn remove_file(&self, _path: &Path) -> Result<(), RuntimeError> {
            Ok(())
        }

        async fn read_dir(&self, _path: &Path) -> Result<Vec<String>, RuntimeError> {
            Ok(vec![])
        }

        fn get_cwd(&self) -> Result<PathBuf, RuntimeError> {
            Ok(PathBuf::from("/test"))
        }
    }

    #[tokio::test]
    async fn test_resolve_relative() {
        let mut config = AnalyzerConfig::default();
        config.cwd = Some(PathBuf::from("/test"));
        
        let runtime = MockRuntime {
            files: vec![PathBuf::from("/test/src/utils.ts")],
        };
        
        let resolver = ModuleResolver::new(config);
        let from = PathBuf::from("/test/src/index.ts");
        
        let result = resolver.resolve("./utils", &from, &runtime).await.unwrap();
        assert!(matches!(result, ResolveResult::Local(_)));
    }

    #[tokio::test]
    async fn test_resolve_external() {
        let mut config = AnalyzerConfig::default();
        config.external = vec!["react".to_string()];
        
        let runtime = MockRuntime { files: vec![] };
        let resolver = ModuleResolver::new(config);
        let from = PathBuf::from("/test/src/index.ts");
        
        let result = resolver.resolve("react", &from, &runtime).await.unwrap();
        assert!(matches!(result, ResolveResult::External(_)));
    }
}

