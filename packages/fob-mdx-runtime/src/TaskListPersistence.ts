/**
 * Helper utilities for persisting task list state
 * Provides localStorage integration for task state persistence
 */

import type { TaskListState } from './TaskListContext.js';

export interface PersistenceOptions {
  /**
   * Storage key prefix for task state
   * @default 'mdx-tasks'
   */
  storageKey?: string;

  /**
   * Optional namespace for multiple MDX documents
   * Useful when you have multiple MDX files and want separate state
   */
  namespace?: string;
}

/**
 * Get the full storage key with optional namespace
 */
function getStorageKey(options: PersistenceOptions = {}): string {
  const { storageKey = 'mdx-tasks', namespace } = options;
  return namespace ? `${storageKey}:${namespace}` : storageKey;
}

/**
 * Load task state from localStorage
 *
 * @param options - Persistence configuration
 * @returns Task state object, or empty object if not found or error
 *
 * @example
 * ```tsx
 * const initialState = loadTaskState({ namespace: 'readme' });
 * <TaskListProvider initialState={initialState}>
 *   <MDXContent />
 * </TaskListProvider>
 * ```
 */
export function loadTaskState(options: PersistenceOptions = {}): TaskListState {
  if (typeof window === 'undefined' || !window.localStorage) {
    return {};
  }

  try {
    const key = getStorageKey(options);
    const stored = window.localStorage.getItem(key);

    if (!stored) {
      return {};
    }

    const parsed = JSON.parse(stored);

    // Validate the structure
    if (typeof parsed !== 'object' || parsed === null) {
      return {};
    }

    return parsed as TaskListState;
  } catch (error) {
    console.warn('Failed to load task state from localStorage:', error);
    return {};
  }
}

/**
 * Save task state to localStorage
 *
 * @param state - Task state to save
 * @param options - Persistence configuration
 *
 * @example
 * ```tsx
 * <TaskListProvider
 *   onTaskToggle={(id, checked, allState) => {
 *     saveTaskState(allState, { namespace: 'readme' });
 *   }}
 * >
 *   <MDXContent />
 * </TaskListProvider>
 * ```
 */
export function saveTaskState(state: TaskListState, options: PersistenceOptions = {}): void {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }

  try {
    const key = getStorageKey(options);
    window.localStorage.setItem(key, JSON.stringify(state));
  } catch (error) {
    console.warn('Failed to save task state to localStorage:', error);
  }
}

/**
 * Clear task state from localStorage
 *
 * @param options - Persistence configuration
 *
 * @example
 * ```tsx
 * // Clear specific namespace
 * clearTaskState({ namespace: 'readme' });
 *
 * // Clear default state
 * clearTaskState();
 * ```
 */
export function clearTaskState(options: PersistenceOptions = {}): void {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }

  try {
    const key = getStorageKey(options);
    window.localStorage.removeItem(key);
  } catch (error) {
    console.warn('Failed to clear task state from localStorage:', error);
  }
}
