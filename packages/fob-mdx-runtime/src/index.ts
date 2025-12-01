/**
 * @fob/mdx-runtime
 *
 * MDX v3-compatible runtime for fob with React 19 support
 *
 * Features:
 * - MDXProvider for component overrides
 * - useMDXComponents hook for accessing and merging components
 * - Full TypeScript support with global .mdx module declarations
 * - React 19 automatic JSX runtime
 * - Nested provider merging
 *
 * @example
 * ```tsx
 * import { MDXProvider } from '@fob/mdx-runtime';
 *
 * function App() {
 *   return (
 *     <MDXProvider components={{ h1: CustomH1 }}>
 *       <MDXContent />
 *     </MDXProvider>
 *   );
 * }
 * ```
 */

export { MDXProvider } from './provider.js';
export { MDXContext, useMDXComponents } from './context.js';
export { TaskListProvider, useTaskListContext } from './TaskListContext.js';
export { useTaskList } from './useTaskList.js';
export { loadTaskState, saveTaskState, clearTaskState } from './TaskListPersistence.js';
export { CodeBlock } from './CodeBlock.js';
export type {
  MDXComponents,
  MDXComponentsMerger,
  MDXProviderProps,
  MDXContentProps,
  MDXContextValue,
  TaskListState,
  TaskListProviderProps,
  PersistenceOptions,
  CodeBlockProps,
} from './types.js';
