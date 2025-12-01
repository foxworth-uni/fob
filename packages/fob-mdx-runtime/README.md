# @fob/mdx-runtime

MDX v3-compatible runtime for Fob bundler with React 19 support.

## Features

- ✅ **React 19 automatic JSX runtime** - Uses the latest React JSX transform
- ✅ **MDXProvider** - Override components via React Context
- ✅ **Nested provider merging** - Later providers override earlier ones
- ✅ **Props spreading** - All components receive `{...props}` and children
- ✅ **TypeScript first** - Full type safety with exported types
- ✅ **Global .mdx module declarations** - Import MDX files with automatic TypeScript support
- ✅ **Zero configuration** - Works out of the box

## Installation

```bash
pnpm add @fob/mdx-runtime react
```

## Basic Usage

```tsx
import { MDXProvider } from '@fob/mdx-runtime';
import Content from './content.mdx'; // ✨ TypeScript knows this is an MDX component!

function App() {
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

### TypeScript Support

When you import from `@fob/mdx-runtime`, TypeScript automatically recognizes `.mdx` file imports:

```tsx
import Article from './article.mdx'; // ✅ Fully typed, no manual declarations needed

// Article is typed as: (props: MDXContentProps) => JSX.Element
```

No need to create a separate `mdx.d.ts` file - it's all handled for you!

## API

### `MDXProvider`

Provides component overrides to MDX content via React Context.

```tsx
interface MDXProviderProps {
  components?: MDXComponents | ((parent: MDXComponents) => MDXComponents);
  children?: ReactNode;
  disableParentContext?: boolean;
}
```

**Props:**

- `components` - Object of component overrides or function that merges with parent
- `children` - React children to render
- `disableParentContext` - Bypass parent context for performance (default: `false`)

**Example with function merger:**

```tsx
<MDXProvider components={(parent) => ({ ...parent, h1: CustomH1 })}>
  <Content />
</MDXProvider>
```

### `useMDXComponents`

Hook to access and merge MDX components from context.

```tsx
function useMDXComponents(
  components?: MDXComponents | ((contextComponents: MDXComponents) => MDXComponents)
): MDXComponents;
```

**Example:**

```tsx
import { useMDXComponents } from '@fob/mdx-runtime';

function MyComponent() {
  const components = useMDXComponents({ h1: CustomH1 });
  return <div>{/* use components */}</div>;
}
```

### TypeScript Types

```tsx
import type {
  MDXComponents,
  MDXComponentsMerger,
  MDXProviderProps,
  MDXContentProps,
  MDXContextValue,
} from '@fob/mdx-runtime';
```

## Advanced Features

### GFM (GitHub Flavored Markdown)

Automatically supported:

- Strikethrough: `~~deleted~~` → ~~deleted~~
- Tables with alignment
- Task lists: `- [x] Done`
- Autolinks: `https://example.com`

### Footnotes

```md
Here's a reference[^1] to a footnote.

[^1]: The footnote content.
```

### Math

Inline math: `$E = mc^2$` → $E = mc^2$

Block math:

```md
$$
\int_0^\infty x^2 dx
$$
```

### Custom Components

Override any HTML element:

```tsx
<MDXProvider
  components={{
    // Block elements
    h1: CustomH1,
    h2: CustomH2,
    h3: CustomH3,
    p: CustomParagraph,
    blockquote: CustomBlockquote,
    pre: CustomPre,
    code: CustomCode,

    // Inline elements
    a: CustomLink,
    strong: CustomStrong,
    em: CustomEmphasis,

    // Lists
    ul: CustomUL,
    ol: CustomOL,
    li: CustomLI,

    // Tables
    table: CustomTable,
    thead: CustomTHead,
    tbody: CustomTBody,
    tr: CustomTR,
    th: CustomTH,
    td: CustomTD,

    // Other
    hr: CustomHR,
    br: CustomBR,
    img: CustomImage,
    del: CustomDel, // GFM strikethrough
  }}
>
  <Content />
</MDXProvider>
```

## Nested Providers

Providers merge components from parent providers:

```tsx
<MDXProvider components={{ h1: ParentH1 }}>
  <MDXProvider components={{ p: ChildP }}>
    {/* Has both h1: ParentH1 and p: ChildP */}
    <Content />
  </MDXProvider>
</MDXProvider>
```

Later providers override earlier ones:

```tsx
<MDXProvider components={{ h1: FirstH1 }}>
  <MDXProvider components={{ h1: SecondH1 }}>
    {/* Only SecondH1 is used */}
    <Content />
  </MDXProvider>
</MDXProvider>
```

## SSR Support

Fully compatible with React 19 SSR and streaming:

```tsx
import { renderToString } from 'react-dom/server';
import { MDXProvider } from '@fob/mdx-runtime';
import Content from './content.mdx';

const html = renderToString(
  <MDXProvider components={{ h1: CustomH1 }}>
    <Content />
  </MDXProvider>
);
```

## Performance Tips

1. **Memoize component objects** to prevent unnecessary re-renders:

```tsx
const components = useMemo(() => ({ h1: CustomH1 }), []);
<MDXProvider components={components}>
```

2. **Use `disableParentContext`** if you don't need parent merging:

```tsx
<MDXProvider components={components} disableParentContext>
```

## Comparison with MDX v2

| Feature              | MDX v2               | @fob/mdx-runtime (v3)  |
| -------------------- | -------------------- | ---------------------- |
| React version        | 16.14+               | 19+                    |
| JSX runtime          | Classic or Automatic | Automatic only         |
| Fragment usage       | `<>`                 | `React.Fragment`       |
| List keys            | Manual or missing    | Auto-generated stable  |
| Props spreading      | Inconsistent         | Always spread          |
| Component resolution | Runtime lookup       | Compile-time optimized |
| TypeScript           | Types in @types      | Built-in               |

## License

MIT
