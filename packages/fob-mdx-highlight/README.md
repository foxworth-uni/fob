# @fob/mdx-highlight

Shiki-based syntax highlighting for Fob MDX with dual theme support and zero client-side JavaScript.

## Features

- 🎨 **Token-level accuracy** - Uses Shiki's TextMate grammars
- 🌗 **Dual themes** - Automatic light/dark mode support
- 🎯 **Line highlighting** - Highlight specific lines with `{1,3-5}`
- 📝 **Word highlighting** - Highlight tokens with `word:foo,bar`
- 🎭 **CSS variables** - Runtime theme switching without re-rendering
- ⚡ **Zero runtime cost** - All highlighting done at build time
- 🔧 **Extensible** - Custom themes and languages

## Installation

```bash
pnpm add @fob/mdx-highlight
```

## Usage

### Basic Highlighting

```typescript
import { highlightCode } from '@fob/mdx-highlight';

const result = await highlightCode('const greeting = "Hello, World!";', 'typescript');

console.log(result.html); // Pre-highlighted HTML
console.log(result.lang); // 'typescript'
```

### With Metadata

```typescript
const result = await highlightCode(
  `function calculate(a, b) {
  const sum = a + b;
  const product = a * b;
  return { sum, product };
}`,
  'javascript',
  'title="math.js" {2,3}'
);

// result.meta.title => "math.js"
// result.meta.highlightLines => [2, 3]
```

### Batch Highlighting

```typescript
import { highlightCodeBlocks } from '@fob/mdx-highlight';

const blocks = [
  { code: 'const x = 1;', lang: 'typescript' },
  { code: 'def hello():', lang: 'python' },
];

const results = await highlightCodeBlocks(blocks);
```

### Custom Themes

```typescript
import { highlightCode } from '@fob/mdx-highlight';

const result = await highlightCode(code, lang, meta, {
  theme: {
    light: 'github-light',
    dark: 'nord',
  },
});
```

## Fence Metadata Syntax

### Title

```markdown
\`\`\`typescript title="example.ts"
const x = 42;
\`\`\`
```

### Line Highlights

```markdown
\`\`\`typescript {1,3-5,7}
// Line 1 highlighted
const x = 1;
// Lines 3-5 highlighted
const y = 2;
const z = 3;
// Line 7 highlighted
const result = x + y + z;
\`\`\`
```

### Word Highlights

```markdown
\`\`\`javascript word:fetch,async
async function getData() {
const response = await fetch('/api');
return response.json();
}
\`\`\`
```

### Combined

```markdown
\`\`\`typescript title="api.ts" {2-3} word:fetch
async function fetchUser(id) {
const response = await fetch(\`/users/\${id}\`);
return response.json();
}
\`\`\`
```

## CSS Variables

Add the CSS variables to your global styles:

```typescript
import { cssVariables } from '@fob/mdx-highlight';

// Inject into <style> tag or CSS file
```

Or import the pre-made stylesheet:

```css
/* In your global CSS */
@import '@fob/mdx-highlight/themes.css';
```

## Supported Languages

Common languages are loaded by default:

- JavaScript/TypeScript (including JSX/TSX)
- HTML/CSS
- JSON/YAML
- Markdown
- Bash/Shell
- Python
- Rust
- Go
- Java
- C/C++
- SQL

### Additional Languages

```typescript
const result = await highlightCode(code, lang, meta, {
  additionalLangs: ['swift', 'kotlin', 'ruby'],
});
```

## API Reference

### `highlightCode(code, lang, meta?, options?)`

Highlight a single code block.

**Parameters:**

- `code: string` - The code to highlight
- `lang: string` - Programming language
- `meta?: string` - Fence metadata (title, line highlights, etc.)
- `options?: HighlightOptions` - Highlighting options

**Returns:** `Promise<HighlightResult>`

### `highlightCodeBlocks(blocks, options?)`

Highlight multiple code blocks efficiently.

**Parameters:**

- `blocks: Array<{ code, lang, meta? }>` - Array of code blocks
- `options?: HighlightOptions` - Highlighting options

**Returns:** `Promise<HighlightResult[]>`

### `parseFenceMeta(meta)`

Parse fence metadata string.

**Parameters:**

- `meta: string` - Metadata string from code fence

**Returns:** `FenceMeta`

## License

MIT
