# Fob Simple Example

The simplest possible Fob bundler example. Perfect for getting started!

## What This Does

This example shows the **most basic** way to use Fob:
- Bundle a single JavaScript file
- Output as ESM format
- Display build results

## Quick Start

```bash
# Install dependencies
pnpm install

# Run the bundler
pnpm build
```

## Code Walkthrough

### bundler.js (The Bundler Script)

```javascript
import { bundle } from '@fob/bundler';

const result = await bundle({
  entries: ['src/index.js'],  // Files to bundle
  outputDir: 'dist',           // Where to put the output
  format: 'esm',               // Output format (ESM)
  sourceMaps: 'external',      // Generate source maps
});

// result contains:
// - chunks: Generated code files
// - stats: Build statistics
// - manifest: Entry point mappings
```

### src/index.js (Your Code)

Simple JavaScript with exports:

```javascript
export function greet(name) {
  return `Hello, ${name}!`;
}

console.log(greet('Fob'));
```

## Output

After running `pnpm build`, you'll see:

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

## What's Next?

Ready for more? Check out the **advanced example** to learn about:
- Code splitting
- Minification
- Watch mode
- Multiple entry points
- Error handling

See: `examples/js/fob-advanced/`

## API Reference

### bundle(options)

Simple bundling function.

**Options:**
- `entries: string[]` - Entry point files to bundle
- `outputDir: string` - Output directory (default: 'dist')
- `format: 'esm' | 'cjs'` - Output format (default: 'esm')
- `sourceMaps: 'none' | 'inline' | 'external'` - Source map generation

**Returns:** `BundleResult`
- `chunks` - Generated code chunks
- `stats` - Build statistics
- `manifest` - Entry point mappings
- `assets` - Static assets

## Learn More

- [Fob Documentation](../../README.md)
- [Advanced Example](../fob-advanced/)
