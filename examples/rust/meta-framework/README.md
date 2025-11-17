# Meta-Framework Example

This example demonstrates how to build a simple meta-framework using fob's Rust API.

## What This Demonstrates

This showcases the **meta-framework pattern** used by Next.js, Remix, and SvelteKit:

- **File-based routing**: Automatically discover routes by scanning `app/routes/`
- **Code splitting**: Each route becomes a separate entry point
- **Shared chunks**: Common code (like React) is automatically extracted
- **Path aliases**: Clean imports using `@` → `./app`
- **Multi-entry bundling**: Build multiple entry points in a single pass

## Project Structure

```
app/
├── routes/           # File-based routing
│   ├── index.tsx    # → / route
│   └── about.tsx    # → /about route
├── router.ts        # Framework runtime
├── server.ts        # Framework server
└── index.ts         # Public API
src/
└── main.rs          # Build script
dist/                # Generated output (multiple chunks)
```

## How It Works

1. **Route Discovery**: Scans `app/routes/` for `.tsx` files
2. **Multi-Entry Build**: Each route becomes a separate entry point
3. **Code Splitting**: Shared code extracted into common chunks
4. **Optimization**: Minification and tree-shaking applied

## Running the Example

### Build the Routes

```bash
cargo run
```

This will discover routes in `app/routes/` and bundle them into `dist/`.

### Run the Development Server

```bash
npm install
npm run dev
```

Or to just start the server (after building):

```bash
npm start
```

Then visit:
- **http://localhost:3000/** - Home page
- **http://localhost:3000/about** - About page

The server uses Hono to serve the SSR-rendered React components.

## Expected Output

```
🚀 Meta-Framework Builder

📁 Discovered 2 routes:
   • /
   • /about

🔨 Building with code splitting enabled...

📦 Generated 3 chunks:
   • about.js (1234 bytes)
   • index.js (1456 bytes)
   • shared-chunk.js (5678 bytes)

✅ Build complete! Output in: dist/
```

## Generated Artifacts

The build produces:

- **Route chunks**: `index.js`, `about.js` - One per route
- **Shared chunks**: Common code extracted automatically
- **ESM format**: Modern ES modules for tree-shaking

## Key Differences from Component Library Example

| Aspect | Component Library | Meta-Framework |
|--------|------------------|----------------|
| Entry points | Single library entry | Multiple route entries |
| Code splitting | Off | On (critical for routes) |
| Output | One bundle | Multiple chunks |
| Use case | Published package | Application framework |
| Path aliases | Less critical | Essential for DX |

## Meta-Framework Concepts

**File-based routing**: Convention over configuration - routes are discovered
automatically from the file system rather than manually registered.

**Code splitting**: Each route is loaded on-demand, improving initial page load.
Users only download the code they need for the current page.

**Shared chunks**: Common dependencies (React, utilities) are extracted into
separate chunks that are cached across page navigations.

This pattern allows frameworks to provide excellent developer experience
(just create a file to add a route) while maintaining optimal performance
through automatic optimization.
