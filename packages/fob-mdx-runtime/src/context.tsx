'use client';

/**
 * MDX Context implementation for React 19
 * Provides component overrides via React Context
 */

import { createContext, useContext } from 'react';
import type { MDXComponents, MDXContextValue } from './types.js';

/**
 * React context for MDX components
 * Uses React 19's createContext with proper typing
 */
export const MDXContext = createContext<MDXContextValue>({});

if (process.env.NODE_ENV !== 'production') {
  MDXContext.displayName = 'MDXContext';
}

/**
 * Hook to access MDX components from context
 * Merges components from all ancestor providers
 *
 * @param components - Additional components to merge with context
 * @returns Merged components object
 */
export function useMDXComponents(
  components?: MDXComponents | ((contextComponents: MDXComponents) => MDXComponents)
): MDXComponents {
  const contextValue = useContext(MDXContext);
  const contextComponents = contextValue.components ?? {};

  // If no components provided, return context components
  if (!components) {
    return contextComponents;
  }

  // If components is a function, call it with context components
  if (typeof components === 'function') {
    return components(contextComponents);
  }

  // Otherwise, merge objects (later components override earlier ones)
  return { ...contextComponents, ...components };
}
