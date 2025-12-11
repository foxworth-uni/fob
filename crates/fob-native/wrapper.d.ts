export * from './index';

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
