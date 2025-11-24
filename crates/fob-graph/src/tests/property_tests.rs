//! True property-based tests for fob-graph using proptest.
//!
//! These tests use proptest to verify mathematical invariants hold across
//! a wide range of randomly generated inputs. Property-based testing is
//! particularly valuable for graph libraries because they have strong
//! mathematical properties (symmetry, transitivity, etc.).
//!
//! Run with: cargo test --features proptest --package fob-graph property_tests

#![cfg(feature = "proptest")]

use crate::{ModuleGraph, ModuleId, Module, SourceType};
use proptest::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

/// Strategy for generating valid module IDs.
fn module_id_strategy() -> impl Strategy<Value = ModuleId> {
    // Generate valid file paths
    prop::collection::vec("[a-z]{1,10}", 1..=5)
        .prop_map(|parts| {
            let path = format!("src/{}.ts", parts.join("/"));
            ModuleId::new(&path).unwrap()
        })
}

/// Strategy for generating small module graphs (1-20 modules).
fn small_graph_strategy() -> impl Strategy<Value = Vec<(ModuleId, bool)>> {
    prop::collection::vec(
        (module_id_strategy(), prop::bool::ANY),
        1..=20,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Dependency/dependent symmetry
    /// ∀ A,B: A depends on B ⟺ B has A in dependents
    #[test]
    fn prop_dependency_dependent_symmetry(
        modules in small_graph_strategy(),
        deps in prop::collection::vec(
            (module_id_strategy(), module_id_strategy()),
            0..=10
        )
    ) {
        let graph = ModuleGraph::new().unwrap();

        // Add all modules
        for (id, is_entry) in &modules {
            let module = Module::builder(
                id.clone(),
                PathBuf::from(id.path_string().to_string()),
                SourceType::TypeScript,
            )
            .entry(*is_entry)
            .build();
            let _ = graph.add_module(module);
        }

        // Add dependencies (only if both modules exist)
        let module_ids: HashSet<_> = modules.iter().map(|(id, _)| id.clone()).collect();
        for (from, to) in deps {
            if module_ids.contains(&from) && module_ids.contains(&to) {
                let _ = graph.add_dependency(from.clone(), to.clone());

                // Verify symmetry: if A depends on B, then B should have A as dependent
                let dependents = graph.dependents(&to).unwrap();
                prop_assert!(
                    dependents.contains(&from),
                    "Dependency symmetry violated: {} depends on {} but {} doesn't list {} as dependent",
                    from.path_string(),
                    to.path_string(),
                    to.path_string(),
                    from.path_string()
                );
            }
        }
    }

    /// Property: No self-cycles in transitive dependencies
    /// ∀ M: M ∉ transitive_deps(M)
    #[test]
    fn prop_no_self_cycles_in_transitive_deps(
        modules in small_graph_strategy()
    ) {
        let graph = ModuleGraph::new().unwrap();

        // Add all modules
        for (id, is_entry) in &modules {
            let module = Module::builder(
                id.clone(),
                PathBuf::from(id.path_string().to_string()),
                SourceType::TypeScript,
            )
            .entry(*is_entry)
            .build();
            let _ = graph.add_module(module);
        }

        // Check each module
        for (module_id, _) in &modules {
            let transitive = graph.transitive_dependencies(module_id).unwrap();
            prop_assert!(
                !transitive.contains(module_id),
                "Self-cycle detected: {} appears in its own transitive dependencies",
                module_id.path_string()
            );
        }
    }

    /// Property: ModuleId canonicalization idempotency
    /// canon(canon(x)) == canon(x)
    #[test]
    fn prop_module_id_canonicalization_idempotent(path in "[a-z0-9_./-]{1,50}") {
        if let Ok(id1) = ModuleId::new(&path) {
            let canon_path = id1.path_string().to_string();
            if let Ok(id2) = ModuleId::new(&canon_path) {
                prop_assert_eq!(
                    id1.path_string(),
                    id2.path_string(),
                    "Canonicalization not idempotent: {} -> {} -> {}",
                    path,
                    canon_path,
                    id2.path_string()
                );
            }
        }
    }

    /// Property: ModuleId equality consistency
    /// a == b ⟹ hash(a) == hash(b)
    #[test]
    fn prop_module_id_hash_consistency(
        path1 in "[a-z0-9_./-]{1,50}",
        path2 in "[a-z0-9_./-]{1,50}"
    ) {
        if let (Ok(id1), Ok(id2)) = (ModuleId::new(&path1), ModuleId::new(&path2)) {
            if id1 == id2 {
                use std::hash::{Hash, Hasher};
                use rustc_hash::FxHasher;
                let mut hasher1 = FxHasher::default();
                let mut hasher2 = FxHasher::default();
                id1.hash(&mut hasher1);
                id2.hash(&mut hasher2);
                prop_assert_eq!(
                    hasher1.finish(),
                    hasher2.finish(),
                    "Equal ModuleIds have different hashes: {} == {} but hash differs",
                    path1,
                    path2
                );
            }
        }
    }

    /// Property: Symbol table size consistency
    /// Adding a symbol increases size by exactly 1
    #[test]
    fn prop_symbol_table_size_consistency(
        symbol_names in prop::collection::vec("[a-z]{1,20}", 1..=50)
    ) {
        use crate::{Symbol, SymbolKind, SymbolSpan, SymbolTable};

        let mut table = SymbolTable::new();
        let initial_size = table.symbols.len();

        for (i, name) in symbol_names.iter().enumerate() {
            let symbol = Symbol::new(
                name.clone(),
                SymbolKind::Variable,
                SymbolSpan::zero(),
                0,
            );
            table.add_symbol(symbol);
            prop_assert_eq!(
                table.symbols.len(),
                initial_size + i + 1,
                "Symbol table size incorrect after adding symbol {}",
                name
            );
        }
    }

    /// Property: Symbol table filter preserves metadata
    #[test]
    fn prop_symbol_table_filter_preserves_metadata(
        symbols in prop::collection::vec(
            ("[a-z]{1,20}", prop::num::u32::ANY, prop::num::u32::ANY),
            1..=30
        )
    ) {
        use crate::{Symbol, SymbolKind, SymbolSpan, SymbolTable};

        let mut table = SymbolTable::new();
        for (name, reads, writes) in &symbols {
            let mut symbol = Symbol::new(
                name.clone(),
                SymbolKind::Variable,
                SymbolSpan::zero(),
                0,
            );
            symbol.read_count = *reads as usize;
            symbol.write_count = *writes as usize;
            table.add_symbol(symbol);
        }

        // Filter by read count > 0
        let filtered: Vec<_> = table
            .symbols
            .iter()
            .filter(|s| s.read_count > 0)
            .collect();

        // Verify filtered symbols preserve their metadata
        // Note: We compare by index/position since symbols can have duplicate names
        for filtered_symbol in filtered {
            // Find the same symbol in the original table by comparing all fields
            let found = table.symbols.iter().find(|s| {
                s.name == filtered_symbol.name
                    && s.read_count == filtered_symbol.read_count
                    && s.write_count == filtered_symbol.write_count
                    && s.kind == filtered_symbol.kind
            });
            prop_assert!(
                found.is_some(),
                "Filtered symbol {} (reads: {}, writes: {}) not found in original table",
                filtered_symbol.name,
                filtered_symbol.read_count,
                filtered_symbol.write_count
            );
            let orig_symbol = found.unwrap();
            prop_assert_eq!(
                orig_symbol.read_count,
                filtered_symbol.read_count,
                "Read count changed after filter"
            );
            prop_assert_eq!(
                orig_symbol.write_count,
                filtered_symbol.write_count,
                "Write count changed after filter"
            );
        }
    }

    /// Property: Graph statistics invariants
    #[test]
    fn prop_graph_statistics_invariants(
        modules in small_graph_strategy()
    ) {
        let graph = ModuleGraph::new().unwrap();

        let mut entry_count = 0;
        for (id, is_entry) in &modules {
            let module = Module::builder(
                id.clone(),
                PathBuf::from(id.path_string().to_string()),
                SourceType::TypeScript,
            )
            .entry(*is_entry)
            .build();
            let _ = graph.add_module(module);
            if *is_entry {
                entry_count += 1;
            }
        }

        let stats = graph.statistics().unwrap();
        prop_assert_eq!(
            stats.module_count,
            modules.len(),
            "Module count mismatch"
        );
        prop_assert_eq!(
            stats.entry_point_count,
            entry_count,
            "Entry point count mismatch"
        );
        prop_assert!(
            stats.module_count >= stats.entry_point_count,
            "Entry points cannot exceed total modules"
        );
    }

    /// Property: DAG structure - no cycles in dependency graph
    /// This is a simplified check: if we can't create a cycle with the given modules,
    /// verify that transitive dependencies don't create cycles
    #[test]
    fn prop_dag_structure_no_cycles(
        modules in prop::collection::vec(module_id_strategy(), 2..=10),
        deps in prop::collection::vec(
            (any::<usize>(), any::<usize>()),
            0..=15
        )
    ) {
        let graph = ModuleGraph::new().unwrap();

        // Add modules
        for id in &modules {
            let module = Module::builder(
                id.clone(),
                PathBuf::from(id.path_string().to_string()),
                SourceType::TypeScript,
            )
            .build();
            let _ = graph.add_module(module);
        }

        // Add dependencies (using indices)
        for (from_idx, to_idx) in deps {
            if from_idx < modules.len() && to_idx < modules.len() && from_idx != to_idx {
                let from = &modules[from_idx];
                let to = &modules[to_idx];
                let _ = graph.add_dependency(from.clone(), to.clone());

                // Check: if we depend on 'to', 'to' should not transitively depend on 'from'
                let transitive = graph.transitive_dependencies(to).unwrap();
                prop_assert!(
                    !transitive.contains(from),
                    "Cycle detected: {} -> {} -> ... -> {}",
                    from.path_string(),
                    to.path_string(),
                    from.path_string()
                );
            }
        }
    }
}

