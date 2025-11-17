# fob-native

Native Node.js bindings for the Fob bundler using N-API (napi-rs).

This crate provides high-performance JavaScript bundling capabilities to Node.js applications by exposing the Rust-based `fob-core` library through N-API bindings.

## Features

- **Zero-copy Data Transfer**: Uses N-API object serialization instead of JSON for 10x performance improvement
- **Rich Bundle Information**: Returns detailed chunk, module, manifest, and statistics data
- **Type-safe**: Auto-generates TypeScript types from Rust structs
- **Cross-platform**: Supports macOS, Linux, and Windows

## Building

### Development Build

Build the N-API bindings in debug mode:

```bash
just build-napi
```

Or using cargo directly:

```bash
cd crates/fob-native
cargo build
```

### Release Build

Build optimized N-API bindings:

```bash
just build-napi-release
```

Or using cargo:

```bash
cd crates/fob-native
cargo build --release
```

## Integration with TypeScript Package

The fob-native library is consumed by the `@fob/bundler` TypeScript package. To build the full stack:

### Copy Native Binary

Copy the built library to the TypeScript package (automatically detects platform):

```bash
just copy-native
```

This command:
- Detects your platform (macOS/Linux/Windows)
- Finds the latest build (release or debug)
- Copies `libfob_native.{dylib,so,dll}` → `packages/fob-bundler/index.node`

### Build TypeScript Package

Build the TypeScript wrapper and copy the native binary:

```bash
just build-ts-bundler
```

This command:
1. Runs `copy-native`
2. Installs TypeScript dependencies
3. Compiles TypeScript sources

### Full Build Workflow

**Development:**
```bash
just build-bundler
```

This runs:
1. `just build-napi` - Build Rust in debug mode
2. `just build-ts-bundler` - Copy binary + build TypeScript

**Release:**
```bash
just build-bundler-release
```

This runs:
1. `just build-napi-release` - Build Rust with optimizations
2. `just build-ts-bundler` - Copy binary + build TypeScript

## API

The native module exports:

### `Fob` Class

```typescript
class Fob {
  constructor(config: BundleConfig);
  bundle(): Promise<BundleResult>;
}
```

### `bundleSingle()` Function

Quick helper for single-entry bundles:

```typescript
function bundleSingle(
  entry: string,
  outputDir: string,
  format?: 'esm' | 'cjs' | 'iife'
): Promise<BundleResult>;
```

### `version()` Function

Get the bundler version:

```typescript
function version(): string;
```

## Return Types

The `BundleResult` includes:

- **chunks**: Array of `ChunkInfo` with code, modules, imports, source maps
- **manifest**: Entry mappings and chunk metadata
- **stats**: Build statistics (module count, total size, duration, cache hit rate)
- **assets**: Static assets emitted during bundling

See [`src/bundle_result.rs`](./src/bundle_result.rs) for detailed type definitions.

## Platform-Specific Builds

Build for a specific target:

```bash
just build-napi-platform <target-triple>
```

Example:
```bash
just build-napi-platform x86_64-unknown-linux-gnu
```

## Testing

Run the test suite:

```bash
just test-napi
```

Or using cargo:

```bash
cargo test --package fob-native
```

## Architecture

```
┌─────────────────────────────────┐
│  @fob/bundler (TypeScript)      │
│  packages/fob-bundler/          │
└────────────┬────────────────────┘
             │ N-API
             ↓
┌─────────────────────────────────┐
│  fob-native (Rust N-API)        │
│  crates/fob-native/             │
│  - BundleResult structs         │
│  - Fob class binding            │
└────────────┬────────────────────┘
             │
             ↓
┌─────────────────────────────────┐
│  fob-core (Rust)                │
│  crates/fob-core/               │
│  - BuildOptions                 │
│  - BuildResult                  │
│  - Rolldown integration         │
└─────────────────────────────────┘
```

## Performance

Using N-API `#[napi(object)]` structs instead of JSON serialization provides:

- **10x faster** data transfer for large bundle results
- **Type safety** with auto-generated TypeScript definitions
- **Zero-copy** for strings and buffers where possible

## License

See the repository root for license information.
