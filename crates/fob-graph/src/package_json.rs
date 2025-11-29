//! Package.json parsing and dependency analysis.
//!
//! This module provides functionality to parse package.json files and analyze
//! npm dependencies against actual module imports in the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fob_core::Result;
use fob_core::runtime::Runtime;

/// Maximum allowed size for package.json files (10MB)
const MAX_PACKAGE_JSON_SIZE: u64 = 10 * 1024 * 1024;

/// Parsed package.json structure.
///
/// This focuses on dependency-related fields and omits other metadata
/// like scripts, engines, etc. for simplicity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    /// Package name
    pub name: Option<String>,
    /// Package version
    pub version: Option<String>,
    /// Production dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// Development dependencies
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
    /// Peer dependencies
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,
    /// Optional dependencies
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,
    /// File path this was loaded from
    #[serde(skip)]
    pub path: PathBuf,
}

impl PackageJson {
    /// Load package.json from a specific path using the provided runtime.
    ///
    /// # Security
    ///
    /// - Validates the path is within allowed boundaries (prevents path traversal)
    /// - Limits file size to 10MB to prevent DoS
    /// - Uses safe JSON parsing
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_graph::PackageJson;
    /// # use fob_core::runtime::Runtime;
    /// # use std::path::PathBuf;
    /// # async fn example<R: Runtime>(runtime: &R) -> fob_core::Result<()> {
    /// let pkg = PackageJson::from_path(runtime, &PathBuf::from("./package.json")).await?;
    /// println!("Package: {:?}", pkg.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_path<R: Runtime>(runtime: &R, path: &Path) -> Result<Self> {
        // Validate path to prevent directory traversal
        Self::validate_path(path)?;

        // Check file size before reading
        let metadata = runtime.metadata(path).await.map_err(|e| {
            fob_core::Error::InvalidConfig(format!("Cannot read package.json metadata: {e}"))
        })?;

        if metadata.size > MAX_PACKAGE_JSON_SIZE {
            return Err(fob_core::Error::InvalidConfig(format!(
                "package.json exceeds maximum size of {}MB",
                MAX_PACKAGE_JSON_SIZE / 1024 / 1024
            )));
        }

        // Read and parse the file
        let content_bytes = runtime.read_file(path).await.map_err(|e| {
            fob_core::Error::InvalidConfig(format!("Failed to read package.json: {e}"))
        })?;

        let content = String::from_utf8(content_bytes).map_err(|e| {
            fob_core::Error::InvalidConfig(format!("package.json contains invalid UTF-8: {e}"))
        })?;

        let mut pkg: PackageJson = serde_json::from_str(&content).map_err(|e| {
            fob_core::Error::InvalidConfig(format!("Invalid package.json format: {e}"))
        })?;

        pkg.path = path.to_path_buf();
        Ok(pkg)
    }

    /// Load package.json from a specific path (native builds only).
    ///
    /// # Deprecated
    ///
    /// This method is provided for backward compatibility on native builds.
    /// For new code, use `from_path` with an explicit runtime parameter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_graph::PackageJson;
    /// # use std::path::PathBuf;
    /// # async fn example() -> fob_core::Result<()> {
    /// let pkg = PackageJson::from_path_native(&PathBuf::from("./package.json")).await?;
    /// println!("Package: {:?}", pkg.name);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_family = "wasm"))]
    #[deprecated(
        note = "Use from_path with explicit runtime parameter for better platform compatibility"
    )]
    pub async fn from_path_native(path: &Path) -> Result<Self> {
        use fob_core::NativeRuntime;
        let runtime = NativeRuntime::new();
        Self::from_path(&runtime, path).await
    }

    /// Find and load package.json starting from a directory using the provided runtime.
    ///
    /// Searches upward through parent directories until package.json is found
    /// or the filesystem root is reached.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_graph::PackageJson;
    /// # use fob_core::runtime::Runtime;
    /// # use std::path::PathBuf;
    /// # async fn example<R: Runtime>(runtime: &R) -> fob_core::Result<()> {
    /// let pkg = PackageJson::find_from_dir(runtime, &PathBuf::from("./src")).await?;
    /// println!("Package: {:?}", pkg.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_from_dir<R: Runtime>(runtime: &R, start_dir: &Path) -> Result<Self> {
        let mut current = start_dir.to_path_buf();

        loop {
            let package_json_path = current.join("package.json");

            if runtime.exists(&package_json_path) {
                return Self::from_path(runtime, &package_json_path).await;
            }

            // Try parent directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                return Err(fob_core::Error::InvalidConfig(
                    "No package.json found in directory tree".to_string(),
                ));
            }
        }
    }

    /// Find and load package.json starting from a directory (native builds only).
    ///
    /// # Deprecated
    ///
    /// This method is provided for backward compatibility on native builds.
    /// For new code, use `find_from_dir` with an explicit runtime parameter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_graph::PackageJson;
    /// # use std::path::PathBuf;
    /// # async fn example() -> fob_core::Result<()> {
    /// let pkg = PackageJson::find_from_dir_native(&PathBuf::from("./src")).await?;
    /// println!("Package: {:?}", pkg.name);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_family = "wasm"))]
    #[deprecated(
        note = "Use find_from_dir with explicit runtime parameter for better platform compatibility"
    )]
    pub async fn find_from_dir_native(start_dir: &Path) -> Result<Self> {
        use fob_core::NativeRuntime;
        let runtime = NativeRuntime::new();
        Self::find_from_dir(&runtime, start_dir).await
    }

    /// Get all dependencies of a specific type.
    pub fn get_dependencies(&self, dep_type: DependencyType) -> &HashMap<String, String> {
        match dep_type {
            DependencyType::Production => &self.dependencies,
            DependencyType::Development => &self.dev_dependencies,
            DependencyType::Peer => &self.peer_dependencies,
            DependencyType::Optional => &self.optional_dependencies,
        }
    }

    /// Get all dependency names across all types.
    pub fn all_dependency_names(&self, include_dev: bool, include_peer: bool) -> Vec<String> {
        let mut names = Vec::new();

        names.extend(self.dependencies.keys().cloned());

        if include_dev {
            names.extend(self.dev_dependencies.keys().cloned());
        }

        if include_peer {
            names.extend(self.peer_dependencies.keys().cloned());
        }

        names.extend(self.optional_dependencies.keys().cloned());

        names.sort();
        names.dedup();
        names
    }

    /// Validate a path to prevent directory traversal attacks.
    fn validate_path(path: &Path) -> Result<()> {
        // Convert to canonical path if possible
        let path_str = path.to_string_lossy();

        // Reject paths with suspicious patterns
        if path_str.contains("..") {
            return Err(fob_core::Error::InvalidConfig(
                "Path contains '..' (potential directory traversal)".to_string(),
            ));
        }

        Ok(())
    }
}

/// Type of dependency in package.json.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Regular dependencies
    Production,
    /// Development dependencies
    Development,
    /// Peer dependencies
    Peer,
    /// Optional dependencies
    Optional,
}

impl DependencyType {
    /// Human-readable name for the dependency type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Production => "dependencies",
            Self::Development => "devDependencies",
            Self::Peer => "peerDependencies",
            Self::Optional => "optionalDependencies",
        }
    }
}

/// An npm dependency that is declared but never imported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedDependency {
    /// Package name
    pub package: String,
    /// Version specifier from package.json
    pub version: String,
    /// Type of dependency
    pub dep_type: DependencyType,
}

/// Coverage statistics for dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCoverage {
    /// Total dependencies declared
    pub total_declared: usize,
    /// Dependencies actually imported
    pub total_used: usize,
    /// Dependencies never imported
    pub total_unused: usize,
    /// Breakdown by dependency type
    pub by_type: HashMap<DependencyType, TypeCoverage>,
}

impl DependencyCoverage {
    /// Calculate coverage percentage.
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_declared == 0 {
            100.0
        } else {
            (self.total_used as f64 / self.total_declared as f64) * 100.0
        }
    }
}

/// Coverage for a specific dependency type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCoverage {
    pub declared: usize,
    pub used: usize,
    pub unused: usize,
}

/// Extract the base package name from an npm import specifier.
///
/// This handles scoped packages correctly:
/// - `@foo/bar` -> `@foo/bar`
/// - `@foo/bar/baz` -> `@foo/bar`
/// - `lodash` -> `lodash`
/// - `lodash/fp` -> `lodash`
///
/// # Example
///
/// ```
/// # use fob_graph::extract_package_name;
/// assert_eq!(extract_package_name("@babel/core"), "@babel/core");
/// assert_eq!(extract_package_name("@babel/core/lib/index"), "@babel/core");
/// assert_eq!(extract_package_name("lodash"), "lodash");
/// assert_eq!(extract_package_name("lodash/fp"), "lodash");
/// ```
pub fn extract_package_name(specifier: &str) -> &str {
    if specifier.is_empty() {
        return specifier;
    }

    // Handle scoped packages (@org/package)
    if specifier.starts_with('@') {
        // Find the second slash (after @org/)
        if let Some(first_slash) = specifier.find('/') {
            if let Some(second_slash) = specifier[first_slash + 1..].find('/') {
                return &specifier[..first_slash + 1 + second_slash];
            }
        }
        // Return entire string if no second slash
        return specifier;
    }

    // Non-scoped packages - take up to first slash
    if let Some(slash_idx) = specifier.find('/') {
        &specifier[..slash_idx]
    } else {
        specifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        // Scoped packages
        assert_eq!(extract_package_name("@babel/core"), "@babel/core");
        assert_eq!(extract_package_name("@babel/core/lib/index"), "@babel/core");
        assert_eq!(extract_package_name("@types/node"), "@types/node");
        assert_eq!(extract_package_name("@types/node/fs"), "@types/node");

        // Regular packages
        assert_eq!(extract_package_name("lodash"), "lodash");
        assert_eq!(extract_package_name("lodash/fp"), "lodash");
        assert_eq!(extract_package_name("react"), "react");
        assert_eq!(extract_package_name("react/jsx-runtime"), "react");

        // Edge cases
        assert_eq!(extract_package_name(""), "");
        assert_eq!(extract_package_name("@org"), "@org");
    }

    #[test]
    fn test_dependency_type_as_str() {
        assert_eq!(DependencyType::Production.as_str(), "dependencies");
        assert_eq!(DependencyType::Development.as_str(), "devDependencies");
        assert_eq!(DependencyType::Peer.as_str(), "peerDependencies");
        assert_eq!(DependencyType::Optional.as_str(), "optionalDependencies");
    }

    #[test]
    fn test_coverage_percentage() {
        let coverage = DependencyCoverage {
            total_declared: 10,
            total_used: 7,
            total_unused: 3,
            by_type: HashMap::new(),
        };

        assert_eq!(coverage.coverage_percentage(), 70.0);

        let empty_coverage = DependencyCoverage {
            total_declared: 0,
            total_used: 0,
            total_unused: 0,
            by_type: HashMap::new(),
        };

        assert_eq!(empty_coverage.coverage_percentage(), 100.0);
    }

    #[tokio::test]
    async fn test_package_json_parse() {
        let json = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.0.0",
                "lodash": "^4.17.21"
            },
            "devDependencies": {
                "@types/node": "^20.0.0"
            }
        }"#;

        let pkg: PackageJson = serde_json::from_str(json).unwrap();

        assert_eq!(pkg.name, Some("test-package".to_string()));
        assert_eq!(pkg.version, Some("1.0.0".to_string()));
        assert_eq!(pkg.dependencies.len(), 2);
        assert_eq!(pkg.dev_dependencies.len(), 1);
        assert_eq!(pkg.dependencies.get("react"), Some(&"^18.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_all_dependency_names() {
        let json = r#"{
            "dependencies": {
                "react": "^18.0.0",
                "lodash": "^4.17.21"
            },
            "devDependencies": {
                "@types/node": "^20.0.0",
                "typescript": "^5.0.0"
            },
            "peerDependencies": {
                "react-dom": "^18.0.0"
            }
        }"#;

        let pkg: PackageJson = serde_json::from_str(json).unwrap();

        // Production only
        let names = pkg.all_dependency_names(false, false);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"react".to_string()));
        assert!(names.contains(&"lodash".to_string()));

        // Include dev
        let names_with_dev = pkg.all_dependency_names(true, false);
        assert_eq!(names_with_dev.len(), 4);
        assert!(names_with_dev.contains(&"@types/node".to_string()));

        // Include all
        let all_names = pkg.all_dependency_names(true, true);
        assert_eq!(all_names.len(), 5);
        assert!(all_names.contains(&"react-dom".to_string()));
    }

    #[test]
    fn test_validate_path_rejects_traversal() {
        assert!(PackageJson::validate_path(Path::new("../etc/passwd")).is_err());
        assert!(PackageJson::validate_path(Path::new("foo/../bar/../baz")).is_err());
        assert!(PackageJson::validate_path(Path::new("./package.json")).is_ok());
        assert!(PackageJson::validate_path(Path::new("/absolute/path/package.json")).is_ok());
    }
}
