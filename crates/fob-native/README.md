# @fox-uni/fob

High-performance JavaScript bundler powered by Rust using NAPI-RS.

## Installation

```bash
npm install @fox-uni/fob
# or
pnpm add @fox-uni/fob
# or
yarn add @fox-uni/fob
```

The package will automatically install the correct native binary for your platform.

## Supported Platforms

- **macOS**: x64, ARM64 (Apple Silicon)
- **Linux**: x64 (glibc & musl), ARM64 (glibc & musl)
- **Windows**: x64

## Usage

### Basic Bundle

```typescript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.js'],
  outputDir: './dist',
  format: 'esm',
  sourcemap: 'external',
  cwd: process.cwd(),
});

const result = await bundler.bundle();
console.log(`Built ${result.chunks.length} chunks in ${result.stats.durationMs}ms`);
```

### Quick Single-File Bundle

```typescript
import { bundleSingle } from '@fox-uni/fob';

const result = await bundleSingle('./src/index.js', './dist', 'esm');
```

## API

### `Fob` Class

#### Constructor

```typescript
new Fob(config: BundleConfig)
```

**BundleConfig:**

- `entries: string[]` - Entry points to bundle
- `outputDir?: string` - Output directory (default: `"dist"`)
- `format?: string` - Output format: `'esm'`, `'cjs'`, or `'iife'` (default: `'esm'`, case-insensitive)
- `sourcemap?: string` - Source map mode: `"external"`, `"inline"`, `"hidden"`, or `"false"` (default: disabled)
- `cwd?: string` - Working directory for resolution (default: `process.cwd()`)

#### Methods

##### `bundle(): Promise<BundleResult>`

Bundles the configured entries and returns detailed bundle information.

**Returns:** `BundleResult`

- `chunks: ChunkInfo[]` - Generated chunks
- `manifest: ManifestInfo` - Bundle manifest
- `stats: BuildStatsInfo` - Build statistics
- `assets: AssetInfo[]` - Static assets

### `bundleSingle` Function

Quick helper to bundle a single entry:

```typescript
bundleSingle(
  entry: string,
  outputDir: string,
  format?: string  // 'esm' | 'cjs' | 'iife'
): Promise<BundleResult>
```

### `version` Function

Returns the bundler version:

```typescript
version(): string
```

## Output Formats

All format strings are **case-insensitive** (`'esm'`, `'ESM'`, `'Esm'` all work).

- **`'esm'`** - ECMAScript Module format (default)
- **`'cjs'`** - CommonJS format
- **`'iife'`** - Immediately Invoked Function Expression format

## Source Map Options

- **`"external"`** or **`"true"`** - Generate external `.map` file
- **`"inline"`** - Embed source map inline as data URL
- **`"hidden"`** - Generate source map but don't link from output
- **`"false"`** or `undefined` - No source map generation (default)

## Features

- **Zero-copy Data Transfer**: Native performance with minimal overhead
- **Rich Bundle Information**: Detailed chunk, module, and statistics data
- **Type-safe**: Full TypeScript definitions included
- **Cross-platform**: Pre-built binaries for all major platforms
- **Production-ready**: Battle-tested error handling and security

## Error Handling

Errors are returned as structured JSON for easy debugging:

```typescript
try {
  await bundler.bundle();
} catch (error) {
  console.error(JSON.parse(error.message));
  // {
  //   type: "MissingExport",
  //   exportName: "Component",
  //   moduleId: "./src/component.js",
  //   ...
  // }
}
```

## Requirements

- Node.js >= 18.0.0

## License

MIT

## Contributing

See [GitHub repository](https://github.com/fox-uni/fob) for contribution guidelines.
