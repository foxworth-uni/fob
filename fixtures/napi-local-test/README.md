# fob-native NAPI Local Testing

This example demonstrates how to test the `fob-native` NAPI bindings during local development. Perfect for the **Edit Rust → Rebuild → Test** workflow.

## Quick Start

```bash
# 1. Install dependencies
pnpm install

# 2. Build the native module
pnpm rebuild

# 3. Run all tests
pnpm test
```

## Development Workflow

### The Fast Loop

```bash
# Make changes to Rust code in crates/fob-native/src/

# Rebuild (debug mode, faster)
pnpm rebuild

# Test your changes
pnpm test
```

### Production Build

```bash
# Optimized release build
pnpm rebuild:release

# Test with release binary
pnpm test
```

## Available Scripts

| Script                 | Description                           |
| ---------------------- | ------------------------------------- |
| `pnpm rebuild`         | Fast debug build of the native module |
| `pnpm rebuild:release` | Optimized release build               |
| `pnpm test`            | Run all test suites                   |
| `pnpm test:simple`     | Basic API tests only                  |
| `pnpm test:advanced`   | Advanced feature tests                |
| `pnpm test:errors`     | Error handling tests                  |

## API Overview

The `fob-native` NAPI module exposes the following:

### Functions

#### `version(): string`

Returns the bundler version.

```javascript
import { version } from '@fox-uni/fob';
console.log(version()); // "0.1.0"
```

#### `bundleSingle(entry, outputDir, format?): Promise<BundleResult>`

Quick helper for single-entry bundling.

```javascript
import { bundleSingle } from '@fox-uni/fob';

const result = await bundleSingle('./src/index.js', './dist', 'esm');
console.log(result.outputPath); // "./dist/index.js"
```

### Classes

#### `Fob`

Main bundler class with full configuration.

```javascript
import { Fob } from '@fox-uni/fob';

const bundler = new Fob({
  entries: ['./src/index.js'],
  outputDir: './dist',
  format: 'esm',
  platform: 'browser',
  sourcemap: 'external',
  minify: false,
});

const result = await bundler.bundle();
console.log(`Bundled ${result.moduleCount} modules`);
```

### Configuration Values

#### Output Format (string, case-insensitive)

- `'esm'` - ES Modules (default)
- `'cjs'` - CommonJS
- `'iife'` - Immediately Invoked Function Expression

#### Source Map Mode (string)

- `'external'` or `'true'` - Separate .map file
- `'inline'` - Inline data URL
- `'hidden'` - Generate but don't reference
- `'false'` or `undefined` - No sourcemap (default)

## Configuration Options

```typescript
interface BundleConfig {
  entries: string[];              // Entry point files
  outputDir?: string;             // Output directory (default: "dist")
  format?: string;                // 'esm' | 'cjs' | 'iife' (default: 'esm')
  platform?: string;              // 'browser' | 'node' (default: 'browser')
  sourcemap?: string;             // 'external' | 'inline' | 'hidden' | 'false'
  minify?: boolean;               // Minify output
  external?: string[];            // External packages
  cwd?: string;                   // Working directory
}
```

## Error Handling

Errors from the native module are serialized as JSON:

```javascript
try {
  await bundleSingle('./nonexistent.js', './dist');
} catch (err) {
  const error = JSON.parse(err.message);
  console.log(error.kind); // "UnresolvedEntry"
  console.log(error.message); // "Cannot find entry..."
  console.log(error.file); // File path if applicable
  console.log(error.line); // Line number if applicable
}
```

### Error Types

- `UnresolvedEntry` - Entry file not found
- `ParseError` - Syntax error in source
- `InvalidConfig` - Invalid configuration
- `BundleError` - General bundling error

## Test Files

### test-simple.js

Basic functionality tests:

- Version checking
- Simple bundling (ESM/CJS)
- Basic Fob class usage
- Import resolution

### test-advanced.js

Advanced feature tests:

- Multiple entry points
- IIFE format
- Inline sourcemaps
- External sourcemaps
- Disabled sourcemaps
- Minification

### test-errors.js

Error handling tests:

- Non-existent files
- Syntax errors
- Invalid configuration
- Error JSON serialization

## Common Pitfalls

### ❌ Stale Native Binary

**Problem**: You made Rust changes but tests still use old code.
**Solution**: Always run `pnpm rebuild` after Rust changes.

### ❌ Platform Mismatch

**Problem**: `.node` file doesn't match your platform.
**Solution**: Rebuild on your platform: `pnpm rebuild`

### ❌ Path Resolution

**Problem**: Entry files not found.
**Solution**: Use absolute paths with `join(__dirname, 'file.js')`

### ❌ Not Awaiting Promises

**Problem**: Tests finish before bundling completes.
**Solution**: Always `await` the bundle operations

## Troubleshooting

### Binary Sync Issues

**Check if binary is stale:**

```bash
md5 ../../crates/fob-native/fob-native.darwin-arm64.node
md5 node_modules/.pnpm/@fox-uni+fob*/node_modules/*/fob-native.darwin-arm64.node
```

If MD5s differ:

```bash
pnpm rebuild  # Rebuilds and auto-syncs
```

The `sync-binary.js` script automatically runs after builds to keep node_modules in sync.

### Config Field Naming

**Important**: NAPI uses camelCase for JavaScript field names. Rust's `snake_case` is automatically converted.

- ✅ `outputDir` (correct - JavaScript camelCase)
- ❌ `output_dir` (wrong - will be ignored!)

Always use camelCase in JavaScript/TypeScript configs:

```javascript
const config = {
  entries: ['./src/index.js'],
  outputDir: './dist', // ✅ camelCase
  format: 'esm',       // ✅ string, case-insensitive
};
```

### Clean Test State

Before debugging, clean outputs:

```bash
rm -rf dist/
pnpm test
```

The `test-all.js` script automatically cleans the `dist/` directory before running tests.

### "Cannot find module '@fox-uni/fob'"

Run `pnpm install` in this directory to link the native package.

### "Error loading shared library"

The native module wasn't built. Run `pnpm rebuild`.

### "Permission denied"

On macOS, you might need to allow the binary: System Preferences → Security & Privacy

### Tests failing after Rust changes

1. Rebuild: `pnpm rebuild`
2. Clear dist: `rm -rf dist/`
3. Run tests: `pnpm test`

### Output Directory Not Working

If files aren't being written to the configured `outputDir`:

1. **Check field name**: Use `outputDir` (camelCase), not `output_dir`
2. **Check debug logs**: The bundler prints debug info to stderr:
   ```
   [BUNDLER DEBUG] config.output_dir = Some("/path/to/dist")
   [BUNDLER DEBUG] Using output_dir from config: /path/to/dist
   ```
3. **Verify path**: Use absolute paths or paths relative to `cwd`:
   ```javascript
   const config = {
     entries: [join(__dirname, 'src/index.js')],
     outputDir: join(__dirname, 'dist'), // ✅ Absolute path
     cwd: __dirname,
   };
   ```

## Project Structure

```
napi-local-test/
├── package.json           # Links to fob-native package
├── test-simple.js         # Basic tests
├── test-advanced.js       # Advanced tests
├── test-errors.js         # Error handling tests
├── test-all.js           # Test orchestrator
├── fixtures/             # Test input files
│   ├── simple/
│   ├── multi-entry/
│   ├── with-import/
│   └── error-case/
└── dist/                 # Test output (gitignored)
```

## Integration with CI

This example can be added to your CI pipeline:

```yaml
- name: Build native module
  run: pnpm build:napi

- name: Test NAPI bindings
  working-directory: examples/js/napi-local-test
  run: pnpm install && pnpm test
```

## Next Steps

- Add more complex bundling scenarios
- Test watch mode if available
- Benchmark performance
- Test platform-specific features

---

**Note**: This example uses the debug build by default for faster iteration. For production testing, use `pnpm rebuild:release`.
