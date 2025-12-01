# Fob

**A JavaScript bundler you can embed in your code.**

Instead of running a CLI tool, call Fob as a library. Build meta-frameworks, custom build tools, or bundle dynamically at runtime.

## Quick Start

### JavaScript/TypeScript API (NAPI)

```bash
npm install @fox-uni/fob
```

```typescript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.ts'],
  outputDir: 'dist',
  format: 'esm',
});

const result = await bundler.bundle();

console.log(`Built ${result.stats.total_modules} modules`);
console.log(`Generated ${result.chunks.length} chunks`);
```

### Rust

```rust
use fob_bundler::BuildOptions;

let result = BuildOptions::library("src/index.ts")
    .external(["react", "react-dom"])
    .build()
    .await?;

println!("Built {} modules", result.stats().module_count);
```

## Why Use Fob as a Library?

**Traditional bundlers** are CLI tools you invoke:

```bash
webpack --config webpack.config.js
rollup -c
```

**Fob is a library** you call from your code:

```javascript
const result = await bundle({ entries: ['src/index.js'] });
// Inspect results, make decisions, bundle again
```

### Use Cases

- **Build meta-frameworks** - Scan directories, bundle routes dynamically
- **Custom build tools** - Embed bundling in your toolchain
- **Dynamic bundling** - Bundle based on runtime conditions
- **IDE extensions** - Bundle in-process without spawning CLI
- **Testing** - Bundle test fixtures programmatically

## Examples

### Simple Bundle (JavaScript)

```javascript
import { bundle } from '@fob/bundler';

const result = await bundle({
  entries: ['src/index.js'],
  outputDir: 'dist',
  format: 'esm',
  sourceMaps: 'external',
});

// Inspect results
for (const chunk of result.chunks) {
  console.log(`${chunk.fileName} (${chunk.size} bytes)`);
}
```

**[See full example →](examples/js/fob-simple)**

### Component Library (Rust)

```rust
use fob_bundler::{BuildOptions, NativeRuntime};
use std::sync::Arc;

let result = BuildOptions::library("components/index.ts")
    .external(["react", "react-dom"])
    .runtime(Arc::new(NativeRuntime))
    .build()
    .await?;

result.write_to_force("dist")?;
```

**[See full example →](examples/rust/component-library)**

### Meta-Framework (Rust)

```rust
// Discover routes from filesystem
let routes = discover_routes("app/routes")?;

// Bundle them dynamically
BuildOptions::app(routes)
    .path_alias("@", "./app")
    .minify(true)
    .build()
    .await?;
```

**[See full example →](examples/rust/meta-framework)**

## Features

- **Library-first** - Call from JavaScript or Rust
- **Type-safe** - Structured results, not strings to parse
- **Cross-platform** - Native (CLI/Node.js) and WASM (browser/edge)
- **Task-based API** - `library()`, `app()`, `components()` presets
- **Analysis without bundling** - Fast module graph analysis
- **Integrated docs** - Extract JSDoc during builds

## Installation

### JavaScript/Node.js

```bash
npm install @fob/bundler
```

### Rust

```toml
[dependencies]
fob-core = "0.1"
```

## JavaScript/TypeScript API

### Installation

```bash
npm install @fox-uni/fob
```

### Quick Start

```typescript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.ts'],
  outputDir: 'dist',
  format: 'esm',
});

const result = await bundler.bundle();
```

### Configuration Reference

#### `BundleConfig`

```typescript
interface BundleConfig {
  // Required
  entries: string[];

  // Output
  outputDir?: string; // default: "dist"

  // Build behavior
  external?: string[]; // packages to externalize
  platform?: 'browser' | 'node'; // target runtime, default: "browser"

  // Output format
  format?: 'esm' | 'cjs' | 'iife'; // lowercase strings

  // Optimizations
  minify?: boolean; // enable minification
  sourcemap?: string; // source map generation: "true"/"external" (external file), "false" (disabled), "inline", or "hidden"

  // Context
  cwd?: string; // working directory for resolution
}
```

#### Configuration Options

- **`entries`** (required): Array of entry point file paths to bundle
- **`outputDir`**: Output directory for bundled files (default: `"dist"`)
- **`external`**: Array of package names to externalize (not bundled). Essential for library authors.
- **`platform`**: Target runtime environment - `"browser"` (default) or `"node"`
- **`format`**: Output module format - `"esm"` (default), `"cjs"`, or `"iife"`
- **`minify`**: Enable JavaScript minification (default: `false`)
- **`sourcemap`**: Source map generation mode (string):
  - `"true"` or `"external"`: Generate external `.map` file
  - `"false"`: Disable source maps
  - `"inline"`: Generate inline source map (data URI)
  - `"hidden"`: Generate source map but don't reference it in bundle
- **`cwd`**: Working directory for module resolution (default: current directory)

### Result Types

#### `BundleResult`

```typescript
interface BundleResult {
  chunks: ChunkInfo[];
  manifest: ManifestInfo;
  stats: BuildStatsInfo;
  assets: AssetInfo[];
  module_count: number;
}
```

#### `ChunkInfo`

```typescript
interface ChunkInfo {
  id: string;
  kind: string; // "entry" | "async" | "shared"
  file_name: string;
  code: string;
  source_map?: string;
  modules: ModuleInfo[];
  imports: string[];
  dynamic_imports: string[];
  size: number;
}
```

### Examples

#### Library Build (with external dependencies)

```typescript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.ts'],
  outputDir: 'dist',
  format: 'esm',
  external: ['react', 'react-dom'],
  sourcemap: 'external',
});

const result = await bundler.bundle();
```

#### App Build (bundled dependencies)

```typescript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.ts'],
  outputDir: 'dist',
  format: 'esm',
  minify: true,
  sourcemap: 'inline',
});

const result = await bundler.bundle();
```

#### Node.js Target

```typescript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.ts'],
  outputDir: 'dist',
  format: 'cjs',
  platform: 'node',
  external: ['fs', 'path'],
});

const result = await bundler.bundle();
```

## Documentation

- [JavaScript API](packages/fob-bundler/README.md)
- [Rust API](crates/fob-core/README.md)
- [Examples](examples/)

## License

MIT
