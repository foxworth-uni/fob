/**
 * MDX v3-compatible type definitions for React 19
 */

import type { ComponentType, ReactNode } from 'react';

/**
 * Represents the standard HTML elements that can be overridden via MDXProvider
 */
export interface MDXComponents {
  // Block elements
  h1?: ComponentType<React.ComponentPropsWithoutRef<'h1'>>;
  h2?: ComponentType<React.ComponentPropsWithoutRef<'h2'>>;
  h3?: ComponentType<React.ComponentPropsWithoutRef<'h3'>>;
  h4?: ComponentType<React.ComponentPropsWithoutRef<'h4'>>;
  h5?: ComponentType<React.ComponentPropsWithoutRef<'h5'>>;
  h6?: ComponentType<React.ComponentPropsWithoutRef<'h6'>>;
  p?: ComponentType<React.ComponentPropsWithoutRef<'p'>>;
  blockquote?: ComponentType<React.ComponentPropsWithoutRef<'blockquote'>>;
  pre?: ComponentType<React.ComponentPropsWithoutRef<'pre'>>;
  code?: ComponentType<React.ComponentPropsWithoutRef<'code'>>;

  // List elements
  ul?: ComponentType<React.ComponentPropsWithoutRef<'ul'>>;
  ol?: ComponentType<React.ComponentPropsWithoutRef<'ol'>>;
  li?: ComponentType<React.ComponentPropsWithoutRef<'li'>>;

  // Table elements
  table?: ComponentType<React.ComponentPropsWithoutRef<'table'>>;
  thead?: ComponentType<React.ComponentPropsWithoutRef<'thead'>>;
  tbody?: ComponentType<React.ComponentPropsWithoutRef<'tbody'>>;
  tfoot?: ComponentType<React.ComponentPropsWithoutRef<'tfoot'>>;
  tr?: ComponentType<React.ComponentPropsWithoutRef<'tr'>>;
  th?: ComponentType<React.ComponentPropsWithoutRef<'th'>>;
  td?: ComponentType<React.ComponentPropsWithoutRef<'td'>>;

  // Inline elements
  a?: ComponentType<React.ComponentPropsWithoutRef<'a'>>;
  strong?: ComponentType<React.ComponentPropsWithoutRef<'strong'>>;
  em?: ComponentType<React.ComponentPropsWithoutRef<'em'>>;
  del?: ComponentType<React.ComponentPropsWithoutRef<'del'>>;

  // Other elements
  hr?: ComponentType<React.ComponentPropsWithoutRef<'hr'>>;
  br?: ComponentType<React.ComponentPropsWithoutRef<'br'>>;
  img?: ComponentType<React.ComponentPropsWithoutRef<'img'>>;

  // Code block with syntax highlighting
  CodeBlock?: ComponentType<CodeBlockProps>;

  // Index signature to allow arbitrary custom component names
  [key: string]: ComponentType<any> | undefined;
}

/**
 * Function that merges parent and child components
 * Later components override earlier ones
 */
export type MDXComponentsMerger = (parent: MDXComponents) => MDXComponents;

/**
 * Props accepted by MDXProvider
 */
export interface MDXProviderProps {
  /**
   * Components to make available to MDX content
   * Can be an object of components or a function that merges with parent components
   */
  components?: MDXComponents | MDXComponentsMerger;

  /**
   * Children to render within the provider
   */
  children?: ReactNode;

  /**
   * Disable context entirely (for performance in specific cases)
   */
  disableParentContext?: boolean;
}

/**
 * Props accepted by generated MDXContent component
 */
export interface MDXContentProps {
  /**
   * Components to override defaults
   */
  components?: MDXComponents;

  /**
   * Additional props passed to MDXContent
   */
  [key: string]: unknown;
}

/**
 * The shape of the MDX context value
 */
export interface MDXContextValue {
  components?: MDXComponents;
}

/**
 * Task list state: maps task IDs to checked status
 */
export interface TaskListState {
  [taskId: string]: boolean;
}

/**
 * Props for TaskListProvider component
 */
export interface TaskListProviderProps {
  children?: ReactNode;
  initialState?: TaskListState;
  onTaskToggle?: (taskId: string, checked: boolean, allState: TaskListState) => void;
}

/**
 * Options for task list persistence
 */
export interface PersistenceOptions {
  storageKey?: string;
  namespace?: string;
}

/**
 * Props for CodeBlock component
 */
export interface CodeBlockProps {
  /** Programming language (e.g., "typescript", "javascript") */
  lang: string;

  /** Raw code content */
  code: string;

  /** Optional title to display above code block */
  title?: string;

  /** Line numbers to highlight (1-indexed) */
  highlightLines?: number[];

  /** Words/tokens to highlight */
  highlightWords?: string[];

  /** Show line numbers */
  showLineNumbers?: boolean;

  /** Enable copy button */
  showCopyButton?: boolean;

  /** Custom class name */
  className?: string;

  /** Additional props */
  [key: string]: unknown;
}
