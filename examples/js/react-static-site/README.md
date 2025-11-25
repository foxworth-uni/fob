# React Static Site Example

This example demonstrates how to build a simple React static site using the fob CLI in a tsup-like workflow.

## Quick Start

1. Install dependencies:

```bash
pnpm install
```

2. Start the development server:

```bash
pnpm dev
```

The dev server will start on `http://localhost:3000` (or the next available port) with hot module replacement (HMR).

3. Build for production:

```bash
pnpm build
```

This creates a production bundle in the `dist/` directory.

4. Build with minification:

```bash
pnpm build:prod
```

## Using fob CLI Directly

You can also use the fob CLI directly:

```bash
# Development server
../../../target/debug/fob-cli dev

# Production build
../../../target/debug/fob-cli build

# Production build with minification
../../../target/debug/fob-cli build --minify
```

## Comparison to tsup

This example follows a similar pattern to tsup:

- **tsup**: `tsup src/index.tsx --format esm --outDir dist`
- **fob**: `fob-cli build` (configured via `fob.config.json`)

The main differences:

- fob uses a config file (`fob.config.json`) instead of CLI flags
- fob includes a built-in dev server with HMR
- fob handles TypeScript/TSX automatically without additional configuration

## Project Structure

```
react-static-site/
├── package.json          # Dependencies and scripts
├── fob.config.json      # fob bundler configuration
├── index.html           # HTML template
├── README.md            # This file
└── src/
    ├── index.tsx        # Entry point (React 18 createRoot)
    └── App.tsx          # Main React component
```

## Features Demonstrated

- React 18 with `createRoot` API
- TypeScript/TSX support (no tsconfig needed)
- React Hooks (useState)
- Hot Module Replacement in dev mode
- Production bundling with source maps
- Modern ES modules
