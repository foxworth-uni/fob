# Fob: LLM Agent Guide

**A JavaScript bundler you can embed in your Rust code**

## Quick Start (30-second orientation)

**What is Fob?**

- JavaScript/TypeScript bundler implemented as a **Rust library** (not CLI)
- Embeddable in meta-frameworks, build tools, and custom toolchains
- Cross-platform: Native (Rust/Node.js) and WASM (browser/edge)
- Built on Rolldown bundler + OXC parser ecosystem

**Key Philosophy**: Library-first, not CLI-first. You call Fob functions from your code.

**Most Common Use Cases**:

1. Building component libraries with React/Vue/Svelte
2. Creating meta-frameworks with file-based routing
3. Dynamic bundling at runtime
4. Fast module graph analysis without bundling

---

## Architecture Overview

### Crate Organization (Dependency Order)

```
Layer 1 (Foundation - No bundling, WASM-compatible):
├── fob              - Runtime trait (file I/O abstraction)
├── fob-config       - Configuration management (TOML/JSON parsing)
├── fob-graph        - Pure module graph data structures
└── fob-gen          - Code generation utilities (AST building)

Layer 2 (Analysis - With I/O, still WASM-compatible):
└── fob-analysis     - Graph analysis with file traversal

Layer 3 (Bundling - Native or WASM with Rolldown):
├── fob-bundler      - Main bundling API (BuildOptions)
└── fob-plugins-*    - CSS, MDX, Tailwind, Vue, Svelte, Astro

Layer 4 (Bindings):
├── fob-native       - Node.js N-API bindings
├── fob-cli          - Command-line interface
└── fob-browser-test - Browser testing utilities
```

**Dependency Rule**: Lower layers never depend on higher layers.

### Key Design Patterns

1. **Runtime Trait Pattern** (`fob`)
   - Abstracts file I/O for cross-platform support
   - `NativeRuntime` for native platforms
   - `WasmRuntime` for browser/edge environments
   - Always pass `Arc<dyn Runtime>` to APIs

2. **Typestate Builder Pattern** (`fob-analysis`, `fob-bundler`)
   - API enforces correctness at compile time
   - Example: `Analyzer<Unconfigured>` → `Analyzer<Configured>`
   - Prevents runtime errors (e.g., missing entry points)

3. **Extension Trait Pattern** (`fob-graph`)
   - Core types stay minimal
   - Functionality added via extension traits
   - Example: `ModuleGraphExt` adds analysis methods to `ModuleGraph`

4. **Arc-based Graph** (`fob-graph`)
   - Thread-safe, immutable data structures
   - Efficient cloning via `Arc<T>`
   - No interior mutability (no Mutex/RwLock in core types)

---

## Core APIs (What to Use When)

### 1. **Just Bundle Something** (`fob-bundler`)

```rust
use fob_bundler::{BuildOptions, NativeRuntime};
use std::sync::Arc;

// App bundling (includes dependencies)
let result = BuildOptions::app(["src/index.ts"])
    .runtime(Arc::new(NativeRuntime))
    .outdir("dist")
    .minify(true)
    .build()
    .await?;

result.write_to_force("dist")?;
```

**When**: You want to bundle JavaScript/TypeScript files.

**Presets**:

- `.app()` - Bundle everything with code splitting (for browsers/Node apps)
- `.library()` - Externalize dependencies (for npm packages)
- `.components()` - Multiple entry points without code splitting (for tree-shaking)
- `.new()` - Single entry, basic bundling
- `.new_multiple()` - Multiple entries, basic bundling

### 2. **Analyze Without Bundling** (`fob-analysis`)

```rust
use fob_analysis::Analyzer;

let analysis = Analyzer::new()
    .entry("src/index.ts")           // Typestate: Unconfigured → Configured
    .external(vec!["react", "vue"])  // Skip these packages
    .path_alias("@", "./src")        // Resolve aliases
    .analyze()                       // Only available after .entry()
    .await?;

// Query results
let unused = analysis.unused_exports()?;
let circular = analysis.find_circular_dependencies()?;
let stats = analysis.stats();
```

**When**: You need dependency graph info but don't need bundled output.

**Use Cases**:

- Finding unused exports
- Detecting circular dependencies
- Computing module counts/sizes
- LSP/IDE integration

### 3. **Work Directly with Module Graph** (`fob-graph`)

```rust
use fob_graph::{ModuleGraph, Module, ModuleId, SourceType};

let graph = ModuleGraph::new()?;
let id = ModuleId::new("src/utils.ts")?;

let module = Module::builder(id.clone(), "src/utils.ts", SourceType::TypeScript)
    .entry(false)
    .build();

graph.add_module(module)?;
graph.add_dependency(&parent_id, &id)?;

// Query
let deps = graph.dependencies(&id)?;
```

**When**: You're building custom analysis or need fine-grained control.

---

## High-Level API Reference

### BuildOptions (fob-bundler)

**Location**: `crates/fob-bundler/src/builders/unified/options.rs`

**Entry Points**:

```rust
BuildOptions::new(entry)              // Single entry, basic bundling
BuildOptions::new_multiple(entries)   // Multiple entries, basic bundling
BuildOptions::app(entries)            // Bundle app (includes all deps, code splitting)
BuildOptions::library(entry)          // Library mode (externalize deps)
BuildOptions::components(entries)     // Multiple entry points (no code splitting)
```

**Common Methods**:

```rust
.runtime(Arc<dyn Runtime>)  // Required: platform abstraction
.outdir("dist")              // Output directory
.outfile("bundle.js")        // Single output file (single entry only)
.format(OutputFormat::Esm)   // esm | cjs | iife
.minify(bool)                // Enable minification
.sourcemap(bool)             // Generate source maps (external file)
.sourcemap_inline()          // Generate inline source maps
.sourcemap_hidden()          // Generate hidden source maps
.external(Vec<&str>)         // Externalize packages
.path_alias(alias, path)     // "@" → "./src"
.bundle(bool)                // Whether to bundle dependencies (default: true)
.splitting(bool)             // Enable code splitting (default: false)
.platform(Platform::Browser) // Browser | Node
.plugin(plugin)              // Add Rolldown plugin
.virtual_file(path, content) // Add virtual file
.decorators(bool)            // Enable modern decorator transformation
.build() -> BuildResult      // Execute build
```

**Result Type**:

```rust
struct BuildResult {
    output: BuildOutput,      // Chunks and assets
    stats: BuildStats,        // Module counts, timings
    cache: CacheStats,        // Cache hit/miss stats
    // ...
}

// Write to disk
result.write_to_force("dist")?;
```

### Analyzer (fob-analysis)

**Location**: `crates/fob-analysis/src/analyzer.rs`

**Typestate Flow**:

```rust
Analyzer::new()                     // Analyzer<Unconfigured>
    .entry("src/index.ts")          // → Analyzer<Configured>
    .analyze().await?               // → Analysis
```

**Configuration**:

```rust
.entry(path)                  // Single entry (transitions state)
.entries(Vec<path>)           // Multiple entries (transitions state)
.external(Vec<&str>)          // Skip these packages
.path_alias(alias, path)      // Path resolution
.max_depth(Option<usize>)     // DoS protection
.max_modules(Option<usize>)   // DoS protection
.analyze_with_options(options) // Advanced options
```

**Analysis Methods**:

```rust
analysis.unused_exports()?                  // Find unused exports
analysis.find_circular_dependencies()?      // Detect cycles
analysis.stats()                            // Module counts
analysis.graph()                            // Access ModuleGraph
```

### ModuleGraph (fob-graph)

**Location**: `crates/fob-graph/src/memory/graph.rs`

**Core Operations**:

```rust
let graph = ModuleGraph::new()?;

// Add modules
graph.add_module(module)?;
graph.add_dependency(&from, &to)?;

// Query
graph.dependencies(&id)?       // Direct dependencies
graph.dependents(&id)?         // Who depends on this?
graph.module(&id)?             // Get module by ID
graph.all_modules()            // Iterator over all modules

// Analysis (via extension traits)
graph.transitive_dependencies(&id)?
graph.dependency_chains_to(&id)?
graph.unused_exports()?
graph.statistics()?
```

### Runtime Trait (fob)

**Location**: `crates/fob/src/runtime.rs`

**Implementations**:

- `NativeRuntime` - Uses `std::fs` and tokio
- `WasmRuntime` - Browser/WASM environment (future)

**Key Methods**:

```rust
trait Runtime: Send + Sync {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>>;
    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()>;
    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata>;
    fn exists(&self, path: &Path) -> bool;
    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf>;
    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()>;
    async fn remove_file(&self, path: &Path) -> RuntimeResult<()>;
    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>>;
    fn get_cwd(&self) -> RuntimeResult<PathBuf>;
}
```

---

## Common Task Recipes

### Task: Bundle a React Component Library

```rust
use fob_bundler::{BuildOptions, NativeRuntime};
use std::sync::Arc;

let result = BuildOptions::library("src/index.ts")
    .runtime(Arc::new(NativeRuntime))
    .external(["react", "react-dom"])  // Peer dependencies
    .outdir("dist")
    .sourcemap(true)
    .build()
    .await?;

result.write_to_force("dist")?;
```

**See**: `examples/rust/component-library/src/main.rs`

### Task: Build a Meta-Framework with Code Splitting

```rust
// 1. Discover routes from filesystem
let routes: Vec<PathBuf> = discover_routes("app/routes")?;

// 2. Bundle with code splitting
BuildOptions::app(routes)
    .runtime(Arc::new(NativeRuntime))
    .path_alias("@", "./app")
    .minify(true)
    .outdir("dist")
    .build()
    .await?;
```

**See**: `examples/rust/meta-framework/src/main.rs`

### Task: Find Unused Exports in a Codebase

```rust
let analysis = Analyzer::new()
    .entry("src/index.ts")
    .analyze()
    .await?;

let unused = analysis.unused_exports()?;
for export in unused {
    println!("Unused: {} in {}", export.export.name, export.module_id);
}
```

**See**: `crates/fob-analysis/examples/basic_analysis.rs`

### Task: Detect Circular Dependencies

```rust
let analysis = Analyzer::new()
    .entry("src/index.ts")
    .analyze()
    .await?;

let circular = analysis.find_circular_dependencies()?;
for chain in circular {
    println!("Cycle: {}", chain.format_chain());
}
```

**See**: `crates/fob-analysis/examples/circular_detection.rs`

### Task: Analyze Module Graph Without Bundling

```rust
// Fast analysis (no bundling overhead)
let analysis = Analyzer::new()
    .entry("src/index.ts")
    .external(vec!["react"])  // Skip node_modules
    .analyze()
    .await?;

let stats = analysis.stats();
println!("Total modules: {}", stats.module_count);
```

### Task: Bundle with Custom Plugins

```rust
use fob_bundler::{BuildOptions, NativeRuntime, plugin};
use fob_plugin_css::CssPlugin;

let result = BuildOptions::app(["src/index.ts"])
    .runtime(Arc::new(NativeRuntime))
    .plugin(plugin(CssPlugin::new()))  // Add CSS plugin
    .build()
    .await?;
```

### Task: Bundle with Virtual Files

```rust
let result = BuildOptions::new("virtual:entry")
    .runtime(Arc::new(NativeRuntime))
    .virtual_file("virtual:entry", r#"
        import { foo } from './real-file.js';
        console.log(foo);
    "#)
    .build()
    .await?;
```

---

## Key Files (Where to Look)

### Entry Points & Public APIs

- `crates/fob-bundler/src/lib.rs` - Main bundling API exports
- `crates/fob-bundler/src/builders/unified/options.rs` - BuildOptions implementation
- `crates/fob-bundler/src/builders/unified/output.rs` - BuildResult types
- `crates/fob-analysis/src/analyzer.rs` - Analysis API (typestate)
- `crates/fob-analysis/src/lib.rs` - Analysis exports
- `crates/fob-graph/src/lib.rs` - Graph types and exports

### Core Data Types

- `crates/fob-graph/src/module.rs` - Module representation
- `crates/fob-graph/src/module_id.rs` - Module identifier
- `crates/fob-graph/src/import.rs` - Import representation
- `crates/fob-graph/src/export.rs` - Export representation
- `crates/fob-graph/src/memory/graph.rs` - ModuleGraph implementation

### Configuration

- `crates/fob-config/src/config.rs` - Configuration types
- `crates/fob-bundler/src/builders/common.rs` - Build execution logic

### Runtime Abstraction

- `crates/fob/src/runtime.rs` - Runtime trait definition
- `crates/fob/src/native_runtime.rs` - Native implementation
- `crates/fob/src/wasm_runtime.rs` - WASM implementation

### Plugins

- `crates/fob-plugin-css/src/lib.rs` - CSS plugin
- `crates/fob-plugin-mdx/src/lib.rs` - MDX plugin
- `crates/fob-plugin-tailwind/src/lib.rs` - Tailwind plugin

---

## Testing Strategy

### Unit Tests

- Located in `src/` directories as `#[cfg(test)]` modules
- Or in `crates/*/tests/*.rs` for integration tests

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p fob-graph

# Specific test
cargo test -p fob-bundler library_tests

# With output
cargo test -- --nocapture
```

### Test Utilities

- `fob` crate has `test-utils` feature for test helpers
- `tempfile` crate for temporary directories
- Example fixtures in `crates/*/tests/fixtures/`

---

## Platform Considerations (CRITICAL)

### WASM Compatibility Rules

**Never use directly**:

- `std::fs` - Use `Runtime` trait instead
- `std::env::current_dir()` - Use `runtime.get_cwd()`
- `tokio` with default features - Use minimal features

**Always**:

- Pass `Arc<dyn Runtime>` to APIs
- Use async I/O via Runtime trait
- Check target with `#[cfg(target_family = "wasm")]`

**Layer Compatibility**:

- ✅ `fob`, `fob-config`, `fob-graph`, `fob-gen` - WASM-compatible
- ✅ `fob-analysis` - WASM-compatible (uses Runtime)
- ⚠️ `fob-bundler` - WASM-compatible via rolldown (requires Runtime)
- ❌ `fob-cli`, `fob-native` - Native-only

### Runtime Requirements

**Native platforms**:

- `NativeRuntime` is automatically created if not provided
- Uses `std::fs` and `tokio::fs` internally

**WASM platforms**:

- **Must** provide a `Runtime` implementation
- Cannot use `std::fs` directly
- Must bridge to JavaScript filesystem APIs

---

## Code Patterns & Idioms

### 1. Error Handling

```rust
// Use anyhow for application code
use anyhow::Result;

// Use thiserror for library errors
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
}
```

### 2. Async/Await

```rust
// Always use async-trait for trait methods
use async_trait::async_trait;

#[async_trait]
trait MyTrait {
    async fn do_something(&self) -> Result<()>;
}
```

### 3. Builder Pattern

```rust
// Typestate builders for compile-time safety
pub struct Builder<S> {
    state: PhantomData<S>,
    // ...
}

impl Builder<Unconfigured> {
    pub fn entry(self) -> Builder<Configured> { ... }
}

impl Builder<Configured> {
    pub async fn build(self) -> Result<Output> { ... }
}
```

### 4. Arc for Shared Ownership

```rust
// Graph types use Arc for efficient cloning
let graph: Arc<ModuleGraph> = Arc::new(ModuleGraph::new()?);
let graph_clone = Arc::clone(&graph);  // Cheap
```

### 5. Runtime Pattern

```rust
// Always use Arc<dyn Runtime> for cross-platform support
let runtime: Arc<dyn Runtime> = Arc::new(NativeRuntime);
// or for WASM: Arc::new(WasmRuntime::new(...))

// Pass to APIs
BuildOptions::new("src/index.js")
    .runtime(runtime)
    .build()
    .await?;
```

---

## Examples to Learn From

**Best starting points**:

1. **Simple bundling**:
   - `examples/rust/basic-bundler/src/main.rs`
   - Shows: entry points, output config, stats

2. **Component library**:
   - `examples/rust/component-library/src/main.rs`
   - Shows: library mode, external deps, multiple builds

3. **Meta-framework**:
   - `examples/rust/meta-framework/src/main.rs`
   - Shows: file discovery, code splitting, path aliases

4. **Analysis without bundling**:
   - `crates/fob-analysis/examples/basic_analysis.rs`
   - Shows: Analyzer API, unused exports, circular deps

5. **Direct graph manipulation**:
   - `crates/fob-graph/examples/dependency_analysis.rs`
   - Shows: ModuleGraph API, queries, traversal

6. **Framework components**:
   - `crates/fob-analysis/examples/framework_components.rs`
   - Shows: Analyzing Vue/Svelte/Astro components

7. **Path aliases**:
   - `crates/fob-analysis/examples/path_aliases.rs`
   - Shows: Configuring and using path aliases

---

## Quick Decision Tree

**I want to...**

→ **Bundle JavaScript for production**

- Use `fob-bundler::BuildOptions::app()` or `BuildOptions::new()`
- See: `examples/rust/basic-bundler`

→ **Build an npm package**

- Use `fob-bundler::BuildOptions::library()`
- See: `examples/rust/component-library`

→ **Find unused code**

- Use `fob-analysis::Analyzer`
- Call `.unused_exports()`

→ **Detect circular dependencies**

- Use `fob-analysis::Analyzer`
- Call `.find_circular_dependencies()`

→ **Build a meta-framework**

- Discover routes from filesystem
- Use `BuildOptions::app(routes)` with path aliases
- See: `examples/rust/meta-framework`

→ **Analyze module graph manually**

- Use `fob-graph::ModuleGraph` directly
- See: `crates/fob-graph/examples/`

→ **Add custom framework support** (Astro, Svelte, etc)

- Implement `FrameworkRule` trait
- See: `crates/fob-graph/src/framework_rules/`

→ **Bundle with CSS/MDX/Tailwind**

- Use plugins: `fob_plugin_css`, `fob_plugin_mdx`, `fob_plugin_tailwind`
- Add via `.plugin(plugin(...))`

---

## Vocabulary & Concepts

**Module**: A single JavaScript/TypeScript file in the dependency graph
**ModuleId**: Unique identifier for a module (usually its path)
**Entry Point**: Starting module(s) for bundling or analysis
**External**: Package to skip (not bundle), e.g., npm packages in library mode
**Runtime**: Platform abstraction for file I/O (Native vs WASM)
**SymbolTable**: Intra-file analysis (variables, functions, usage counts)
**Chunk**: Output file from bundling (code splitting creates multiple chunks)
**Path Alias**: Custom import resolution (`@` → `./src`)
**Code Splitting**: Breaking output into multiple chunks for optimal loading
**Tree Shaking**: Removing unused code from the bundle

**Rolldown**: The underlying bundler (fork of Rollup in Rust)
**OXC**: High-performance JavaScript parser and AST tools
**Typestate**: API pattern using generic types to enforce state at compile time
**Extension Trait**: Trait that adds methods to existing types without modifying them

---

## Common Pitfalls

❌ **Forgetting to set runtime**

```rust
BuildOptions::app(["src/index.js"])
    .build()  // Error: missing runtime (on WASM)
    .await?;
```

✅ **Always provide runtime (or use NativeRuntime on native)**

```rust
BuildOptions::app(["src/index.js"])
    .runtime(Arc::new(NativeRuntime))  // ✓
    .build()
    .await?;
```

❌ **Calling analyze() without entry**

```rust
Analyzer::new()
    .analyze()  // Compile error: no .entry() called
    .await?;
```

✅ **Entry required (typestate enforces this)**

```rust
Analyzer::new()
    .entry("src/index.ts")  // ✓
    .analyze()
    .await?;
```

❌ **Using std::fs in WASM-compatible code**

```rust
let content = std::fs::read("file.js")?;  // Breaks WASM
```

✅ **Use Runtime trait**

```rust
let content = runtime.read_file(Path::new("file.js")).await?;
```

❌ **Using outfile with multiple entries**

```rust
BuildOptions::app(["a.js", "b.js"])
    .outfile("bundle.js")  // Error: outfile only for single entry
    .build()
    .await?;
```

✅ **Use outdir for multiple entries**

```rust
BuildOptions::app(["a.js", "b.js"])
    .outdir("dist")  // ✓
    .build()
    .await?;
```

❌ **Using splitting without bundle**

```rust
BuildOptions::new("src/index.js")
    .bundle(false)
    .splitting(true)  // Error: splitting requires bundle: true
    .build()
    .await?;
```

✅ **Enable bundling for code splitting**

```rust
BuildOptions::app(["a.js", "b.js"])
    .bundle(true)     // ✓
    .splitting(true)  // ✓
    .build()
    .await?;
```

---

## Debugging Tips

**Enable logging**:

```rust
use tracing_subscriber;
tracing_subscriber::fmt::init();
```

**Inspect build result**:

```rust
let result = BuildOptions::app(["src/index.js"])
    .build()
    .await?;

println!("Chunks: {:#?}", result.output.chunks());
println!("Stats: {:#?}", result.stats);
println!("Cache: {:#?}", result.cache);
```

**Check module graph**:

```rust
let analysis = Analyzer::new()
    .entry("src/index.ts")
    .analyze()
    .await?;

let graph = analysis.graph();
for module in graph.all_modules() {
    println!("Module: {:?}", module.id());
    println!("  Dependencies: {:?}", graph.dependencies(&module.id())?);
    println!("  Dependents: {:?}", graph.dependents(&module.id())?);
}
```

**Validate build options**:

```rust
let options = BuildOptions::app(["src/index.js"])
    .outfile("bundle.js");  // Invalid: outfile with multiple entries

if let Err(e) = options.validate() {
    println!("Invalid config: {}", e);
}
```

**Check for errors in build result**:

```rust
let result = BuildOptions::app(["src/index.js"])
    .build()
    .await?;

// BuildResult contains diagnostics if there were warnings/errors
if !result.diagnostics.is_empty() {
    for diag in result.diagnostics {
        println!("Warning: {}", diag.message);
    }
}
```

---

## Integration with Other Tools

### Using with Tokio

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let result = BuildOptions::app(["src/index.js"])
        .runtime(Arc::new(NativeRuntime))
        .build()
        .await?;
    Ok(())
}
```

### Using with Async Runtime

```rust
// Works with any async runtime that supports async-trait
// Tokio is the default for native platforms
```

### Using with WASM

```rust
// On WASM, you must provide a Runtime implementation
// that bridges to JavaScript filesystem APIs
let runtime = Arc::new(WasmRuntime::new(js_fs_bridge));
BuildOptions::new("src/index.js")
    .runtime(runtime)
    .build()
    .await?;
```

---

## Performance Considerations

**Caching**:

- Build results include cache statistics
- Module graph analysis is cached automatically
- File reads are cached during analysis

**Parallel Processing**:

- Module graph construction is parallelized
- Analysis uses rayon for parallel traversal
- Bundle execution uses Rolldown's parallel processing

**Memory Usage**:

- ModuleGraph uses Arc for efficient sharing
- Large graphs are memory-efficient
- WASM builds have additional constraints

---

## Security Considerations

**Path Traversal Protection**:

- All paths are validated to prevent `..` escapes
- Asset resolution includes security checks
- Output paths are sanitized

**DoS Protection**:

- Maximum depth limits in Analyzer
- Maximum module count limits
- File size limits (10MB default)

**Asset Security**:

- Asset paths are validated
- Directory traversal attempts are blocked
- Large files are rejected

---

## Contributing Guidelines

**Code Style**:

- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Document public APIs with `///` comments

**Testing**:

- Write unit tests in `#[cfg(test)]` modules
- Write integration tests in `tests/` directories
- Use fixtures from `tests/fixtures/` for test data

**Documentation**:

- Update this guide when adding new APIs
- Add examples for new features
- Document breaking changes clearly

---

## Additional Resources

- **Main README**: `/README.md` - Project overview
- **Graph README**: `crates/fob-graph/README.md` - Graph API details
- **Analysis README**: `crates/fob-analysis/README.md` - Analysis API details
- **Examples**: `examples/rust/` - Working code examples
- **Tests**: `crates/*/tests/` - Test cases as examples

---

## Quick Reference: Type Signatures

```rust
// BuildOptions
BuildOptions::new(entry: impl AsRef<Path>) -> Self
BuildOptions::app(entries: impl IntoIterator) -> Self
BuildOptions::library(entry: impl AsRef<Path>) -> Self
BuildOptions::components(entries: impl IntoIterator) -> Self

// Analyzer
Analyzer::new() -> Analyzer<Unconfigured>
Analyzer<Unconfigured>::entry(path) -> Analyzer<Configured>
Analyzer<Configured>::analyze() -> Future<Output = Result<Analysis>>

// ModuleGraph
ModuleGraph::new() -> Result<ModuleGraph>
ModuleGraph::add_module(module: Module) -> Result<()>
ModuleGraph::dependencies(id: &ModuleId) -> Result<Vec<ModuleId>>

// Runtime
trait Runtime: Send + Sync {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>>;
    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()>;
    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf>;
}
```

---

_Last updated: Based on Fob v0.1.3_
