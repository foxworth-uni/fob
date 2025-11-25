//! Dependency chain analysis for understanding module relationships.
//!
//! This module provides tools to trace how modules are connected through
//! import chains, useful for debugging why a module is included in a bundle
//! or understanding circular dependencies.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::ModuleId;

/// Maximum depth for dependency chain traversal to prevent DoS attacks and infinite loops.
///
/// This limit prevents path explosion in large graphs with deep dependency chains.
/// A value of 50 allows for very deep dependency graphs while still preventing
/// exponential growth that could cause performance issues or memory exhaustion.
const MAX_DEPENDENCY_CHAIN_DEPTH: usize = 50;

/// A chain of dependencies from an entry point to a target module.
///
/// Represents one path through the dependency graph, useful for understanding
/// why a particular module is included in the bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyChain {
    /// The path of module IDs from entry to target
    pub path: Vec<ModuleId>,
    /// Depth of this chain (path length - 1)
    pub depth: usize,
}

impl DependencyChain {
    /// Create a new dependency chain from a path.
    pub fn new(path: Vec<ModuleId>) -> Self {
        let depth = if !path.is_empty() { path.len() - 1 } else { 0 };

        Self { path, depth }
    }

    /// Get the entry point (first module in the chain).
    pub fn entry_point(&self) -> Option<&ModuleId> {
        self.path.first()
    }

    /// Get the target (last module in the chain).
    pub fn target(&self) -> Option<&ModuleId> {
        self.path.last()
    }

    /// Check if this chain contains a cycle (same module appears twice).
    pub fn has_cycle(&self) -> bool {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for module in &self.path {
            if !seen.insert(module) {
                return true;
            }
        }
        false
    }

    /// Format the chain as a human-readable string.
    ///
    /// Example: "entry.js -> utils.js -> helper.js"
    pub fn format_chain(&self) -> String {
        self.path
            .iter()
            .map(|id| id.path_string())
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

/// Analysis of all dependency chains to a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAnalysis {
    /// Target module being analyzed
    pub target: ModuleId,
    /// All chains leading to this module
    pub chains: Vec<DependencyChain>,
    /// Shortest chain depth
    pub min_depth: Option<usize>,
    /// Longest chain depth
    pub max_depth: Option<usize>,
    /// Average chain depth
    pub avg_depth: f64,
    /// Number of unique entry points that reach this module
    pub entry_point_count: usize,
}

impl ChainAnalysis {
    /// Create chain analysis from a collection of chains.
    pub fn from_chains(target: ModuleId, chains: Vec<DependencyChain>) -> Self {
        let min_depth = chains.iter().map(|c| c.depth).min();
        let max_depth = chains.iter().map(|c| c.depth).max();
        let avg_depth = if chains.is_empty() {
            0.0
        } else {
            chains.iter().map(|c| c.depth).sum::<usize>() as f64 / chains.len() as f64
        };

        let mut entry_points = std::collections::HashSet::new();
        for chain in &chains {
            if let Some(entry) = chain.entry_point() {
                entry_points.insert(entry.clone());
            }
        }

        Self {
            target,
            chains,
            min_depth,
            max_depth,
            avg_depth,
            entry_point_count: entry_points.len(),
        }
    }

    /// Check if the module is reachable from any entry point.
    pub fn is_reachable(&self) -> bool {
        !self.chains.is_empty()
    }

    /// Get the shortest chain (if any).
    pub fn shortest_chain(&self) -> Option<&DependencyChain> {
        self.chains.iter().min_by_key(|c| c.depth)
    }

    /// Get all chains containing cycles.
    pub fn circular_chains(&self) -> Vec<&DependencyChain> {
        self.chains.iter().filter(|c| c.has_cycle()).collect()
    }
}

/// Find all dependency chains from entry points to a target module using BFS.
///
/// This is an internal helper used by ModuleGraph implementations.
pub(crate) fn find_chains<F>(
    entry_points: &[ModuleId],
    target: &ModuleId,
    mut get_dependencies: F,
) -> Vec<DependencyChain>
where
    F: FnMut(&ModuleId) -> Vec<ModuleId>,
{
    let mut chains = Vec::new();
    let mut queue: VecDeque<Vec<ModuleId>> = VecDeque::new();

    // Start BFS from each entry point
    for entry in entry_points {
        queue.push_back(vec![entry.clone()]);
    }

    // Track visited paths to avoid infinite loops in circular dependencies
    let mut visited_paths = std::collections::HashSet::new();

    while let Some(current_path) = queue.pop_front() {
        // Defensive check: skip empty paths (should never happen, but prevents panic)
        let current_module = match current_path.last() {
            Some(module) => module,
            None => continue, // Skip empty paths - this indicates a bug but we handle gracefully
        };

        // Create a path signature for cycle detection
        let path_sig = current_path
            .iter()
            .map(|id| id.path_string())
            .collect::<Vec<_>>()
            .join("->");

        if !visited_paths.insert(path_sig) {
            // We've seen this exact path before, skip to avoid infinite loops
            continue;
        }

        // If we've reached the target, record the chain
        // Continue exploring to find cycles (paths that include target multiple times)
        if current_module == target {
            chains.push(DependencyChain::new(current_path.clone()));
            // Don't return here - continue exploring to find cycles
        }

        // Limit path depth to prevent explosion in large graphs (DoS protection)
        if current_path.len() > MAX_DEPENDENCY_CHAIN_DEPTH {
            continue;
        }

        // Explore dependencies
        for dep in get_dependencies(current_module) {
            let mut new_path = current_path.clone();
            new_path.push(dep);
            queue.push_back(new_path);
        }
    }

    chains
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_module_id(name: &str) -> ModuleId {
        ModuleId::new_virtual(format!("virtual:{}", name))
    }

    #[test]
    fn test_empty_dependency_path_edge_case() {
        // Test that find_chains handles empty paths gracefully
        // This should never happen in practice, but we handle it defensively
        let entry = mock_module_id("entry");
        let target = mock_module_id("target");

        // Create a get_dependencies function that returns empty vec
        let get_dependencies = |_id: &ModuleId| -> Vec<ModuleId> { vec![] };

        // Should not panic even with edge cases
        let chains = find_chains(&[entry], &target, get_dependencies);

        // With no dependencies, we should only find the entry point if it matches target
        // Since entry != target, chains should be empty
        assert_eq!(chains.len(), 0);
    }

    #[test]
    fn test_dependency_chain_creation() {
        let path = vec![
            mock_module_id("entry"),
            mock_module_id("utils"),
            mock_module_id("target"),
        ];

        let chain = DependencyChain::new(path.clone());

        assert_eq!(chain.depth, 2);
        assert_eq!(chain.entry_point(), Some(&path[0]));
        assert_eq!(chain.target(), Some(&path[2]));
        assert!(!chain.has_cycle());
    }

    #[test]
    fn test_dependency_chain_cycle_detection() {
        let a = mock_module_id("a");
        let b = mock_module_id("b");

        // Chain with cycle: a -> b -> a
        let path = vec![a.clone(), b.clone(), a.clone()];
        let chain = DependencyChain::new(path);

        assert!(chain.has_cycle());
    }

    #[test]
    fn test_dependency_chain_no_cycle() {
        let path = vec![
            mock_module_id("a"),
            mock_module_id("b"),
            mock_module_id("c"),
        ];
        let chain = DependencyChain::new(path);

        assert!(!chain.has_cycle());
    }

    #[test]
    fn test_chain_analysis() {
        let target = mock_module_id("target");

        let chains = vec![
            DependencyChain::new(vec![mock_module_id("entry1"), target.clone()]),
            DependencyChain::new(vec![
                mock_module_id("entry2"),
                mock_module_id("middle"),
                target.clone(),
            ]),
        ];

        let analysis = ChainAnalysis::from_chains(target.clone(), chains);

        assert!(analysis.is_reachable());
        assert_eq!(analysis.min_depth, Some(1));
        assert_eq!(analysis.max_depth, Some(2));
        assert_eq!(analysis.avg_depth, 1.5);
        assert_eq!(analysis.entry_point_count, 2);
    }

    #[test]
    fn test_chain_analysis_unreachable() {
        let target = mock_module_id("unreachable");
        let analysis = ChainAnalysis::from_chains(target, vec![]);

        assert!(!analysis.is_reachable());
        assert_eq!(analysis.min_depth, None);
        assert_eq!(analysis.max_depth, None);
        assert_eq!(analysis.avg_depth, 0.0);
        assert_eq!(analysis.entry_point_count, 0);
    }

    #[test]
    fn test_find_chains_basic() {
        // Graph: entry -> a -> target
        let entry = mock_module_id("entry");
        let a = mock_module_id("a");
        let target = mock_module_id("target");

        let get_deps = |module: &ModuleId| -> Vec<ModuleId> {
            if module == &entry {
                vec![a.clone()]
            } else if module == &a {
                vec![target.clone()]
            } else {
                vec![]
            }
        };

        let chains = find_chains(std::slice::from_ref(&entry), &target, get_deps);

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].path.len(), 3);
        assert_eq!(chains[0].depth, 2);
    }

    #[test]
    fn test_find_chains_multiple_paths() {
        // Graph:
        //   entry -> a -> target
        //   entry -> b -> target
        let entry = mock_module_id("entry");
        let a = mock_module_id("a");
        let b = mock_module_id("b");
        let target = mock_module_id("target");

        let get_deps = |module: &ModuleId| -> Vec<ModuleId> {
            if module == &entry {
                vec![a.clone(), b.clone()]
            } else if module == &a || module == &b {
                vec![target.clone()]
            } else {
                vec![]
            }
        };

        let chains = find_chains(std::slice::from_ref(&entry), &target, get_deps);

        assert_eq!(chains.len(), 2);
        assert!(chains.iter().all(|c| c.path.len() == 3));
    }

    #[test]
    fn test_format_chain() {
        let path = vec![
            mock_module_id("entry"),
            mock_module_id("utils"),
            mock_module_id("target"),
        ];

        let chain = DependencyChain::new(path);
        let formatted = chain.format_chain();

        assert!(formatted.contains("entry"));
        assert!(formatted.contains("utils"));
        assert!(formatted.contains("target"));
        assert!(formatted.contains("->"));
    }
}
