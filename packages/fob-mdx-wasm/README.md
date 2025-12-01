# bunny-wasm

WebAssembly bindings for bunny-mdx - compile MDX to JSX in the browser or Node.js.

This package provides TypeScript bindings for the bunny-wasm WebAssembly module, allowing you to compile MDX files to JSX without a bundler.

## Features

- ✅ **Compile MDX to JSX** - Full MDX v3 support
- ✅ **Browser & Node.js** - Works in both environments
- ✅ **Runtime evaluation** - `function-body` output format for `new Function()` eval
- ✅ **Extract frontmatter** - YAML and TOML frontmatter parsing
- ✅ **GFM support** - GitHub Flavored Markdown features
- ✅ **Math support** - LaTeX math expressions
- ✅ **Footnotes** - Markdown footnotes
- ✅ **No bundling** - Compile-only, WASM-compatible

## Installation

```bash
npm install bunny-wasm
# or
pnpm add bunny-wasm
# or
yarn add bunny-wasm
```

## Usage

### Browser

```typescript
import init, { compile_mdx, WasmMdxOptions } from 'bunny-wasm';

// Initialize WASM module (required in browser)
await init();

// Create compilation options
const options = new WasmMdxOptions();
options.set_gfm(true);
options.set_output_format('function-body'); // For runtime eval

const result = compile_mdx('# Hello **World**', options);
console.log(result.code);
```

### Node.js

```typescript
import { compile_mdx, WasmMdxOptions } from 'bunny-wasm';

// No init() needed in Node.js!
const options = new WasmMdxOptions();
options.set_gfm(true);

const result = compile_mdx('# Hello **World**', options);
console.log(result.code);
```

### Runtime Evaluation (Browser)

The `function-body` output format enables runtime evaluation with `new Function()`:

```typescript
import init, { compile_mdx, WasmMdxOptions } from 'bunny-wasm';
import * as jsxRuntime from 'react/jsx-runtime';

await init();

const options = new WasmMdxOptions();
options.set_output_format('function-body');

const result = compile_mdx('# Hello World', options);

// Evaluate the compiled code
const fn = new Function(result.code);
const module = fn({
  jsx: jsxRuntime.jsx,
  jsxs: jsxRuntime.jsxs,
  Fragment: jsxRuntime.Fragment,
});

// Render the component
const element = module.default({ components: {} });
```

## Output Formats

### `program` (default)

ES module format with `import`/`export` statements. Use with bundlers.

```javascript
import {jsx as _jsx, jsxs as _jsxs} from 'react/jsx-runtime';
export const frontmatter = {...};
export default function MDXContent({components}) { ... }
```

### `function-body`

Function body format for runtime evaluation. Use with `new Function()`.

```javascript
"use strict";
const {jsx: _jsx, jsxs: _jsxs, Fragment: _Fragment} = arguments[0];
const frontmatter = {...};
function MDXContent({components}) { ... }
return {default: MDXContent, frontmatter};
```

## API

### `init()` (Browser only)

Initialize the WASM module. Must be called before using any other functions in the browser.

```typescript
await init();
```

### `compile_mdx(source: string, options?: WasmMdxOptions): WasmMdxResult`

Compile MDX source code to JSX.

**Parameters:**

- `source` - MDX source code as string
- `options` - Optional compilation options

**Returns:**

- `WasmMdxResult` - Object containing compiled JSX and metadata

### `WasmMdxOptions`

Options for MDX compilation.

**Methods:**

- `new WasmMdxOptions()` - Create new options with defaults
- `set_filepath(path: string)` - Set filepath for error messages
- `set_gfm(enabled: boolean)` - Enable/disable GFM features
- `set_math(enabled: boolean)` - Enable/disable math expressions
- `set_footnotes(enabled: boolean)` - Enable/disable footnotes
- `set_jsx_runtime(runtime: string)` - Set JSX runtime (default: "react/jsx-runtime")
- `set_output_format(format: string)` - Set output format ("program" or "function-body")

**Properties:**

- `filepath: string | null` - Filepath for error messages
- `gfm: boolean` - GFM enabled flag
- `math: boolean` - Math enabled flag
- `footnotes: boolean` - Footnotes enabled flag
- `jsx_runtime: string` - JSX runtime string
- `output_format: string` - Output format ("program" or "function-body")

### `WasmMdxResult`

Result of MDX compilation.

**Properties:**

- `code: string` - Compiled JSX code
- `frontmatter: WasmFrontmatter | null` - Extracted frontmatter (if present)
- `images: string[]` - List of image URLs found in document
- `namedExports: string[]` - Named exports found in document
- `reexports: string[]` - Re-exports found in document
- `imports: string[]` - Imports found in document
- `defaultExport: string | null` - Default export name (if present)

### `WasmFrontmatter`

Frontmatter data extracted from MDX.

**Properties:**

- `raw: string` - Raw frontmatter string
- `format: 'yaml' | 'toml'` - Frontmatter format
- `data: object | null` - Parsed frontmatter data

## Building

```bash
# Build both browser and Node.js targets
pnpm build

# Build browser WASM only
pnpm build:wasm:web

# Build Node.js WASM only
pnpm build:wasm:node

# Build TypeScript
pnpm build:js
```

Or using just:

```bash
# Full build (browser + Node.js + TypeScript)
just wasm-build

# Browser only
just wasm-build-web

# Node.js only
just wasm-build-node

# Development build (browser only, unoptimized)
just wasm-build-dev
```

## Environment Support

### Browser

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

### Node.js

- Node.js 18+

## License

MIT
