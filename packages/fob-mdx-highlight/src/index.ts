/**
 * @fob/mdx-highlight
 *
 * Shiki-based syntax highlighting for fob MDX
 *
 * Features:
 * - Token-level accurate syntax highlighting
 * - Dual theme support (light/dark)
 * - CSS variable-based theming
 * - Line highlighting
 * - Word highlighting
 * - Zero client-side JavaScript
 *
 * @example
 * ```typescript
 * import { highlightCode } from '@fob/mdx-highlight';
 *
 * const result = await highlightCode(
 *   'const x = 42;',
 *   'typescript',
 *   'title="example.ts" {1}'
 * );
 *
 * console.log(result.html); // Pre-highlighted HTML
 * ```
 */

export {
  highlightCode,
  highlightCodeBlocks,
  disposeHighlighter,
  type HighlightOptions,
  type HighlightResult,
} from './shiki-transformer.js';

export { parseFenceMeta, type FenceMeta } from './meta-parser.js';

export {
  defaultThemeConfig,
  cssVariableTheme,
  cssVariables,
  type ThemeConfig,
} from './theme-config.js';
