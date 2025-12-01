# joy-mdx

[![Crates.io](https://img.shields.io/crates/v/joy-mdx.svg)](https://crates.io/crates/joy-mdx)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> MDX v3 compiler for the Joy ecosystem - Built for Rolldown with a standalone compilation API

**joy-mdx** is a high-performance MDX compiler built for the [Joy](https://github.com/nine-gen/joy) bundler ecosystem. It transforms MDX files into React 19-compatible JSX and provides native [Rolldown](https://rolldown.rs) plugin integration. The core `compile()` function can be used standalone for custom integrations.

## Features

- ✅ **React 19 compatible** - Generates modern JSX with automatic runtime
- ✅ **Rolldown plugin** - Native integration with Rolldown bundler
- ✅ **GFM support** - Tables, strikethrough, task lists, autolinks
- ✅ **Footnotes** - Reference-style footnotes with automatic backrefs
- ✅ **Math** - Inline (`$...$`) and block (`$$...$$`) math expressions
- ✅ **Pluggable** - Extend with custom plugins
- ✅ **Fast** - Rust-native performance with `markdown-rs` and OXC
- ✅ **Type-safe** - Full type information and error contexts

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
joy-mdx = "0.1"
```

### Feature Flags

joy-mdx supports optional features for different use cases:

#### Default Usage (With Rolldown Plugin)

```toml
[dependencies]
joy-mdx = "0.1"  # Includes Rolldown plugin by default
```

Use this when you need the `JoyMdxPlugin` for Rolldown integration.

#### Minimal Usage (Compiler Only)

```toml
[dependencies]
joy-mdx = { version = "0.1", default-features = false }
```

Use this for:

- Just the `compile()` function
- Custom bundler integrations
- Smaller dependency tree
- Faster compile times
- Environments where Rolldown doesn't compile

**Note:** When `default-features = false`, the `JoyMdxPlugin` and `rolldown_plugin` module are not available.

## Quick Start

```rust
use joy_mdx::{compile, MdxCompileOptions};

fn main() -> anyhow::Result<()> {
    let mdx = r#"
---
title: My Post
---

# Hello World

This is **bold** text with ~~strikethrough~~.

## Math

Inline: $E = mc^2$

Block:
$$
\int_0^\infty x^2 dx
$$
"#;

    // Zero config: all features ON, default plugins ON
    let result = compile(mdx, MdxCompileOptions::new())?;

    println!("JSX: {}", result.code);
    println!("Images: {:?}", result.images);
    println!("Frontmatter: {:?}", result.frontmatter);

    Ok(())
}
```

## Usage with Rolldown

joy-mdx provides a **native Rolldown plugin** for seamless integration:

```rust
use joy_mdx::JoyMdxPlugin;
use rolldown::BundlerBuilder;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create the MDX plugin (pre-configured with all features)
    let mdx_plugin = Arc::new(JoyMdxPlugin::new());

    // Add to Rolldown bundler
    let mut bundler = BundlerBuilder::default()
        .with_input(vec!["./src/index.mdx".into()])
        .with_plugins(vec![mdx_plugin])
        .build()?;

    let output = bundler.write().await?;
    println!("✅ Bundled {} assets", output.assets.len());

    Ok(())
}
```

**What the plugin does:**

- Intercepts `.mdx` files during bundling
- Compiles MDX → JSX using joy-mdx
- Returns JSX as `ModuleType::Jsx` for further processing
- Pre-configured with GFM, footnotes, math, and default plugins

### With Joy (Simplified API)

Joy bundles MDX with the task-based builders:

```rust
use joy_core::{library, plugin, JoyMdxPlugin};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bundle = library("./src/index.mdx")
        .plugin(plugin(JoyMdxPlugin::new()))
        .bundle()
        .await?;

    println!("✅ Bundled {} assets", bundle.output().assets.len());
    Ok(())
}
```

Joy automatically wires `JoyMdxPlugin` when you call `plugin(JoyMdxPlugin::new())`.

### Standalone Usage (Any Bundler)

Use the `compile()` function directly for custom integrations:

```rust
use joy_mdx::{compile, MdxCompileOptions};

// In your bundler plugin (Rollup, esbuild, Vite, etc.)
// Zero config: all features ON, default plugins ON
let mdx_result = compile(source, MdxCompileOptions::new())?;

// Pass mdx_result.code to your bundler
```

## API Reference

### `compile(source: &str, options: MdxCompileOptions) -> Result<MdxCompileResult>`

Main compilation function.

**Arguments:**

- `source` - MDX source code
- `options` - Compilation options

**Returns:**

- `MdxCompileResult` - Compiled JSX and metadata

### `MdxCompileOptions`

Configuration for MDX compilation.

**Fields (all enabled by default):**

- `filepath: Option<String>` - File path for error reporting
- `gfm: bool` - Enable GitHub Flavored Markdown (default: true)
- `footnotes: bool` - Enable footnotes (default: true)
- `math: bool` - Enable math expressions (default: true)
- `use_default_plugins: bool` - Use HeadingIdPlugin and ImageOptimizationPlugin (default: true)
- `plugins: Vec<Box<dyn MdxPlugin>>` - Additional custom plugins

**Methods:**

- `new()` - Create with sensible defaults (all features ON)
- `with_plugin(plugin)` - Add a custom plugin (on top of defaults)
- `with_jsx_runtime(runtime)` - Set JSX runtime (default: "react/jsx-runtime")

### `MdxCompileResult`

Result of MDX compilation.

**Fields:**

- `code: String` - Generated JSX code
- `frontmatter: Option<FrontmatterData>` - Parsed frontmatter (YAML/TOML)
- `images: Vec<String>` - Collected image URLs
- `named_exports: Vec<String>` - Named export statements
- `reexports: Vec<String>` - Re-export statements
- `imports: Vec<String>` - Import statements
- `default_export: Option<String>` - Default export name

## Plugins

joy-mdx includes several built-in plugins:

### HeadingIdPlugin

Automatically generates anchor IDs for headings:

```markdown
## My Section → <h2 id="my-section">My Section</h2>
```

### ImageOptimizationPlugin

Collects image URLs for optimization (enabled by default):

```rust
let result = compile(mdx, MdxCompileOptions::new())?;
// result.images contains all image URLs found
```

### LinkValidationPlugin

Validates internal links during compilation (optional, development-only):

```rust
let options = MdxCompileOptions::new()
    .with_plugin(Box::new(joy_mdx::plugins::LinkValidationPlugin::default()));
```

## Custom Plugins

Implement the `MdxPlugin` trait to create custom plugins:

```rust
use joy_mdx::{MdxPlugin, mdx::plugin_trait::PluginResult};
use markdown::mdast::Node;

struct MyPlugin;

impl MdxPlugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn transform_ast(&self, ast: &mut Node) -> PluginResult<()> {
        // Transform the markdown AST before JSX conversion
        Ok(())
    }

    fn transform_jsx(&self, jsx: &mut String) -> PluginResult<()> {
        // Transform the generated JSX
        Ok(())
    }
}
```

## Error Handling

joy-mdx provides detailed error messages with context:

```rust
let result = compile(bad_mdx, options);

match result {
    Ok(compiled) => { /* success */ },
    Err(err) => {
        eprintln!("MDX Error: {}", err.message);
        if let Some(file) = err.file {
            eprintln!("  in {}", file);
        }
        if let (Some(line), Some(col)) = (err.line, err.column) {
            eprintln!("  at {}:{}", line, col);
        }
        if let Some(suggestion) = err.suggestion {
            eprintln!("  💡 {}", suggestion);
        }
    }
}
```

## Generated Code Example

**Input MDX:**

```markdown
# Hello

This is **bold** text.
```

**Output JSX:**

```jsx
import { useMDXComponents } from '@fob/mdx-runtime';
import { Fragment as _Fragment } from 'react';

export default function MDXContent({ components: userComponents, ...props }) {
  const _components = useMDXComponents(userComponents);
  const { h1, p, strong } = _components;

  return (
    <_Fragment>
      {h1 ? <h1 {...props}>Hello</h1> : <h1>Hello</h1>}
      {p ? (
        <p {...props}>
          This is {strong ? <strong {...props}>bold</strong> : <strong>bold</strong>} text.
        </p>
      ) : (
        <p>
          This is <strong>bold</strong> text.
        </p>
      )}
    </_Fragment>
  );
}
```

## Comparison with Joy Tools

| Tool              | Purpose            | When to Use                           |
| ----------------- | ------------------ | ------------------------------------- |
| `joy-mdx`         | MDX→JSX compiler   | Core compiler, used by all Joy tools  |
| `joy-core`        | Build-time bundler | Static sites, pre-build all content   |
| `joy-mdx-bundler` | Runtime bundler    | SSR apps, CMS-driven content          |
| `joy-wasm`        | Edge/WASI bundler  | StackBlitz, Fastly, edge environments |

## Comparison with JavaScript Tools

| Feature          | joy-mdx                  | @mdx-js/mdx | mdx-bundler    |
| ---------------- | ------------------------ | ----------- | -------------- |
| Language         | Rust                     | JavaScript  | JavaScript     |
| React 19         | ✅                       | ❌ (16-18)  | ❌             |
| GFM Built-in     | ✅                       | ❌ (plugin) | ✅             |
| Bundler          | Rolldown                 | Any         | esbuild        |
| Runtime Bundling | ✅ (via joy-mdx-bundler) | ❌          | ✅             |
| Speed            | Fast (Rust)              | Medium      | Fast (esbuild) |
| WASM Support     | ✅                       | ❌          | ❌             |

## Examples

See the integration tests for real-world usage:

- [`joy-core/tests/mdx_integration.rs`](../joy-core/tests/mdx_integration.rs) - Build-time bundling with Rolldown
- [`joy-mdx-bundler/src/lib.rs`](../joy-mdx-bundler/src/lib.rs) - Runtime bundling examples
- [`joy-wasm/tests/integration.rs`](../joy-wasm/tests/integration.rs) - WASM bundling for edge

Run tests:

```bash
# Test Rolldown plugin integration
cargo test -p joy-core mdx_integration

# Test runtime bundling
cargo test -p joy-mdx-bundler

# Test WASM bundling
cargo test -p joy-wasm
```

## Development

```bash
# Build the crate
cargo build -p joy-mdx

# Run tests
cargo test -p joy-mdx

# Check for issues
cargo clippy -p joy-mdx

# Test Rolldown plugin integration
cargo test -p joy-core mdx_integration
```

## Architecture

joy-mdx is built on top of:

- **`markdown-rs`** - Fast, safe markdown parser with MDX support
- **`oxc`** - JavaScript parser for ESM validation
- **`serde`** - Frontmatter deserialization (YAML/TOML)

The compilation pipeline:

1. Parse MDX source to markdown AST (`mdast`)
2. Extract and parse frontmatter (YAML/TOML)
3. Apply pre-conversion plugins (AST transforms)
4. Convert `mdast` to JSX code
5. Apply post-conversion plugins (JSX transforms)
6. Collect metadata (images, exports, imports)
7. Return `MdxCompileResult`

## License

MIT © [Joy Contributors](https://github.com/nine-gen/joy)

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Related Projects

- [Joy](https://github.com/nine-gen/joy) - Pre-configured Rolldown + MDX for modern web development
- [Rolldown](https://rolldown.rs) - Rust-based JavaScript bundler (Joy's core bundler)
- [markdown-rs](https://github.com/wooorm/markdown-rs) - The underlying markdown parser
- [OXC](https://oxc.rs) - JavaScript parser for ESM validation

## Status

**Early development** - API may change before 1.0 release.

## Support

- 📖 [Documentation](https://joy.dev/docs/mdx)
- 💬 [Discussions](https://github.com/nine-gen/joy/discussions)
- 🐛 [Issues](https://github.com/nine-gen/joy/issues)
