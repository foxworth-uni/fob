/**
 * TypeScript config helpers for programmatic Joy usage
 *
 * This module provides ergonomic helpers for creating BundleOptions from
 * partial configs, useful for library/SaaS use cases where config comes from
 * databases or APIs rather than files.
 */

import type { BundleOptions } from './types.js';

/**
 * Partial bundle options for ergonomic config creation
 *
 * All fields are optional - missing fields will use Joy defaults.
 */
export interface PartialBundleOptions {
  /** Entry point files (ESM modules) */
  entries?: string[];

  /** Output directory for generated chunks */
  output_dir?: string;

  /** Output format (currently only 'esm' supported) */
  format?: 'esm' | 'preserve-modules';

  /** Target platform */
  platform?: 'browser' | 'node' | 'worker' | 'deno';

  /** Enable code splitting for dynamic imports */
  code_splitting?: boolean;

  /** Enable minification */
  minify?: boolean;

  /** Source map generation */
  source_maps?: 'none' | 'inline' | 'external' | 'external-with-content';

  /** Shared chunk threshold in bytes (default: 20000) */
  shared_chunk_threshold?: number;

  /** External packages (not bundled) */
  external?: string[];

  /** Cache configuration */
  cache_config?: {
    enabled?: boolean;
    cache_dir?: string | null;
    max_size?: number;
  };

  /** Transform options */
  transform?: {
    typescript?: boolean;
    jsx?: boolean;
    target?: string;
    jsx_runtime?: 'classic' | 'automatic';
    jsx_import_source?: string | null;
    jsx_dev?: boolean;
    define?: Record<string, string>;
    enable_ssr_transform?: boolean;
    mode?: string;
    public_env?: Record<string, string>;
    minify?: boolean;
  };

  /** Path aliases for import resolution */
  path_aliases?: Record<string, string>;
}

/**
 * Create a complete BundleOptions from partial config
 *
 * Merges provided options with Joy defaults. Useful for programmatic bundling
 * where config comes from a database or API.
 *
 * @example
 * ```typescript
 * const config = createConfig({
 *   entries: ['index.mdx'],
 *   minify: true,
 *   platform: 'browser',
 * });
 *
 * const result = await bundleInMemory(files, config);
 * ```
 */
export function createConfig(partial: PartialBundleOptions): BundleOptions {
  return {
    entries: partial.entries || [],
    staticDir: null, // Only relevant for filesystem bundling
    outputDir: partial.output_dir || 'dist',
    format: partial.format || 'esm',
    platform: partial.platform || 'browser',
    codeSplitting: partial.code_splitting ?? true,
    minify: partial.minify ?? false,
    sourceMaps: partial.source_maps || 'external',
    sharedChunkThreshold: partial.shared_chunk_threshold ?? 20000,
    external: partial.external || [],
    experimental: {
      css: false,
      json: true,
      analysis: false,
    },
    plugins: [], // Plugins not supported in programmatic mode yet
    cacheConfig: {
      enabled: partial.cache_config?.enabled ?? true,
      cacheDir: partial.cache_config?.cache_dir ?? null,
      maxSize: partial.cache_config?.max_size ?? 0,
    },
    transform: {
      typescript: partial.transform?.typescript ?? true,
      jsx: partial.transform?.jsx ?? true,
      target: partial.transform?.target || 'ES2022',
      typeCheck: 'none',
      jsxRuntime: partial.transform?.jsx_runtime || 'automatic',
      jsxImportSource: partial.transform?.jsx_import_source || null,
      jsxDev: partial.transform?.jsx_dev ?? true,
      define: partial.transform?.define || {},
      enableSsrTransform: partial.transform?.enable_ssr_transform ?? false,
      mode: partial.transform?.mode || 'development',
      publicEnv: partial.transform?.public_env || {},
      minify: partial.transform?.minify ?? partial.minify ?? false,
    },
    typescriptConfig: null, // No tsconfig.json for in-memory bundling
    html: null, // No HTML generation for library use
    virtualModules: null,
    inlineTransforms: null,
    css: {
      enabled: true,
      modules: false,
      tailwind: null,
      postcssConfig: null,
    },
    pathAliases: partial.path_aliases || {},
  };
}

/**
 * Validate config schema (without filesystem checks)
 *
 * Performs basic validation like checking for empty entries.
 * Does NOT validate that files exist - use this for in-memory bundling.
 *
 * @throws Error if config is invalid
 */
export function validateConfig(config: BundleOptions): void {
  if (!config.entries || config.entries.length === 0) {
    throw new Error('Config validation failed: no entries specified');
  }

  for (const external of config.external || []) {
    if (!external || external.trim() === '') {
      throw new Error('Config validation failed: external package names cannot be empty');
    }
  }
}
