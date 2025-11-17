# Fob Advanced Example

Advanced bundler patterns and features for experienced users.

**New to Fob?** Start with the [simple example](../fob-simple/) first!

## What This Demonstrates

- **Multiple bundler patterns**: functional API, class API, watch mode
- **Code splitting** with dynamic imports
- **Minification** with size comparison
- **Build statistics**: modules, chunks, sizes, duration, cache hit rate
- **File watching** for development workflow
- **Error handling** with structured error details
- **Advanced configurations**: multiple entry points, utilities

## Project Structure

```
fob-advanced/
├── package.json
├── README.md
├── bundler.js              # Basic bundling example
├── bundler-minified.js     # Production build with minification
├── bundler-watch.js        # File watcher with rebuild
└── src/
    ├── index.js            # Entry point
    ├── utils.js            # Utility functions
    ├── date-utils.js       # Date utilities
    └── heavy-module.js     # Code-split module
```

## Installation

From the repository root:

```bash
pnpm install
```

## Usage

### Basic Build

```bash
pnpm build
```

Bundles `src/index.js` and outputs to `dist/` with:
- ESM format
- External source maps
- Code splitting enabled

### Production Build (Minified)

```bash
pnpm build:minified
```

Compares unminified vs minified bundle sizes and outputs to `dist-prod/`.

### Watch Mode

```bash
pnpm build:watch
```

Watches `src/` directory and rebuilds on changes. Press `Ctrl+C` to stop.

## Key Features Demonstrated

### 1. Simple Bundle Function

```javascript
import { bundle } from '@fob/bundler';

const result = await bundle({
  entries: ['src/index.js'],
  outputDir: 'dist',
  format: 'esm',
  platform: 'node',
  sourceMaps: 'external',
  codeSplitting: true,
});
```

### 2. Reusable Bundler Instance

```javascript
import { Fob } from '@fob/bundler';

const bundler = new Fob({
  defaultOptions: {
    entries: ['src/index.js'],
    outputDir: 'dist',
  }
});

// Build with different options
await bundler.bundle({ minify: false });
await bundler.bundle({ minify: true });
```

### 3. Build Statistics

```javascript
console.log('Modules:', result.stats.totalModules);
console.log('Chunks:', result.stats.totalChunks);
console.log('Size:', result.stats.totalSize);
console.log('Duration:', result.stats.durationMs);
console.log('Cache hit rate:', result.stats.cacheHitRate);
```

### 4. Chunk Information

```javascript
for (const chunk of result.chunks) {
  console.log(chunk.fileName);     // e.g., "index-abc123.js"
  console.log(chunk.kind);         // "entry" | "async" | "shared"
  console.log(chunk.size);         // Size in bytes
  console.log(chunk.modules);      // Modules in this chunk
}
```

### 5. Error Handling

```javascript
try {
  await bundle(options);
} catch (error) {
  console.error(error.message);
  
  // Structured error details
  if (error.details) {
    switch (error.details.type) {
      case 'missing_export':
        console.log('Available:', error.details.available_exports);
        break;
      case 'circular_dependency':
        console.log('Cycle:', error.details.cycle_path);
        break;
    }
  }
}
```

## Configuration Options

See the examples for common options:

- `entries` - Entry point files
- `outputDir` - Output directory
- `format` - Output format (`esm` or `preserve-modules`)
- `platform` - Target platform (`node`, `browser`, `worker`, `deno`)
- `minify` - Enable minification
- `sourceMaps` - Source map generation (`none`, `inline`, `external`)
- `codeSplitting` - Enable code splitting for dynamic imports
- `external` - External dependencies to exclude from bundle

## Learn More

- [Fob Documentation](../../../docs/)
- [@fob/bundler Package](../../../packages/fob-bundler/)
- [@fob/edge Package](../../../packages/fob-edge/) - For edge runtime bundling

