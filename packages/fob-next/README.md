# @fob/next

Next.js integration for fob MDX bundler. Enables rendering MDX files with full component import support in Next.js App Router.

## Features

- ✅ **React Server Components** - Full RSC support for MDX rendering
- ✅ **Component Imports** - MDX files can import React components
- ✅ **Automatic Bundling** - Uses fob bundler with MDX plugin
- ✅ **Request Caching** - Cached per request via React's `cache()`
- ✅ **TypeScript** - Full type safety

## Installation

```bash
pnpm add @fob/next @fox-uni/fob @fob/mdx-runtime
```

## Usage

### Basic Example

```tsx
// app/page.tsx
import { loadMdxModule } from "@fob/next";
import path from "node:path";

export default async function Page() {
  const mod = await loadMdxModule({
    filePath: path.join(process.cwd(), "content/post.mdx"),
  });

  const Content = mod.default;

  return (
    <main>
      <Content />
    </main>
  );
}
```

### With Component Overrides

```tsx
// app/page.tsx
import { loadMdxModule } from "@fob/next";
import { MDXProvider } from "@fob/next";
import path from "node:path";

export default async function Page() {
  const mod = await loadMdxModule({
    filePath: path.join(process.cwd(), "content/post.mdx"),
  });

  const Content = mod.default;

  return (
    <MDXProvider
      components={{
        h1: (props) => <h1 className="text-4xl font-bold" {...props} />,
        p: (props) => <p className="my-4" {...props} />,
      }}
    >
      <Content />
    </MDXProvider>
  );
}
```

### Using renderMdx Helper

```tsx
// app/page.tsx
import { renderMdx } from "@fob/next";
import path from "node:path";

export default async function Page() {
  return (
    <main>
      {await renderMdx(path.join(process.cwd(), "content/post.mdx"), {
        components: {
          h1: (props) => <h1 className="text-4xl" {...props} />,
        },
      })}
    </main>
  );
}
```

### MDX with Component Imports

Your MDX files can import React components:

```mdx
// content/post.mdx
import { Callout } from "../components/Callout";

# My Post

<Callout type="info">
  This is a custom component!
</Callout>
```

The bundler will automatically resolve and bundle these imports.

## API

### `loadMdxModule(options)`

Load an MDX module. Cached per request.

**Options:**
- `filePath` (required) - Absolute path to MDX file
- `mdx` - MDX compilation options
  - `gfm?: boolean` - Enable GFM features (default: true)
  - `footnotes?: boolean` - Enable footnotes (default: true)
  - `math?: boolean` - Enable math (default: true)
  - `jsxRuntime?: string` - JSX runtime path (default: "react/jsx-runtime")
  - `useDefaultPlugins?: boolean` - Use default plugins (default: true)
- `external?: string[]` - External packages (default: ["react", "react-dom", "@fob/mdx-runtime"])
- `cwd?: string` - Working directory for resolution
- `cache?: boolean` - Enable caching (default: true)

**Returns:** `Promise<BundledMdxModule>` with `default` export and any named exports

### `renderMdx(filePath, options?)`

Convenience function to load and render MDX in one call.

**Parameters:**
- `filePath` - Absolute path to MDX file
- `options` - Same as `loadMdxModule` plus:
  - `components?: MDXComponents` - Component overrides

**Returns:** `Promise<React.ReactElement>`

### `bundleMdx(options)`

Lower-level API for bundling MDX without React cache.

## How It Works

1. **Bundling**: Uses `@fox-uni/fob` to bundle the MDX file with the MDX plugin
2. **Caching**: Results are cached per request using React's `cache()`
3. **Module Loading**: Compiled modules are imported dynamically
4. **Component Injection**: Uses `@fob/mdx-runtime` for component customization

## License

MIT

