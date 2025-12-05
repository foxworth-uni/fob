# Target-Based Build Configuration

This document explains the target-based API for configuring builds in fob-bundler, which addresses the problem of different deployment targets requiring fundamentally different build configurations.

## The Problem

When building for different deployment targets, you need very different configurations:

| Target             | Platform | Format | Bundle | External           |
| ------------------ | -------- | ------ | ------ | ------------------ |
| Browser            | Browser  | ESM    | true   | none               |
| Vercel Serverless  | Node     | ESM    | true   | node builtins only |
| Cloudflare Workers | Browser  | ESM    | true   | none               |
| SSR Pages          | Node     | ESM    | true   | node builtins only |
| Library (npm)      | Node     | ESM    | false  | all deps           |

Previously, developers had to remember these combinations and manually configure them. This led to:

1. **Misconfiguration errors**: Accidentally using browser settings for Node.js targets
2. **Verbose code**: Repeating the same 5-6 configuration lines for every build
3. **Missing externals**: Forgetting to externalize Node.js builtins in serverless functions
4. **Inconsistency**: Different projects using slightly different configurations

## The Solution: Type-Safe Target Presets

We introduce phantom-typed target presets that:

1. **Encode constraints in types**: Can't accidentally mix browser and Node.js settings
2. **Provide smart defaults**: One line gives you the right configuration
3. **Remain flexible**: Can still customize everything after using a preset
4. **Are self-documenting**: `target::serverless()` is clearer than 5 config lines

### Architecture

```rust
// Phantom type markers (zero runtime cost)
pub struct Browser;
pub struct Serverless;
pub struct Worker;
pub struct Ssr;
pub struct Library;

// Target configuration with phantom type parameter
pub struct Target<T> {
    platform: Platform,
    format: OutputFormat,
    bundle: bool,
    external: Vec<String>,
    minify_default: bool,
    _marker: PhantomData<T>,  // Zero-sized, compile-time only
}
```

The `T` parameter exists only at compile time. It has zero runtime cost but provides:

- **Documentation**: The type tells you what kind of build this is
- **API guidance**: IDE autocomplete knows what methods make sense
- **Future extensibility**: We can add target-specific methods later

## Usage Examples

### 1. Vercel Serverless Function

```rust
use fob_bundler::{target, BuildOptions};

let result = BuildOptions::target(target::serverless())
    .entry("./api/users.ts")
    .outdir("./.vercel/output/functions/api/users.func")
    .external(["@prisma/client"])  // Add custom externals if needed
    .build()
    .await?;
```

**What it does:**

- Sets `platform: Node`
- Sets `format: ESM`
- Sets `bundle: true`
- Automatically externalizes all Node.js builtins (`fs`, `path`, `http`, etc.)
- Allows adding custom externals like database drivers

### 2. Cloudflare Workers

```rust
let result = BuildOptions::target(target::worker())
    .entry("./src/worker.ts")
    .outfile("./dist/_worker.js")
    .build()
    .await?;
```

**What it does:**

- Sets `platform: Browser` (Workers use V8, not Node.js)
- Sets `format: ESM`
- Sets `bundle: true`
- Enables minification by default (smaller = faster cold starts)
- No externals (everything must be bundled)

### 3. Browser Client Bundle

```rust
let result = BuildOptions::target(target::browser())
    .entry("./src/client.tsx")
    .outdir("./dist/public")
    .splitting(true)
    .minify_level("identifiers")
    .build()
    .await?;
```

**What it does:**

- Sets `platform: Browser`
- Sets `format: ESM`
- Sets `bundle: true`
- No externals (everything bundled)
- Allows code splitting for multi-page apps

### 4. SSR Page (Server-Side Rendering)

```rust
let result = BuildOptions::target(target::ssr())
    .entry("./routes/blog/[slug].tsx")
    .outdir("./.vercel/output/functions/blog/[slug].func")
    .build()
    .await?;
```

**What it does:**

- Sets `platform: Node`
- Sets `format: ESM`
- Sets `bundle: true` (bundles React and UI libraries)
- Externalizes Node.js builtins (but NOT React/UI libs)

### 5. Library (npm package)

```rust
let result = BuildOptions::target(target::library())
    .entry("./src/index.ts")
    .outdir("./dist")
    .emit_dts(true)
    .build()
    .await?;
```

**What it does:**

- Sets `platform: Node`
- Sets `format: ESM`
- Sets `bundle: false` (deps are peer dependencies)
- Generates TypeScript declarations if enabled

## Node.js Builtins Externalization

One of the key features is automatic externalization of Node.js builtin modules for serverless and SSR targets.

### Why Externalize Node Builtins?

1. **Size**: Builtins are provided by the runtime, no need to include them
2. **Correctness**: Some builtins can't be polyfilled (like native addons)
3. **Performance**: Runtime builtins are optimized native code

### What Gets Externalized

The serverless and SSR targets automatically externalize:

- Core I/O: `fs`, `fs/promises`, `path`
- Networking: `http`, `https`, `http2`, `net`, `dns`
- Crypto: `crypto`
- Streams: `stream`, `stream/promises`, `stream/web`
- Process: `process`, `child_process`
- And 40+ more Node.js core modules

Both prefixed (`node:fs`) and unprefixed (`fs`) forms are included because:

- Modern code uses `node:` prefix (explicit, recommended)
- Legacy code uses unprefixed imports (still common)
- Both work in Node.js 18+

## Type Safety

The phantom type parameter prevents certain classes of errors:

```rust
// This works - serverless target gets Node platform
let opts = BuildOptions::target(target::serverless())
    .entry("./api/handler.ts")
    .outdir("./functions");

assert_eq!(opts.platform, Platform::Node);
```

While you _can_ still change `platform` after the fact (it's a public field), the target preset gives you the right starting point, and IDE autocomplete will guide you toward correct configurations.

## Customization

Presets provide smart defaults, but everything remains customizable:

```rust
let result = BuildOptions::target(target::serverless())
    .entry("virtual:handler")
    .virtual_file("virtual:handler", "export default (req) => ({ ok: true })")
    .path_alias("@", "./src")
    .external(["better-sqlite3"])  // Add to Node builtins
    .minify_level("syntax")         // Override minification
    .sourcemap(true)
    .build()
    .await?;
```

## Migration Guide

### Before (Verbose, Error-Prone)

```rust
let opts = BuildOptions::new("./api/users.ts")
    .platform(Platform::Node)
    .format(OutputFormat::Esm)
    .bundle(true)
    .external([
        "fs", "path", "http", "https", "crypto", "stream", // ...100 more
    ])
    .outdir("./functions");
```

Problems:

- Must remember all the settings
- Easy to forget Node builtins
- Repetitive across many functions
- Not immediately clear this is a serverless build

### After (Concise, Correct)

```rust
let opts = BuildOptions::target(target::serverless())
    .entry("./api/users.ts")
    .outdir("./functions");
```

Benefits:

- One line encodes the target environment
- Node builtins handled automatically
- Self-documenting (clearly a serverless function)
- Still allows customization when needed

## Design Tradeoffs

### What We Chose

**Phantom types over enums**: We use `Target<Browser>`, `Target<Serverless>`, etc. rather than an enum like `TargetKind::Browser`.

**Pros:**

- Zero runtime cost (phantom types compile away)
- Type-level documentation
- Extensible (can add target-specific methods later)
- Familiar Rust pattern (like `PhantomData` itself)

**Cons:**

- Slightly more complex type signatures
- Can't pattern match on target type at runtime (but we don't need to)

### What We Didn't Choose

**Builder with `.preset(Preset::Serverless)`**: We considered this but it's less ergonomic:

```rust
// Rejected approach
BuildOptions::new(entry)
    .preset(Preset::Serverless)  // Extra line, easy to forget
    .build()
```

**Helper functions in gumbo-deploy**: We could have put these in gumbo-deploy instead of fob-bundler:

```rust
// Rejected: Target-specific configuration in gumbo-deploy
fn serverless_options(entry: &Path) -> BuildOptions {
    BuildOptions::new(entry)
        .platform(Platform::Node)
        .external(all_node_builtins())
        // ...
}
```

**Why we didn't:** This is useful for ANY framework, not just Gumbo. By putting it in fob-bundler, other projects can benefit.

## Testing

The target system includes comprehensive tests:

1. **Unit tests**: Verify each target has correct defaults
2. **Integration tests**: Test targets with BuildOptions API
3. **Property tests**: Ensure Node builtins list has no duplicates
4. **Compile-fail tests**: Document type safety (as doc comments)

Run tests:

```bash
cd crates/fob-bundler
cargo test target
```

## Future Enhancements

Potential additions (not implemented yet):

1. **Target-specific methods**: Add methods that only make sense for certain targets

   ```rust
   impl Target<Worker> {
       pub fn compatibility_date(self, date: &str) -> Self { ... }
   }
   ```

2. **Validation**: Warn if configuration doesn't match target semantics

   ```rust
   // Warn: serverless usually doesn't need code splitting
   BuildOptions::target(target::serverless())
       .entry("./api/handler.ts")
       .splitting(true)  // Warning: splitting rarely useful for serverless
   ```

3. **More targets**: Add presets for other platforms
   - `target::deno()` - Deno Deploy
   - `target::bun()` - Bun runtime
   - `target::netlify()` - Netlify Functions

4. **Profile-based optimization**: Production vs development settings
   ```rust
   BuildOptions::target(target::worker())
       .entry("./worker.ts")
       .profile(Profile::Production)  // Aggressive minification
   ```

## Summary

The target-based API provides:

- **Correctness**: Right defaults for each platform
- **Ergonomics**: One line instead of five
- **Safety**: Type system prevents some misconfigurations
- **Flexibility**: Can still customize everything
- **Reusability**: Other frameworks can use these presets

For the common case (deploying to Vercel, Cloudflare, etc.), you get correct builds with minimal code. For advanced cases, full customization remains available.
