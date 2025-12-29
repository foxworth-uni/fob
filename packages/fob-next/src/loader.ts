/**
 * Next.js App Router loader for MDX files
 *
 * Provides React Server Component-compatible API for loading MDX
 */

import { bundleMdx, BundleMdxOptions, BundledMdxModule } from './bundler.js';
import { cache } from 'react';
import type { MDXComponents } from '@fob/mdx-runtime';
import React from 'react';

export interface LoadMdxOptions extends Omit<BundleMdxOptions, 'filePath'> {
  /** Absolute path to MDX file */
  filePath: string;
}

// Re-export bundleMdx for convenience
export { bundleMdx };

/**
 * Load an MDX module (cached per request via React cache)
 *
 * This function is wrapped with React's cache() to ensure
 * the same MDX file is only bundled once per request.
 */
export const loadMdxModule = cache(async (options: LoadMdxOptions): Promise<BundledMdxModule> => {
  return bundleMdx(options);
});

/**
 * Render MDX content with component overrides
 *
 * Convenience wrapper that loads and renders MDX in one call
 */
export async function renderMdx(
  filePath: string,
  options?: {
    components?: MDXComponents;
    mdx?: LoadMdxOptions['mdx'];
    external?: string[];
    cwd?: string;
  }
): Promise<React.ReactElement> {
  const module = await loadMdxModule({
    filePath,
    ...(options?.mdx !== undefined && { mdx: options.mdx }),
    ...(options?.external !== undefined && { external: options.external }),
    ...(options?.cwd !== undefined && { cwd: options.cwd }),
  });

  const Content = module.default as React.ComponentType<{
    components?: MDXComponents;
  }>;

  const props: { components?: MDXComponents } = {};
  if (options?.components !== undefined) {
    props.components = options.components;
  }

  return React.createElement(Content, props);
}
