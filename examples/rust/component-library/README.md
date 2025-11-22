# Component Library Example

This example demonstrates how to build a React component library using fob's Rust API.

## What This Example Shows

- **Library mode bundling**: Configured for npm package distribution
- **Peer dependency externalization**: React and React-DOM are not bundled
- **App mode bundling**: Demo app bundles all dependencies for browser
- **Multi-target builds**: Build both library and demo in one command
- **TypeScript declarations**: Auto-generated `.d.ts` files for type safety
- **Source maps**: Generated for debugging both library and demo
- **Real-world workflow**: Shows how to structure and build a complete project

## Project Structure

```
component-library/
├── components/
│   ├── index.ts      # Main entry exporting all components
│   ├── Button.tsx    # Button component (3 variants)
│   ├── Card.tsx      # Card component with title
│   └── Badge.tsx     # Badge component (4 variants)
├── demo/
│   ├── index.html    # Demo page HTML
│   └── app.tsx       # Interactive demo app
├── src/
│   └── main.rs       # Build script using fob-core API
├── server.js         # Development server
└── Cargo.toml        # Rust dependencies
```

## Running the Example

### Build Everything (Library + Demo)

```bash
cargo run
```

This single command builds **both**:

**Component Library** (`dist/`):

1. Bundles `components/index.ts` as the main entry point
2. Externalizes React and React-DOM (peer dependencies)
3. Generates TypeScript declarations from `.tsx` files
4. Outputs to `dist/` directory

**Demo App** (`demo/dist/`):

1. Bundles `demo/app.tsx` with all dependencies
2. Includes React (bundled for browser)
3. Transpiles JSX to JavaScript
4. Outputs to `demo/dist/` directory

This demonstrates **real-world workflow** - building multiple targets in one project!

### Run the Interactive Demo

```bash
npm install
npm run dev
```

Then visit **http://localhost:3001** to see the components in action!

The demo includes:

- Interactive Button component with click counter
- Card components with different content
- Badge components with all variants
- Usage examples and code snippets

## Generated Output

After running, you'll find in `dist/`:

```
dist/
├── index.js        # Bundled JavaScript (ESM format)
├── index.js.map    # Source map for debugging
└── index.d.ts      # TypeScript type declarations
```

## Key API Features Used

### Multi-Target Builds

This example shows how to build **multiple targets** in one project:

```rust
// Build the library (externalizes dependencies)
let result = BuildOptions::library("components/index.ts")
    .external(["react", "react-dom"])
    .runtime(runtime.clone())
    .build()
    .await?;

// Build the demo app (bundles everything)
let demo_result = BuildOptions::app(["demo/app.tsx"])
    .runtime(runtime)
    .outdir("demo/dist")
    .build()
    .await?;
```

### Library Mode

```rust
BuildOptions::library("components/index.ts")
```

Sets `bundle: false` to externalize all dependencies, which is standard for npm libraries.

### App Mode

```rust
BuildOptions::app(["demo/app.tsx"])
```

Bundles everything including dependencies, perfect for browser applications.

### External Dependencies

```rust
.external(["react", "react-dom"])
```

Explicitly marks React as a peer dependency. While library mode externalizes everything,
being explicit documents the intent and makes the build configuration clearer.

### Output Configuration

```rust
.outdir("dist")
.sourcemap(true)
```

Specifies where to write files and enables source map generation for better debugging.

### Writing to Disk

```rust
result.write_to_force("dist")?;
```

Writes all generated assets (JS, source maps, `.d.ts` files) to the output directory,
overwriting any existing files.

## Build Statistics

The example prints useful build metrics:

```
Modules analyzed: 3
Entry points: 1
Cache hits: 0/3
```

These statistics help monitor build performance and module graph size.

## Components Included

### Button

- **Variants**: `primary`, `secondary`, `danger`
- **Props**: `children`, `onClick`, `variant`, `disabled`
- Inline styles with hover effects

### Card

- **Props**: `title`, `children`
- Clean, bordered design with padding
- Perfect for content grouping

### Badge

- **Variants**: `success`, `warning`, `error`, `info`
- **Props**: `children`, `variant`
- Pill-shaped design with color coding

## Using the Library

After building, consumers can import the library:

```typescript
// Import everything
import { Button, Card, Badge } from 'your-lib';

// Use the components
<Button variant="primary" onClick={() => alert('Hello!')}>
  Click Me
</Button>

<Card title="My Card">
  <p>Card content</p>
</Card>

<Badge variant="success">New</Badge>
```

## Next Steps

To extend this example:

1. **Add more components**: Create new `.tsx` files in `components/`
2. **Multiple entry points**: Configure additional entries for better tree-shaking
3. **Different output formats**: Add CJS output alongside ESM
4. **Minification**: Enable `.minify(true)` for production builds
5. **Custom TypeScript config**: Use `.dts_outdir()` to customize declaration output
