# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fob is a JavaScript/TypeScript bundler designed to be embedded as a library (not just a CLI tool). It provides language bindings for Node.js (`@fox-uni/fob`) and Rust (`fob-bundler`), built on top of Rolldown.

## Build Commands

### Primary Development Workflow

```bash
just check        # Format check + lint + build CLI + test (run before committing)
just dev          # Format + check (full development workflow)
just compile      # Quick compilation check (no tests/lint)
```

### Testing

```bash
cargo test --workspace                    # All Rust tests
cargo test --package <crate-name>         # Single crate tests
just test-napi                            # N-API JavaScript integration tests
pnpm test                                 # All JS/TS tests via Turbo
```

### Building

```bash
cargo build --workspace                   # Build all Rust crates
cargo build --package fob-cli             # Build CLI only
just build-napi                           # Build N-API bindings (release)
just build-napi-debug                     # Build N-API bindings (debug)
pnpm build                                # Build all packages via Turbo
```

### Linting & Formatting

```bash
cargo fmt --all                           # Format Rust
cargo clippy --workspace --all-features -- -D warnings
pnpm format                               # Format JS/TS
pnpm lint                                 # Lint JS/TS with oxlint
```

## Architecture

### Crate Hierarchy

```
fob-bundler (main bundler API)
├── fob-graph (pure graph data structures, WASM-compatible)
│   └── fob-gen (AST code generation using OXC)
├── fob-mdx (MDX v3 compiler)
└── rolldown (underlying bundler engine)

fob-native (Node.js N-API bindings)
└── fob-bundler

fob-cli (CLI tool)
├── fob-bundler
└── fob-gen
```

### Key Design Patterns

**Library-first**: Fob is designed to be called programmatically, not just via CLI. The `BuildOptions` and `BuildConfig` builders provide the primary Rust API.

**WASM Compatibility**: `fob-graph` and `fob-gen` are designed to run in WASM environments. Platform-specific code uses conditional compilation:

```rust
#[cfg(not(target_family = "wasm"))]
tokio = { workspace = true, features = ["rt", "fs"] }
```

**OXC Ecosystem**: Uses OXC (Oxidation Compiler) for parsing, AST manipulation, code generation, and minification. All OXC types are re-exported via `fob_graph::oxc::*` for consistent workspace versioning.

**Extension Trait Pattern**: `ModuleGraph` functionality is organized via extension traits in `fob-graph`:

- `memory::queries` - Query operations
- `memory::mutations` - Modification operations
- `memory::exports` - Export analysis
- `memory::symbols` - Symbol-level analysis
- `memory::chains` - Dependency chain analysis

### Entry Points

- **Rust Library**: `fob_bundler::BuildOptions::new("./src/index.js").build().await`
- **Rust Config-based**: `fob_bundler::BuildConfig::new("./src/index.js").bundle(false).build().await`
- **Node.js**: `new Fob({ entries: ['./src/index.ts'] }).bundle()`
- **CLI**: `fob build` (binary from fob-cli)

## Workspace Configuration

- **Rust Edition**: 2024 (requires Rust 1.85+)
- **Package Manager**: pnpm 10.24+
- **Node.js**: 18+
- **Task Runner**: just (justfile) + Turbo (turbo.json)

### Workspace Dependencies

All shared dependencies are declared in the root `Cargo.toml` under `[workspace.dependencies]` and inherited by crates using `.workspace = true`.

### Feature Flags

- `fob-bundler`: `dts-generation` (TypeScript .d.ts generation), `logging` (init_logging utilities), `test-utils`
- `fob-gen`: `parser`, `query-api`, `transform-engine`, `fob_internal`
- `fob-mdx`: `bundler` (bundler integration), `runtime`
- `fob-graph`: `proptest`, `bundler`, `test-utils`

## Code Conventions

- Maximum 500 lines per file
- Use `thiserror` for error types
- Use `miette` for diagnostic display
- Workspace lints: `async_fn_in_trait = "allow"`, `disallowed_methods = "deny"` (Clippy)
