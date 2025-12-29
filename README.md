# Fob

**A JavaScript bundler you can embed in your code.**

Instead of running a CLI tool, call Fob as a library. Build meta-frameworks, custom build tools, or bundle dynamically at runtime.

## Language Bindings

| Language | Package | Status |
|----------|---------|--------|
| **Rust** | `fob-bundler` | 🚧 Beta |
| **Node.js** | `@fox-uni/fob` | 🚧 Beta |
| **Python** | `fob` | 🧪 Alpha |
| **Ruby** | `fob_ruby` | 🧪 Alpha |
| **PHP** | `fob-php` | 🧪 Alpha |

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

### Python

```bash
pip install fob
# or: maturin develop --manifest-path crates/fob-python/Cargo.toml
```

```python
import fob
import asyncio

async def main():
    bundler = fob.Fob({
        "entries": ["./src/index.ts"],
        "output_dir": "dist",
        "format": "esm",
    })
    result = await bundler.bundle()
    print(f"Built {result['stats']['total_modules']} modules")

asyncio.run(main())
```

### Ruby

```ruby
require 'fob_ruby'

bundler = Fob::Bundler.new(
  entries: ['./src/index.ts'],
  out_dir: 'dist',
  format: :esm
)

result = bundler.bundle
puts "Built #{result[:stats][:total_modules]} modules"
```

### PHP

```php
<?php
$bundler = new Fob([
    'entries' => ['./src/index.ts'],
    'output_dir' => 'dist',
    'format' => 'esm',
]);

$result = $bundler->bundle();
echo "Built {$result['stats']['total_modules']} modules\n";
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

### Node.js

```javascript
const bundler = new Fob({
  entries: [{
    content: "console.log('Hello!');",
    name: 'main.js',
  }],
  outputDir: 'dist',
});
```

### Python

```python
bundler = fob.Fob({
    "entries": [{
        "content": "console.log('Hello!');",
        "name": "main.js",
    }],
    "output_dir": "dist",
})
```

### Ruby

```ruby
bundler = Fob::Bundler.new(
  entries: [{
    content: "console.log('Hello!');",
    name: "main.js"
  }],
  out_dir: "dist"
)
```

### PHP

```php
$bundler = new Fob([
    'entries' => [[
        'content' => "console.log('Hello!');",
        'name' => 'main.js',
    ]],
    'output_dir' => 'dist',
]);
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

All language bindings share the same configuration options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `entries` | `string[]` or `Entry[]` | required | Entry points (paths or inline content) |
| `outputDir` | `string` | `"dist"` | Output directory |
| `format` | `"esm"` \| `"cjs"` \| `"iife"` | `"esm"` | Output module format |
| `platform` | `"browser"` \| `"node"` | `"browser"` | Target runtime |
| `minify` | `boolean` | `false` | Enable minification |
| `sourcemap` | `string` | `"false"` | `"true"`, `"inline"`, `"hidden"`, `"false"` |
| `external` | `string[]` | `[]` | Packages to externalize |
| `cwd` | `string` | current dir | Working directory |

### Entry Object

```typescript
interface Entry {
  content: string;    // Inline JavaScript/TypeScript code
  name: string;       // Virtual filename (e.g., "main.js", "app.ts")
  loader?: string;    // "js", "ts", "jsx", "tsx" (inferred from name)
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
  kind: "entry" | "async" | "shared";
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

- **Library-first** - Call from JavaScript, Python, Ruby, PHP, or Rust
- **Type-safe** - Structured results, not strings to parse
- **Cross-platform** - Native bindings and WASM (browser/edge)
- **Inline content** - Bundle code without file I/O
- **Task-based API** - `library()`, `app()`, `components()` presets (Rust)

## Examples

See the `examples/` directory for complete examples in each language:

- [`examples/js/fob-simple/`](examples/js/fob-simple/) - Node.js examples
- [`examples/python/fob-simple/`](examples/python/fob-simple/) - Python examples
- [`examples/ruby/fob-simple/`](examples/ruby/fob-simple/) - Ruby examples
- [`examples/php/fob-simple/`](examples/php/fob-simple/) - PHP examples
- [`examples/rust/`](examples/rust/) - Rust examples

## License

MIT
