# fob-analysis

Graph analysis with I/O and traversal capabilities for JavaScript/TypeScript module graphs.

This crate provides the `Analyzer` API and related analysis functionality that operates on top of the `fob-graph` data structures. It enables fast, standalone analysis of module dependency graphs without requiring full bundling.

## Features

- **Type-safe API**: Typestate pattern ensures analysis can only be performed after configuration is complete
- **Security**: Path traversal protection and DoS limits (max depth, max modules, file size)
- **Framework Support**: Extracts JavaScript/TypeScript from Astro, Svelte, and Vue components
- **Path Aliases**: Supports path alias resolution (e.g., `@` → `./src`)
- **External Packages**: Mark npm packages as external to skip analysis
- **Usage Analysis**: Compute export usage counts across the module graph
- **Circular Dependency Detection**: Find and report circular dependencies

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fob-analysis = { path = "../fob-analysis" }
```

## Quick Start

```rust
use fob_analysis::Analyzer;

#[tokio::main]
async fn main() -> fob::Result<()> {
    // Create analyzer and configure entry points
    let analysis = Analyzer::new()
        .entry("src/index.ts")  // Required: transitions to Configured state
        .external(vec!["react", "lodash"])  // Mark as external
        .path_alias("@", "./src")  // Configure path aliases
        .max_depth(Some(100))  // Set DoS protection limits
        .analyze()  // Only available on Configured
        .await?;

    // Use analysis results
    let unused = analysis.unused_exports()?;
    println!("Found {} unused exports", unused.len());

    let circular = analysis.find_circular_dependencies()?;
    println!("Found {} circular dependencies", circular.len());

    Ok(())
}
```

## Typestate Pattern

The `Analyzer` uses a typestate pattern to ensure type safety:

- `Analyzer<Unconfigured>` - Created with `Analyzer::new()`, cannot call `analyze()`
- `Analyzer<Configured>` - Created by calling `entry()` or `entries()`, can call `analyze()`

This prevents runtime errors from missing entry points:

```rust
// This won't compile - missing entry point
let analyzer = Analyzer::new();
// analyzer.analyze().await?;  // ❌ Compile error!

// This compiles - has entry point
let analyzer = Analyzer::new().entry("src/index.ts");
analyzer.analyze().await?;  // ✅ OK
```

## Configuration

### Entry Points

```rust
Analyzer::new()
    .entry("src/index.ts")  // Single entry
    .entries(vec!["src/a.ts", "src/b.ts"])  // Multiple entries
```

### External Packages

```rust
Analyzer::new()
    .entry("src/index.ts")
    .external(vec!["react", "lodash", "vue"])
```

### Path Aliases

```rust
Analyzer::new()
    .entry("src/index.ts")
    .path_alias("@", "./src")  // "@/components/Button" → "./src/components/Button"
    .path_alias("~", "./src")  // "~/utils/helpers" → "./src/utils/helpers"
```

### DoS Protection

```rust
Analyzer::new()
    .entry("src/index.ts")
    .max_depth(Some(100))      // Maximum dependency depth
    .max_modules(Some(100_000)) // Maximum number of modules
```

### Framework Rules

```rust
use fob_analysis::{Analyzer, AnalyzeOptions};
use fob_graph::FrameworkRule;

let options = AnalyzeOptions {
    framework_rules: vec![
        // Add your custom framework rules
        // Box::new(MyReactRule),
    ],
    compute_usage_counts: true,
};

let analysis = Analyzer::new()
    .entry("src/index.ts")
    .analyze_with_options(options)
    .await?;
```

## Examples

See the `examples/` directory for more detailed usage:

- `basic_analysis.rs` - Simple analysis workflow
- `path_aliases.rs` - Configuring and using path aliases
- `circular_detection.rs` - Detecting circular dependencies
- `framework_components.rs` - Analyzing framework-specific components

Run examples with:

```bash
cargo run --example basic_analysis
```

## Security Considerations

The analyzer includes several security features:

- **Path Traversal Protection**: All paths are validated to prevent escaping the current working directory
- **DoS Protection**: Limits on maximum depth, module count, and file size prevent resource exhaustion attacks
- **File Size Limits**: Files larger than `MAX_FILE_SIZE` (10MB) are rejected

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Analyzer API                          │
│  (Typestate pattern: Unconfigured → Configured → Analysis)  │
└────────────────────┬────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                      GraphWalker                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Traversal   │  │    Parser    │  │  Validation  │      │
│  │   (BFS)      │→ │  (Extract)   │→ │  (Security)  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    ModuleResolver                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Algorithm   │  │   Aliases    │  │  Extensions  │      │
│  │ (Resolution) │→ │  (Path maps) │→ │  (.ts, .js)  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    ModuleGraph                              │
│              (from fob-graph crate)                         │
└─────────────────────────────────────────────────────────────┘
```

## API Documentation

Full API documentation is available at [docs.rs](https://docs.rs/fob-analysis) (when published).

## License

See the workspace root for license information.

## Contributing

Contributions are welcome! Please see the workspace contributing guidelines.
