# Fob

**A JavaScript bundler you can embed in your code.**

Instead of running a CLI tool, call Fob as a library. Build meta-frameworks, custom build tools, or bundle dynamically at runtime.

## Language Bindings

| Language    | Package        | Status  |
| ----------- | -------------- | ------- |
| **Node.js** | `@fox-uni/fob` | 🚧 Beta |
| **Rust**    | `fob-bundler`  | 🚧 Beta |

## Quick Start

### Node.js

```bash
npm install @fox-uni/fob
```

```javascript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.ts'],
  outputDir: 'dist',
  format: 'esm',
});

const result = await bundler.bundle();
console.log(`Built ${result.stats.totalModules} modules`);
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

## Inline Content

Bundle code directly without files — useful for dynamic code generation:

```javascript
const bundler = new Fob({
  entries: [
    {
      content: "console.log('Hello!');",
      name: 'main.js',
    },
  ],
  outputDir: 'dist',
});
```

## Why Use Fob as a Library?

**Traditional bundlers** are CLI tools:

```bash
webpack --config webpack.config.js
rollup -c
```

**Fob is a library** you call from your code:

```javascript
const result = await bundler.bundle();
// Inspect results, make decisions, bundle again
```

### Use Cases

- **Build meta-frameworks** - Scan directories, bundle routes dynamically
- **Custom build tools** - Embed bundling in your toolchain
- **Dynamic bundling** - Bundle based on runtime conditions
- **IDE extensions** - Bundle in-process without spawning CLI
- **Testing** - Bundle test fixtures programmatically

## Configuration

| Option      | Type                           | Default     | Description                                 |
| ----------- | ------------------------------ | ----------- | ------------------------------------------- |
| `entries`   | `string[]` or `Entry[]`        | required    | Entry points (paths or inline content)      |
| `outputDir` | `string`                       | `"dist"`    | Output directory                            |
| `format`    | `"esm"` \| `"cjs"` \| `"iife"` | `"esm"`     | Output module format                        |
| `platform`  | `"browser"` \| `"node"`        | `"browser"` | Target runtime                              |
| `minify`    | `boolean`                      | `false`     | Enable minification                         |
| `sourcemap` | `string`                       | `"false"`   | `"true"`, `"inline"`, `"hidden"`, `"false"` |
| `external`  | `string[]`                     | `[]`        | Packages to externalize                     |
| `cwd`       | `string`                       | current dir | Working directory                           |

### Entry Object

```typescript
interface Entry {
  content: string; // Inline JavaScript/TypeScript code
  name: string; // Virtual filename (e.g., "main.js", "app.ts")
  loader?: string; // "js", "ts", "jsx", "tsx" (inferred from name)
}
```

## Result Types

### BundleResult

```typescript
interface BundleResult {
  chunks: ChunkInfo[];
  stats: {
    totalModules: number;
    totalSize: number;
  };
  manifest: ManifestInfo;
  assets: AssetInfo[];
}
```

### ChunkInfo

```typescript
interface ChunkInfo {
  id: string;
  kind: 'entry' | 'async' | 'shared';
  fileName: string;
  code: string;
  sourceMap?: string;
  modules: ModuleInfo[];
  imports: string[];
  dynamicImports: string[];
  size: number;
}
```

## Features

- **Library-first** - Call from JavaScript or Rust
- **Type-safe** - Structured results, not strings to parse
- **Cross-platform** - Native bindings and WASM (browser/edge)
- **Inline content** - Bundle code without file I/O
- **Task-based API** - `library()`, `app()`, `components()` presets (Rust)

## Examples

See the `examples/` directory for complete examples:

- [`examples/js/fob-simple/`](examples/js/fob-simple/) - Node.js examples
- [`examples/rust/`](examples/rust/) - Rust examples

## License

MIT
