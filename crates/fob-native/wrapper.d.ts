export * from './index';
export { Entry, Entries } from './index';

/** Output format enum-like helper */
export declare const OutputFormat: {
  readonly Esm: 'esm';
  readonly Cjs: 'cjs';
  readonly Iife: 'iife';
};

/** Sourcemap mode helper */
export declare const SourceMapMode: {
  readonly External: 'external';
  readonly Inline: 'inline';
  readonly Hidden: 'hidden';
  readonly Disabled: 'false';
};

/**
 * Normalize flexible entries input to internal format.
 * Exported for testing and advanced usage.
 */
export declare function normalizeEntries(input: import('./index').Entries): {
  entries: string[];
  virtualFiles?: Record<string, string>;
};
