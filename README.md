# Fob

**A JavaScript bundler you can embed in your code.**

Instead of running a CLI tool, call Fob as a library. Build meta-frameworks, custom build tools, or bundle dynamically at runtime.

## Quick Start

### JavaScript/Node.js

```javascript
import { bundle } from '@fob/bundler';

const result = await bundle({
  entries: ['src/index.js'],
  outputDir: 'dist',
  format: 'esm',
});

console.log(`Built ${result.stats.totalModules} modules`);
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

## Documentation

- [JavaScript API](packages/fob-bundler/README.md)
- [Rust API](crates/fob-core/README.md)
- [Examples](examples/)

## License

MIT
