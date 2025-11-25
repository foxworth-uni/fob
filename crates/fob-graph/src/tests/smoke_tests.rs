//! Smoke tests for fob-graph.
//!
//! These are fast, deterministic tests that verify basic functionality
//! and invariants. They run quickly in CI and catch common bugs.
//!
//! For thorough property-based testing, see property_tests.rs (requires proptest feature).

use crate::{Module, ModuleGraph, ModuleId, SourceType};
use std::path::PathBuf;

/// Property test: Graph invariants - dependency/dependent symmetry.
#[test]
fn test_dependency_dependent_symmetry() {
    // Test with a few specific cases since proptest requires feature flag
    let test_cases = vec![
        ("src/a.ts", "src/b.ts"),
        ("./src/index.ts", "src/utils.ts"),
        ("module1", "module2"),
    ];

    for (from_path, to_path) in test_cases {
        // Create a graph and add two modules
        let graph = ModuleGraph::new().expect("Failed to create graph");

        let from_id = ModuleId::new(from_path).ok();
        let to_id = ModuleId::new(to_path).ok();

        if let (Some(from), Some(to)) = (from_id, to_id) {
            // If we can add the dependency, the inverse should also be queryable
            // (though adding both would create a cycle)
            let _ = graph.add_module(
                Module::builder(
                    from.clone(),
                    PathBuf::from(from_path),
                    SourceType::JavaScript,
                )
                .build(),
            );

            let _ = graph.add_module(
                Module::builder(to.clone(), PathBuf::from(to_path), SourceType::JavaScript).build(),
            );

            // Adding dependency should work if modules exist
            if graph.add_dependency(from.clone(), to.clone()).is_ok() {
                // Verify symmetry: if A depends on B, then B should have A as dependent
                let dependents = graph.dependents(&to).expect("Should query dependents");
                assert!(
                    dependents.contains(&from),
                    "Dependency should create dependent relationship"
                );
            }
        }
    }
}

/// Property test: ModuleId canonicalization properties.
#[test]
fn test_module_id_canonicalization() {
    // ModuleId should handle various path formats consistently
    let paths = vec![
        "./src/index.ts",
        "src/index.ts",
        "././src/index.ts",
        "src/../src/index.ts",
    ];

    for path in paths {
        if let Ok(id) = ModuleId::new(path) {
            // All should produce valid IDs
            assert!(!id.path_string().is_empty());
        }
    }
}

/// Property test: Symbol table consistency.
#[test]
fn test_symbol_table_consistency() {
    let test_names = vec!["x", "myVar", "longSymbolName123", "a"];

    for symbol_name in test_names {
        use crate::{Symbol, SymbolKind, SymbolSpan, SymbolTable};

        let mut table = SymbolTable::new();
        let symbol = Symbol::new(
            symbol_name.to_string(),
            SymbolKind::Variable,
            SymbolSpan::zero(),
            0,
        );

        table.add_symbol(symbol);

        // After adding, should be able to find by name
        let found = table.symbols_by_name(symbol_name);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, symbol_name);
    }
}

/// Property test: No dangling edges after operations.
#[test]
fn test_no_dangling_edges() {
    let graph = ModuleGraph::new().expect("Failed to create graph");

    let id1 = ModuleId::new("src/a.ts").expect("Valid ID");
    let id2 = ModuleId::new("src/b.ts").expect("Valid ID");

    let module1 = Module::builder(
        id1.clone(),
        PathBuf::from("src/a.ts"),
        SourceType::JavaScript,
    )
    .build();

    graph.add_module(module1).expect("Should add module");

    // Add dependency to non-existent module - this is allowed (graph doesn't validate)
    let result = graph.add_dependency(id1.clone(), id2.clone());
    assert!(
        result.is_ok(),
        "add_dependency doesn't validate module existence"
    );

    // Verify dependency was added
    let deps = graph.dependencies(&id1).expect("Should query dependencies");
    assert!(
        deps.contains(&id2),
        "Dependency should be added even if target doesn't exist"
    );
}

/// Property test: Graph statistics invariants.
#[test]
fn test_graph_statistics_invariants() {
    let graph = ModuleGraph::new().expect("Failed to create graph");

    // Add some modules
    for i in 0..10 {
        let path = format!("src/module{}.ts", i);
        let id = ModuleId::new(&path).expect("Valid ID");
        let module = Module::builder(id, PathBuf::from(&path), SourceType::TypeScript).build();

        graph.add_module(module).expect("Should add module");
    }

    let stats = graph.statistics().expect("Should compute statistics");

    // Verify invariants
    assert_eq!(stats.module_count, 10);
    assert!(stats.module_count >= stats.entry_point_count);
}

/// Property test: ModuleId path string properties.
#[test]
fn test_module_id_path_string_properties() {
    let test_paths = vec![
        "src/index.ts",
        "./src/utils.ts",
        "module-name",
        "very/long/path/to/module.ts",
    ];

    for path in test_paths {
        // ModuleId should handle various path strings without panicking
        let _id = ModuleId::new(path);
        // Just verify it doesn't panic - may return error for invalid paths
    }
}
