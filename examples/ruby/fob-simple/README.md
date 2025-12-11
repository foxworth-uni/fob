# Fob Simple Ruby Example

The simplest possible Fob bundler example using Ruby bindings. Perfect for getting started!

## What This Does

This example shows the **most basic** way to use Fob from Ruby:

- Bundle a single JavaScript file
- Output as ESM format
- Display build results
- Use Ruby-idiomatic patterns

## Prerequisites

- Ruby 2.7 or higher
- Rust toolchain (for building the Ruby extension)
- The fob-ruby crate built

## Quick Start

```bash
# Build the Ruby extension (from project root)
cd ../../..
cargo build --package fob-ruby

# Run the bundler
cd examples/ruby/fob-simple
ruby bundler.rb
```

## Code Walkthrough

### bundler.rb (The Bundler Script)

```ruby
require 'fob'

# Initialize logging (optional)
Fob.init_logging(:info)

# Bundle using the simple helper function
result = Fob.bundle_entry(
  'src/index.js',
  {
    out_dir: 'dist',
    format: :esm
  }
)

# result is a Hash containing:
# - chunks: Generated code files
# - stats: Build statistics
# - manifest: Entry point mappings
# - assets: Static assets
```

### Using the Fob::Bundler Class

For more control, use the `Fob::Bundler` class:

```ruby
require 'fob'

# Create a bundler instance
bundler = Fob::Bundler.new(
  entries: ['src/index.js'],
  out_dir: 'dist',
  format: :esm,
  sourcemap: 'external',
  minify: false
)

# Bundle and get results
result = bundler.bundle

puts "Bundled #{result[:module_count]} modules"
```

### src/index.js (Your Code)

Simple JavaScript with exports:

```javascript
export function greet(name) {
  return `Hello, ${name}!`;
}

export function add(a, b) {
  return a + b;
}

// Run some code when loaded
console.log(greet('Fob'));
console.log('2 + 3 =', add(2, 3));
```

## Output

After running `ruby bundler.rb`, you'll see:

```
🚀 Building with Fob...

✅ Build complete!

📦 Chunks generated:
  - index.js (123 bytes)

📊 Build stats:
  Modules: 1
  Total size: 123 bytes
  Duration: 45ms
```

And your bundled code will be in `dist/index.js`!

## API Examples

### Using Preset Methods

```ruby
# Bundle a single entry (app mode)
result = Fob.bundle_entry(
  'src/index.ts',
  { out_dir: 'dist', minify: true }
)

# Build a library (externalizes dependencies)
result = Fob.library(
  'src/index.ts',
  { external: ['react', 'react-dom'] }
)

# Build an app with code splitting
result = Fob.app(
  ['src/client.tsx', 'src/worker.ts'],
  { code_splitting: { min_size: 20_000, min_imports: 2 } }
)

# Build a component library
result = Fob.components(
  ['src/Button.tsx', 'src/Card.tsx'],
  { out_dir: 'dist' }
)
```

### Using Symbols for Enums

Ruby uses symbols for enum-like values:

```ruby
# Format options
result = Fob.bundle_entry('src/index.js', { format: :esm })   # ESM format
result = Fob.bundle_entry('src/index.js', { format: :cjs })  # CommonJS format
result = Fob.bundle_entry('src/index.js', { format: :iife })  # IIFE format

# Entry mode
bundler = Fob::Bundler.new(
  entries: ['src/index.js'],
  entry_mode: :shared    # Entries share chunks
)
bundler = Fob::Bundler.new(
  entries: ['src/index.js'],
  entry_mode: :isolated  # Each entry is separate
)

# Log levels
Fob.init_logging(:silent)  # No logging
Fob.init_logging(:error)   # Errors only
Fob.init_logging(:warn)    # Warnings and errors
Fob.init_logging(:info)    # Info, warnings, and errors (default)
Fob.init_logging(:debug)   # All logs
```

## Error Handling

```ruby
require 'fob'

begin
  result = Fob.bundle_entry('src/index.js', { out_dir: 'dist' })
rescue Fob::Error => e
  puts "Bundling failed: #{e.message}"
  return
end

puts "Build successful!"
```

## Configuration Options

### Basic Options

```ruby
Fob::Bundler.new(
  entries: ['src/index.js'],        # Required: entry point(s)
  out_dir: 'dist',                  # Output directory (default: "dist")
  format: :esm,                     # Output format: :esm, :cjs, or :iife
  sourcemap: 'external',            # Source map: true, false, 'inline', 'hidden', 'external'
  platform: 'browser',               # Target platform: 'browser' or 'node'
  minify: true,                      # Enable minification
  cwd: Dir.pwd                      # Working directory
)
```

### External Dependencies

```ruby
# Externalize specific packages
Fob::Bundler.new(
  entries: ['src/index.js'],
  external: ['react', 'react-dom']
)

# Externalize from package.json
Fob::Bundler.new(
  entries: ['src/index.js'],
  external_from_manifest: true
)
```

### Code Splitting

```ruby
Fob.app(
  ['src/client.tsx', 'src/worker.ts'],
  {
    code_splitting: {
      min_size: 20_000,      # Minimum chunk size in bytes
      min_imports: 2          # Minimum shared imports
    }
  }
)
```

### MDX Support

```ruby
Fob::Bundler.new(
  entries: ['src/post.mdx'],
  mdx: {
    gfm: true,                      # GitHub Flavored Markdown
    footnotes: true,                 # Footnotes support
    math: true,                      # Math support
    jsx_runtime: 'react/jsx-runtime', # JSX runtime
    use_default_plugins: true        # Use default plugins
  }
)
```

## What's Next?

Ready for more? Check out:

- **Advanced bundling**: Multiple entries, code splitting, minification
- **Library mode**: Externalize dependencies for npm packages
- **Component libraries**: Build UI component bundles
- **App mode**: Code splitting for web applications

## API Reference

### Fob.bundle_entry(entry, options = {})

Quick helper to bundle a single entry.

**Parameters:**

- `entry: String` - Entry file path
- `options: Hash` - Optional configuration:
  - `out_dir: String` - Output directory (default: "dist")
  - `format: Symbol` - Output format: `:esm`, `:cjs`, or `:iife` (default: `:esm`)
  - `sourcemap: String | Boolean` - Source map mode
  - `platform: String` - Target platform: "browser" or "node"
  - `minify: Boolean` - Enable minification
  - `external: Array<String>` - Packages to externalize
  - `cwd: String` - Working directory

**Returns:** `Hash` - Bundle result

### Fob::Bundler.new(config)

Create a bundler instance with full configuration.

**Parameters:**

- `config: Hash` - Configuration hash with keys:
  - `entries: Array<String>` - Entry point files
  - `out_dir: String | nil` - Output directory (default: "dist")
  - `format: Symbol | nil` - Output format (default: `:esm`)
  - `sourcemap: String | Boolean | nil` - Source map mode
  - `platform: String | nil` - Target platform: "browser" or "node"
  - `minify: Boolean | nil` - Enable minification
  - `external: Array<String> | nil` - Packages to externalize
  - `external_from_manifest: Boolean | nil` - Externalize from package.json
  - `entry_mode: Symbol` - Entry mode: `:shared` or `:isolated`
  - `code_splitting: Hash | nil` - Code splitting config
  - `mdx: Hash | nil` - MDX compilation options
  - `cwd: String | nil` - Working directory

**Methods:**

- `bundle() -> Hash` - Bundle configured entries

### Fob.bundle_entry(entry, options = {})

Build a standalone bundle (single entry, full bundling).

### Fob.library(entry, options = {})

Build a library (single entry, externalize dependencies).

### Fob.app(entries, options = {})

Build an app with code splitting (multiple entries, unified output).

### Fob.components(entries, options = {})

Build a component library (multiple entries, separate bundles).

### Fob.init_logging(level = :info)

Initialize logging with specified level.

**Parameters:**

- `level: Symbol` - Log level: `:silent`, `:error`, `:warn`, `:info`, or `:debug`

### Fob.init_logging_from_env

Initialize logging from `RUST_LOG` environment variable.

### Fob.version

Get the bundler version string.

## Learn More

- [Fob Documentation](../../../README.md)
- [Ruby API Documentation](../../../crates/fob-ruby/README.md)
