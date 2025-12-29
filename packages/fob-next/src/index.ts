/**
 * @fob/next
 * 
 * Next.js integration for fob MDX bundler
 * 
 * Provides React Server Component-compatible APIs for loading
 * and rendering MDX files with full component import support.
 */

export { bundleMdx, loadMdxModule, renderMdx } from "./loader.js";
export type {
  BundleMdxOptions,
  BundledMdxModule,
} from "./bundler.js";
export type { LoadMdxOptions } from "./loader.js";

// Re-export MDX runtime types for convenience
export type { MDXComponents, MDXContentProps } from "@fob/mdx-runtime";
export { MDXProvider } from "@fob/mdx-runtime";

