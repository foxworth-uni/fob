# fob-graph

Pure graph data structures for module dependency graphs.

This crate provides the core graph primitives and `ModuleGraph` implementation without any I/O or analysis logic. It's designed to be lightweight and WASM-compatible.

## Features

- **Pure Data Structures**: No I/O, no file system dependencies
- **WASM-Compatible**: Can run in browser environments
- **Thread-Safe**: Uses `Arc` for efficient shared ownership
- **Memory Efficient**: Arena-based allocation where possible
- **Type-Safe**: Strong typing for modules, imports, exports, and dependencies
- **Extensible**: Extension trait pattern for adding custom functionality

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    ModuleGraph                              │
│  (Arc-based, thread-safe, WASM-compatible)                  │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┐
        │            │            │
        ▼            ▼            ▼
┌───────────┐  ┌───────────┐  ┌───────────┐
│  Module   │  │  Import   │  │  Export   │
│  (Node)    │  │  (Edge)   │  │  (Edge)   │
└───────────┘  └───────────┘  └───────────┘
        │            │            │
        └────────────┼────────────┘
                     │
                     ▼
        ┌──────────────────────┐
        │   SymbolTable        │
        │   (Intra-file        │
        │    analysis)         │
        └──────────────────────┘
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fob-graph = { path = "../fob-graph" }
```

## Quick Start

### Building a Module Graph

```rust
use fob_graph::{ModuleGraph, Module, ModuleId, SourceType};
use std::path::PathBuf;

// Create a new graph
let graph = ModuleGraph::new()?;

// Add a module
let module_id = ModuleId::new("src/index.ts")?;
let module = Module::builder(module_id.clone(), PathBuf::from("src/index.ts"), SourceType::TypeScript)
    .entry(true)
    .build();

graph.add_module(module)?;

// Add dependencies
let utils_id = ModuleId::new("src/utils.ts")?;
graph.add_dependency(&module_id, &utils_id)?;

// Query the graph
let dependencies = graph.dependencies(&module_id)?;
println!("Dependencies: {:?}", dependencies);
```

### Symbol Analysis

```rust
use fob_graph::semantic::analyze_symbols;
use fob_graph::{SourceType, SymbolTable};

let source = r#"
    const used = 42;
    const unused = 100;
    console.log(used);
"#;

let table: SymbolTable = analyze_symbols(source, "example.js", SourceType::JavaScript)?;

// Find unused symbols
let unused = table.unused_symbols();
println!("Unused symbols: {:?}", unused);
```

### Finding Circular Dependencies

```rust
use fob_graph::ModuleGraph;

let graph: ModuleGraph = /* ... */;

// Find all circular dependency chains
let circular = graph.dependency_chains_to(&target_module_id)?;
for chain in circular {
    if chain.has_cycle() {
        println!("Circular dependency: {}", chain.format_chain());
    }
}
```

## Core Types

### ModuleGraph

The main graph structure. Provides methods for:

- Adding/removing modules
- Querying dependencies and dependents
- Finding circular dependencies
- Computing statistics
- Symbol-level analysis

### Module

Represents a single module in the graph:

- **ModuleId**: Unique identifier (path-based or virtual)
- **Imports**: List of imports from this module
- **Exports**: List of exports from this module
- **SymbolTable**: Intra-file symbol analysis results

### SymbolTable

Tracks symbols within a single module:

- Variable declarations
- Function declarations
- Class declarations
- Usage counts (reads/writes)
- Scope information

## Extension Trait Pattern

The crate uses extension traits to add functionality without modifying core types:

```rust
use fob_graph::ModuleGraph;

// Extension traits are automatically available
let unused_exports = graph.unused_exports()?;
let statistics = graph.statistics()?;
let circular = graph.find_circular_dependencies()?;
```

## Thread Safety

`ModuleGraph` uses `Arc` internally for efficient shared ownership. Multiple threads can safely:

- Read from the graph concurrently
- Query dependencies/dependents
- Compute statistics

For modifications, use appropriate synchronization (e.g., `Mutex` or `RwLock`).

## Performance Characteristics

- **Graph Construction**: O(n) where n is the number of modules
- **Dependency Queries**: O(1) average case
- **Transitive Dependencies**: O(n) worst case
- **Circular Detection**: O(n + e) where e is the number of edges

## WASM Compatibility

The crate is designed to work in WASM environments:

- No file system dependencies
- No network dependencies
- Pure Rust data structures
- Compatible with `wasm-bindgen` and `wasm-pack`

## Examples

See the `examples/` directory for more detailed usage:

- `basic_graph.rs` - Simple graph construction
- `symbol_analysis.rs` - Symbol table usage
- `dependency_analysis.rs` - Finding dependencies
- `concurrent_access.rs` - Thread-safe operations

## Module Organization

- **Core Types**: `module.rs`, `import.rs`, `export.rs`, `module_id.rs`
- **Graph Implementation**: `memory/` directory
- **Semantic Analysis**: `semantic/` directory (symbol extraction)
- **Symbol Tracking**: `symbol/` directory (intra-file analysis)
- **Code Quality**: `quality/` directory (metrics calculation)
- **Collection**: `collection.rs` (parsing and collection)

## License

See the workspace root for license information.

## Contributing

Contributions are welcome! Please see the workspace contributing guidelines.
