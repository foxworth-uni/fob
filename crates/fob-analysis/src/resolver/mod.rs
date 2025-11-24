//! Module resolution for standalone analysis.
//!
//! Implements Node.js-style module resolution algorithm without requiring
//! the full bundler infrastructure.

mod algorithm;
mod aliases;
mod extensions;

pub use algorithm::{is_external, resolve_local, resolve_with_alias};
pub use aliases::resolve_path_alias;
pub use extensions::{resolve_with_extensions, EXTENSIONS};

use std::path::{Path, PathBuf};

use fob::runtime::{Runtime, RuntimeError};

use crate::config::{AnalyzerConfig, ResolveResult};

/// Module resolver for standalone analysis.
pub struct ModuleResolver {
    config: AnalyzerConfig,
}

impl ModuleResolver {
    /// Create a new module resolver with the given configuration.
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
        if algorithm::is_external(specifier, &self.config.external) {
            return Ok(ResolveResult::External(specifier.to_string()));
        }

        // Check path aliases first
        let cwd = self.get_cwd(runtime)?;
        if let Some(resolved) = algorithm::resolve_with_alias(
            specifier,
            from,
            cwd.as_path(),
            &self.config.path_aliases,
            runtime,
        )
        .await?
        {
            return Ok(resolved);
        }

        // Try direct resolution
        if specifier.starts_with('.') || specifier.starts_with('/') {
            // Relative or absolute path
            return algorithm::resolve_local(specifier, from, self.config.cwd.as_deref(), runtime)
                .await;
        }

        // Must be an external package (bare import)
        Ok(ResolveResult::External(specifier.to_string()))
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
    use fob::runtime::RuntimeError;
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

        async fn metadata(
            &self,
            path: &Path,
        ) -> Result<fob::runtime::FileMetadata, RuntimeError> {
            if self.files.iter().any(|f| f == path) {
                Ok(fob::runtime::FileMetadata {
                    is_file: true,
                    is_dir: false,
                    size: 0,
                    modified: None,
                })
            } else {
                Err(RuntimeError::FileNotFound(path.to_path_buf()))
            }
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

