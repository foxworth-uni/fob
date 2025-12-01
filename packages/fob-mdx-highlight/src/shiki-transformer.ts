/**
 * Shiki-based syntax highlighting transformer for Joy MDX
 *
 * Transforms code blocks with accurate token-level syntax highlighting
 * using Shiki's language grammars.
 */

import { createHighlighter, type Highlighter, type BundledLanguage } from 'shiki';
import type { ThemeConfig } from './theme-config.js';
import { defaultThemeConfig } from './theme-config.js';
import { parseFenceMeta } from './meta-parser.js';

export interface HighlightOptions {
  /**
   * Theme configuration for light/dark mode
   */
  theme?: ThemeConfig;

  /**
   * Default language when none specified
   */
  defaultLang?: string;

  /**
   * Show line numbers by default
   */
  showLineNumbers?: boolean;

  /**
   * Additional languages to load (beyond common ones)
   */
  additionalLangs?: BundledLanguage[];
}

export interface HighlightResult {
  html: string;
  lang: string;
  meta: {
    title?: string;
    highlightLines: number[];
    highlightWords: string[];
  };
}

/**
 * Shiki highlighter instance (singleton)
 */
let highlighterInstance: Highlighter | null = null;

/**
 * Get or create the Shiki highlighter instance
 */
async function getHighlighter(options: HighlightOptions = {}): Promise<Highlighter> {
  if (highlighterInstance) {
    return highlighterInstance;
  }

  const { theme = defaultThemeConfig } = options;

  highlighterInstance = await createHighlighter({
    themes: [theme.light, theme.dark],
    langs: [
      'javascript',
      'typescript',
      'jsx',
      'tsx',
      'json',
      'html',
      'css',
      'markdown',
      'bash',
      'shell',
      'python',
      'rust',
      'go',
      'java',
      'c',
      'cpp',
      'sql',
      'yaml',
      ...(options.additionalLangs || []),
    ],
  });

  return highlighterInstance;
}

/**
 * Highlight code with Shiki
 *
 * @param code - The code to highlight
 * @param lang - Programming language
 * @param meta - Fence metadata string
 * @param options - Highlighting options
 * @returns Highlighted HTML with metadata
 */
export async function highlightCode(
  code: string,
  lang: string,
  meta: string = '',
  options: HighlightOptions = {}
): Promise<HighlightResult> {
  const highlighter = await getHighlighter(options);
  const { theme = defaultThemeConfig, defaultLang = 'text' } = options;

  // Parse metadata
  const parsedMeta = parseFenceMeta(meta);

  // Normalize language
  const normalizedLang = normalizeLang(lang || defaultLang);

  // Check if language is supported
  const supportedLangs = highlighter.getLoadedLanguages();
  const finalLang = supportedLangs.includes(normalizedLang as BundledLanguage)
    ? normalizedLang
    : 'text';

  // Generate highlighted HTML with dual themes
  const html = highlighter.codeToHtml(code, {
    lang: finalLang,
    themes: {
      light: theme.light,
      dark: theme.dark,
    },
    defaultColor: false,
    transformers: [
      {
        name: 'joy-mdx-highlight',
        pre(node) {
          // Add custom class to pre element
          this.addClassToHast(node, 'shiki');
        },
        code(node) {
          // Add language class
          this.addClassToHast(node, `language-${finalLang}`);
        },
        line(node, line) {
          // Add line highlighting
          if (parsedMeta.highlightLines.includes(line)) {
            this.addClassToHast(node, 'highlighted');
          }

          // Add line number attribute
          node.properties['data-line'] = line;
        },
      },
    ],
  });

  return {
    html,
    lang: finalLang,
    meta: parsedMeta,
  };
}

/**
 * Normalize language aliases to canonical names
 */
function normalizeLang(lang: string): string {
  const aliases: Record<string, string> = {
    js: 'javascript',
    ts: 'typescript',
    py: 'python',
    rb: 'ruby',
    sh: 'bash',
    yml: 'yaml',
    md: 'markdown',
    rs: 'rust',
  };

  return aliases[lang.toLowerCase()] || lang.toLowerCase();
}

/**
 * Batch highlight multiple code blocks
 *
 * More efficient than calling highlightCode multiple times
 */
export async function highlightCodeBlocks(
  blocks: Array<{ code: string; lang: string; meta?: string }>,
  options: HighlightOptions = {}
): Promise<HighlightResult[]> {
  // Pre-warm the highlighter
  await getHighlighter(options);

  // Highlight all blocks in parallel
  return Promise.all(
    blocks.map((block) => highlightCode(block.code, block.lang, block.meta, options))
  );
}

/**
 * Clean up the highlighter instance (for testing or rebuilds)
 */
export function disposeHighlighter(): void {
  highlighterInstance = null;
}
