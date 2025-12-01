/**
 * Shiki theme configuration for Joy MDX
 *
 * Uses CSS variables for seamless light/dark mode support
 */

import type { BundledTheme } from 'shiki';

export interface ThemeConfig {
  light: BundledTheme;
  dark: BundledTheme;
  defaultTheme?: 'light' | 'dark';
}

/**
 * Default theme configuration using GitHub themes
 * Maps to CSS variables for runtime theme switching
 */
export const defaultThemeConfig: ThemeConfig = {
  light: 'github-light',
  dark: 'github-dark',
  defaultTheme: 'light',
};

/**
 * CSS variable mapping for Shiki tokens
 *
 * This allows Shiki to output inline styles that reference CSS variables,
 * enabling runtime theme switching without re-rendering
 */
export const cssVariableTheme = {
  name: 'css-variables',
  type: 'dark' as const,
  colors: {
    'editor.foreground': 'var(--shiki-color-text)',
    'editor.background': 'var(--shiki-color-background)',
  },
  tokenColors: [
    {
      scope: ['comment', 'punctuation.definition.comment'],
      settings: {
        foreground: 'var(--shiki-token-comment)',
      },
    },
    {
      scope: ['string', 'string.quoted', 'string.template'],
      settings: {
        foreground: 'var(--shiki-token-string)',
      },
    },
    {
      scope: ['keyword', 'storage.type', 'storage.modifier'],
      settings: {
        foreground: 'var(--shiki-token-keyword)',
      },
    },
    {
      scope: ['entity.name.function', 'support.function'],
      settings: {
        foreground: 'var(--shiki-token-function)',
      },
    },
    {
      scope: ['constant.numeric', 'constant.language', 'constant.other'],
      settings: {
        foreground: 'var(--shiki-token-constant)',
      },
    },
    {
      scope: ['variable', 'support.variable'],
      settings: {
        foreground: 'var(--shiki-token-variable)',
      },
    },
    {
      scope: ['entity.name.type', 'entity.name.class', 'support.type', 'support.class'],
      settings: {
        foreground: 'var(--shiki-token-type)',
      },
    },
    {
      scope: ['entity.other.attribute-name'],
      settings: {
        foreground: 'var(--shiki-token-attribute)',
      },
    },
    {
      scope: ['meta.tag', 'punctuation.definition.tag'],
      settings: {
        foreground: 'var(--shiki-token-tag)',
      },
    },
  ],
};

/**
 * CSS variables for Shiki themes
 * Add to your global CSS or inject at runtime
 */
export const cssVariables = `
/* Shiki token colors - Light theme */
:root {
  --shiki-color-text: #24292f;
  --shiki-color-background: #ffffff;
  --shiki-token-comment: #6e7781;
  --shiki-token-string: #0a3069;
  --shiki-token-keyword: #cf222e;
  --shiki-token-function: #8250df;
  --shiki-token-constant: #0550ae;
  --shiki-token-variable: #24292f;
  --shiki-token-type: #953800;
  --shiki-token-attribute: #116329;
  --shiki-token-tag: #116329;
}

/* Shiki token colors - Dark theme */
@media (prefers-color-scheme: dark) {
  :root {
    --shiki-color-text: #c9d1d9;
    --shiki-color-background: #0d1117;
    --shiki-token-comment: #8b949e;
    --shiki-token-string: #a5d6ff;
    --shiki-token-keyword: #ff7b72;
    --shiki-token-function: #d2a8ff;
    --shiki-token-constant: #79c0ff;
    --shiki-token-variable: #ffa657;
    --shiki-token-type: #ffa657;
    --shiki-token-attribute: #7ee787;
    --shiki-token-tag: #7ee787;
  }
}

/* Manual theme overrides */
[data-theme='light'] {
  --shiki-color-text: #24292f;
  --shiki-color-background: #ffffff;
  --shiki-token-comment: #6e7781;
  --shiki-token-string: #0a3069;
  --shiki-token-keyword: #cf222e;
  --shiki-token-function: #8250df;
  --shiki-token-constant: #0550ae;
  --shiki-token-variable: #24292f;
  --shiki-token-type: #953800;
  --shiki-token-attribute: #116329;
  --shiki-token-tag: #116329;
}

[data-theme='dark'] {
  --shiki-color-text: #c9d1d9;
  --shiki-color-background: #0d1117;
  --shiki-token-comment: #8b949e;
  --shiki-token-string: #a5d6ff;
  --shiki-token-keyword: #ff7b72;
  --shiki-token-function: #d2a8ff;
  --shiki-token-constant: #79c0ff;
  --shiki-token-variable: #ffa657;
  --shiki-token-type: #ffa657;
  --shiki-token-attribute: #7ee787;
  --shiki-token-tag: #7ee787;
}
`;
