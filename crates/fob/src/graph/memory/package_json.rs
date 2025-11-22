//! Package.json analysis methods for ModuleGraph.

use rustc_hash::FxHashSet as HashSet;

use super::super::package_json::{
    extract_package_name, DependencyCoverage, DependencyType, PackageJson, TypeCoverage,
    UnusedDependency,
};
use super::graph::ModuleGraph;
use crate::Result;

impl ModuleGraph {
    /// Detect unused npm dependencies by cross-referencing package.json with imports.
    ///
    /// This identifies dependencies declared in package.json but never actually imported
    /// in the codebase. Useful for cleaning up unused packages.
    ///
    /// # Parameters
    ///
    /// - `package_json`: Parsed package.json file
    /// - `include_dev`: Whether to check devDependencies
    /// - `include_peer`: Whether to check peerDependencies
    pub fn unused_npm_dependencies(
        &self,
        package_json: &PackageJson,
        include_dev: bool,
        include_peer: bool,
    ) -> Result<Vec<UnusedDependency>> {
        let inner = self.inner.read();

        // Collect all imported package names
        let mut imported_packages = HashSet::default();
        for module in inner.modules.values() {
            for import in module.imports.iter() {
                if import.is_external() {
                    let package_name = extract_package_name(&import.source);
                    imported_packages.insert(package_name.to_string());
                }
            }
        }

        let mut unused = Vec::new();

        // Check each dependency type
        let dep_types = [
            (DependencyType::Production, true),
            (DependencyType::Development, include_dev),
            (DependencyType::Peer, include_peer),
            (DependencyType::Optional, true),
        ];

        for (dep_type, should_check) in dep_types {
            if !should_check {
                continue;
            }

            for (package, version) in package_json.get_dependencies(dep_type) {
                if !imported_packages.contains(package) {
                    unused.push(UnusedDependency {
                        package: package.clone(),
                        version: version.clone(),
                        dep_type,
                    });
                }
            }
        }

        Ok(unused)
    }

    /// Get dependency coverage statistics.
    ///
    /// Provides detailed metrics about which dependencies are actually used
    /// vs declared in package.json.
    pub fn dependency_coverage(&self, package_json: &PackageJson) -> Result<DependencyCoverage> {
        let inner = self.inner.read();

        // Collect all imported package names
        let mut imported_packages = HashSet::default();
        for module in inner.modules.values() {
            for import in module.imports.iter() {
                if import.is_external() {
                    let package_name = extract_package_name(&import.source);
                    imported_packages.insert(package_name.to_string());
                }
            }
        }

        let mut by_type = std::collections::HashMap::new();
        let mut total_declared = 0;
        let mut total_used = 0;

        for dep_type in [
            DependencyType::Production,
            DependencyType::Development,
            DependencyType::Peer,
            DependencyType::Optional,
        ] {
            let deps = package_json.get_dependencies(dep_type);
            let declared = deps.len();
            let used = deps
                .keys()
                .filter(|pkg| imported_packages.contains(*pkg))
                .count();
            let unused = declared - used;

            total_declared += declared;
            total_used += used;

            by_type.insert(
                dep_type,
                TypeCoverage {
                    declared,
                    used,
                    unused,
                },
            );
        }

        Ok(DependencyCoverage {
            total_declared,
            total_used,
            total_unused: total_declared - total_used,
            by_type,
        })
    }
}
