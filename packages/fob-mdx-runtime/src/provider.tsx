'use client';

/**
 * MDXProvider component for React 19
 * Provides component overrides to MDX content via context
 */

import type { ReactElement } from 'react';
import { MDXContext, useMDXComponents } from './context.js';
import type { MDXComponents, MDXProviderProps } from './types.js';

/**
 * MDXProvider - provides component overrides to MDX content
 *
 * Features:
 * - Nested providers merge components (later overrides earlier)
 * - Supports function-based component merging
 * - Optional parent context bypass for performance
 *
 * @example
 * ```tsx
 * <MDXProvider components={{ h1: CustomH1 }}>
 *   <MDXContent />
 * </MDXProvider>
 * ```
 *
 * @example With function merger
 * ```tsx
 * <MDXProvider components={(parent) => ({ ...parent, h1: CustomH1 })}>
 *   <MDXContent />
 * </MDXProvider>
 * ```
 */
export function MDXProvider({
  components,
  children,
  disableParentContext = false,
}: MDXProviderProps): ReactElement {
  // Get parent components (unless disabled)
  const parentComponents = useMDXComponents(disableParentContext ? undefined : {});

  // Merge components
  let mergedComponents: MDXComponents;

  if (!components) {
    // No components provided, use parent
    mergedComponents = parentComponents;
  } else if (typeof components === 'function') {
    // Function merger - call with parent components
    mergedComponents = components(parentComponents);
  } else {
    // Object merge - later components override earlier
    mergedComponents = disableParentContext ? components : { ...parentComponents, ...components };
  }

  return <MDXContext value={{ components: mergedComponents }}>{children}</MDXContext>;
}

if (process.env.NODE_ENV !== 'production') {
  MDXProvider.displayName = 'MDXProvider';
}
