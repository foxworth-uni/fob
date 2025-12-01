/**
 * Global type declarations for .mdx modules
 *
 * This allows TypeScript to recognize .mdx file imports without requiring
 * users to create their own module declarations.
 */

import type { MDXContentProps } from './types';

declare module '*.mdx' {
  /**
   * MDX files export a default component that accepts MDXContentProps
   * and renders the MDX content.
   *
   * @example
   * ```tsx
   * import Article from './article.mdx';
   *
   * function Page() {
   *   return <Article />;
   * }
   * ```
   */
  const MDXComponent: (props: MDXContentProps) => JSX.Element;
  export default MDXComponent;
}
