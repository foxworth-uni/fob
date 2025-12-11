# Fob Simple Python Example

The simplest possible Fob bundler example using Python bindings. Perfect for getting started!

## What This Does

This example shows the **most basic** way to use Fob from Python:

- Bundle a single JavaScript file
- Output as ESM format
- Display build results
- Use async/await patterns

## Prerequisites

- Python 3.8 or higher
- Rust toolchain (for building the Python extension)
- The fob-python crate built and installed

## Quick Start

```bash
# Build the Python extension (from project root)
cd ../../..
maturin develop --manifest-path crates/fob-python/Cargo.toml

# Run the bundler
cd examples/python/fob-simple
python bundler.py
```

## Code Walkthrough

### bundler.py (The Bundler Script)

```python
import asyncio
import fob

async def main():
    # Initialize logging (optional)
    fob.init_logging("info")

    # Bundle using the simple helper function
    result = await fob.bundle_single(
        entry="src/index.js",
        output_dir="dist",
        format="esm"
    )

    # result is a dict containing:
    # - chunks: Generated code files
    # - stats: Build statistics
    # - manifest: Entry point mappings
    # - assets: Static assets
```

### Using the Fob Class

For more control, use the `Fob` class:

```python
import asyncio
import fob
from pathlib import Path

async def main():
    # Create a bundler instance
    bundler = fob.Fob({
        "entries": ["src/index.js"],
        "output_dir": "dist",
        "format": "esm",
        "sourcemap": "external",
        "minify": False
    })

    # Bundle and get results
    result = await bundler.bundle()

    print(f"Bundled {result['module_count']} modules")
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

After running `python bundler.py`, you'll see:

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

```python
# Bundle a single entry (app mode)
result = await fob.Fob.bundle_entry(
    "src/index.ts",
    {"out_dir": "dist", "minify": True}
)

# Build a library (externalizes dependencies)
result = await fob.Fob.library(
    "src/index.ts",
    {"external": ["react", "react-dom"]}
)

# Build an app with code splitting
result = await fob.Fob.app(
    ["src/client.tsx", "src/worker.ts"],
    {"code_splitting": {"min_size": 20000, "min_imports": 2}}
)

# Build a component library
result = await fob.Fob.components(
    ["src/Button.tsx", "src/Card.tsx"],
    {"out_dir": "dist"}
)
```

### Using pathlib.Path

All path parameters accept both strings and `pathlib.Path` objects:

```python
from pathlib import Path

result = await fob.bundle_single(
    entry=Path("src/index.js"),
    output_dir=Path("dist"),
    format="esm"
)
```

## Error Handling

```python
import asyncio
import fob

async def main():
    try:
        result = await fob.bundle_single("src/index.js", "dist")
    except fob.FobError as e:
        print(f"Bundling failed: {e}")
        return

    print("Build successful!")
```

## What's Next?

Ready for more? Check out:

- **Advanced bundling**: Multiple entries, code splitting, minification
- **Library mode**: Externalize dependencies for npm packages
- **Component libraries**: Build UI component bundles
- **App mode**: Code splitting for web applications

## API Reference

### fob.bundle_single(entry, output_dir, format=None)

Quick helper to bundle a single entry.

**Parameters:**

- `entry: str | Path` - Entry file path
- `output_dir: str | Path` - Output directory
- `format: str | None` - Output format: "esm", "cjs", or "iife" (default: "esm")

**Returns:** `dict` - Bundle result

### fob.Fob(config)

Create a bundler instance with full configuration.

**Parameters:**

- `config: dict` - Configuration dictionary with keys:
  - `entries: list[str | Path]` - Entry point files
  - `output_dir: str | Path | None` - Output directory (default: "dist")
  - `format: str | None` - Output format (default: "esm")
  - `sourcemap: str | bool | None` - Source map mode
  - `platform: str | None` - Target platform: "browser" or "node"
  - `minify: bool | None` - Enable minification
  - `external: list[str] | str | None` - Packages to externalize
  - `cwd: str | Path | None` - Working directory

**Methods:**

- `bundle() -> dict` - Bundle configured entries

### fob.Fob.bundle_entry(entry, options=None)

Build a standalone bundle (single entry, full bundling).

### fob.Fob.library(entry, options=None)

Build a library (single entry, externalize dependencies).

### fob.Fob.app(entries, options=None)

Build an app with code splitting (multiple entries, unified output).

### fob.Fob.components(entries, options=None)

Build a component library (multiple entries, separate bundles).

## Learn More

- [Fob Documentation](../../../README.md)
- [Python API Documentation](../../../crates/fob-python/README.md)
