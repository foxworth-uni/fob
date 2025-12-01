# joy-mdx-bundler

[![Crates.io](https://img.shields.io/crates/v/joy-mdx-bundler.svg)](https://crates.io/crates/joy-mdx-bundler)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> Runtime MDX compiler and bundler - the Rust equivalent of [mdx-bundler](https://github.com/kentcdodds/mdx-bundler)

**joy-mdx-bundler** compiles and bundles MDX files at runtime, perfect for CMS-driven sites, SSR applications, and dynamic content platforms. Fetch MDX from databases, APIs, or files, and get back executable JavaScript bundles ready to send to clients.

## Features

- ✅ **Runtime bundling** - Compile + bundle MDX at request time
- ✅ **Handles imports** - Bundles dependencies into single .js file
- ✅ **Framework agnostic** - Works with any Rust web framework
- ✅ **Fast** - Built on Rolldown (Rust-based bundler)
- ✅ **Virtual filesystem** - Provide dependencies as strings
- ✅ **Full MDX support** - GFM, math, footnotes, frontmatter

## Quick Start

```rust
use joy_mdx_bundler::{bundle_mdx, BundleMdxOptions};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MDX content with imports
    let mdx = r#"
---
title: My Blog Post
---

# Hello World

import Button from './Button.tsx'

<Button>Click me!</Button>
    "#;

    // Provide dependencies as virtual files
    let options = BundleMdxOptions {
        source: mdx.to_string(),
        files: HashMap::from([
            ("./Button.tsx".to_string(), r#"
export default function Button({children}) {
    return <button className="btn">{children}</button>
}
            "#.to_string()),
        ]),
        mdx_options: None, // Uses defaults (all features enabled)
    };

    // Bundle at runtime
    let result = bundle_mdx(options).await?;

    println!("Bundle size: {} bytes", result.code.len());
    // Send result.code to client for execution

    Ok(())
}
```

## Use Cases

| Scenario             | Description                                                        |
| -------------------- | ------------------------------------------------------------------ |
| **CMS Integration**  | Fetch MDX from Contentful, Sanity, etc. and bundle at request time |
| **SSR Applications** | Server-side render MDX with imported components                    |
| **Preview Systems**  | Show live previews before publishing                               |
| **Dynamic Content**  | Personalize MDX per user with dynamic imports                      |

## Comparison with Other Tools

| Tool              | Purpose              | When to Use                              |
| ----------------- | -------------------- | ---------------------------------------- |
| `joy-mdx-bundler` | Runtime MDX bundling | Server-side, dynamic content from CMS/DB |
| `joy-core`        | Build-time bundling  | Static sites, pre-build all content      |
| `joy-wasm`        | Edge/WASI bundling   | StackBlitz, Fastly, edge environments    |
| `joy-mdx`         | Just MDX compilation | Building custom integrations             |

## API Reference

### `bundle_mdx(options: BundleMdxOptions) -> Result<BundleMdxResult>`

Main entry point for runtime bundling.

**Arguments:**

- `options: BundleMdxOptions` - Configuration for bundling

**Returns:**

- `Ok(BundleMdxResult)` - Bundled code and metadata
- `Err` - Compilation or bundling error

### `BundleMdxOptions`

Configuration for runtime bundling.

**Fields:**

- `source: String` - The MDX source code
- `files: HashMap<String, String>` - Virtual filesystem for imports
- `mdx_options: Option<MdxCompileOptions>` - MDX features and plugins

**Methods:**

- `new(source)` - Create with source only
- `with_file(path, content)` - Add virtual file
- `with_mdx_options(opts)` - Set MDX options

### `BundleMdxResult`

Result of bundling.

**Fields:**

- `code: String` - Executable JavaScript bundle
- `frontmatter: Option<FrontmatterData>` - Parsed frontmatter

**Methods:**

- `size()` - Get bundle size in bytes
- `has_frontmatter()` - Check if frontmatter exists

## Advanced Usage

### With Custom MDX Options

```rust
use joy_mdx::MdxCompileOptions;

let result = bundle_mdx(BundleMdxOptions {
    source: mdx.to_string(),
    files: HashMap::new(),
    mdx_options: None, // Uses sensible defaults (all features ON)
}).await?;
```

### Builder API

```rust
let result = bundle_mdx(
    BundleMdxOptions::new(mdx)
        .with_file("./utils.js", utils_code)
        .with_file("./Button.tsx", button_code)
        // mdx_options defaults to all features ON
).await?;
```

## Performance Considerations

### Caching Strategy

Runtime bundling has overhead. **Always cache results in production:**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Simple in-memory cache
type Cache = Arc<RwLock<HashMap<String, String>>>;

async fn get_bundled_mdx(mdx_id: &str, cache: &Cache) -> Result<String> {
    // Check cache
    {
        let read = cache.read().await;
        if let Some(cached) = read.get(mdx_id) {
            return Ok(cached.clone());
        }
    }

    // Not cached - bundle it
    let mdx_source = fetch_mdx_from_db(mdx_id).await?;
    let result = bundle_mdx(BundleMdxOptions::new(mdx_source)).await?;

    // Store in cache
    {
        let mut write = cache.write().await;
        write.insert(mdx_id.to_string(), result.code.clone());
    }

    Ok(result.code)
}
```

### Production Recommendations

- **Use Redis** or similar for distributed caching
- **Cache by content hash** to invalidate on changes
- **Add TTL** for automatic cache expiration
- **Rate limit** bundling requests
- **Monitor** bundle times and sizes

### Performance Benchmarks

Typical bundling times (M1 Mac, debug build):

- Basic MDX (no imports): ~15ms
- With 1 JSX import: ~25ms
- With 5 JSX imports: ~50ms
- Complex MDX + dependencies: ~100ms

## Architecture

```text
MDX source (from CMS/database)
    ↓
joy-mdx (compile MDX → JSX)
    ↓
Rolldown (bundle JSX + imports → single .js)
    ↓
Executable JavaScript string (send to client)
```

## Client-Side Integration

On the client, use a runtime to execute the bundled code:

```javascript
// Similar to mdx-bundler/client
import { getMDXComponent } from '@mdx-js/react';

function BlogPost({ code }) {
  const Component = getMDXComponent(code);
  return <Component />;
}
```

Or build your own runtime wrapper around the executable bundle.

## Examples

### Axum Web Server

```rust
use axum::{Router, routing::get, Json};
use joy_mdx_bundler::{bundle_mdx, BundleMdxOptions};

async fn get_post(id: String) -> Json<String> {
    let mdx = fetch_from_db(&id).await.unwrap();
    let result = bundle_mdx(BundleMdxOptions::new(mdx))
        .await
        .unwrap();
    Json(result.code)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/posts/:id", get(get_post));
    // ... run server
}
```

### With Frontmatter

```rust
let result = bundle_mdx(BundleMdxOptions::new(r#"
---
title: My Post
author: Jane Doe
---

# Content here
"#)).await?;

if let Some(fm) = result.frontmatter {
    println!("Title: {}", fm.raw);
}
```

## Development

```bash
# Build
cargo build -p joy-mdx-bundler

# Test
cargo test -p joy-mdx-bundler

# Check
cargo clippy -p joy-mdx-bundler
```

## Troubleshooting

### "No JavaScript output file generated"

- Ensure entry file has valid JSX syntax
- Check that all imports are provided in `files` map
- Verify temp directory has write permissions

### Large bundle sizes

- Bundle size depends on dependencies
- Use code splitting for large apps
- Consider build-time bundling (joy-core) instead

### Slow bundling

- Add caching (see Performance section)
- Use build-time bundling for static content
- Profile with `RUST_LOG=joy_mdx_bundler=debug`

## Related Projects

- [mdx-bundler](https://github.com/kentcdodds/mdx-bundler) - JavaScript equivalent
- [joy-core](../joy-core) - Build-time bundler
- [joy-mdx](../joy-mdx) - Standalone MDX compiler
- [Rolldown](https://rolldown.rs) - Underlying bundler

## License

MIT © [Joy Contributors](https://github.com/nine-gen/joy)
