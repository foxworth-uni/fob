import type { Range } from './core.js';
import type { OperationResult } from './result.js';

/**
 * Options for find operations
 */
export interface FindOptions {
  /** Case sensitive search */
  caseSensitive?: boolean;
  /** Match whole words only */
  wholeWord?: boolean;
  /** Use regular expression */
  regex?: boolean;
}

/**
 * Result of a find operation
 */
export interface FindResult {
  /** Range where the match was found */
  range: Range;
  /** The matched text */
  text: string;
  /** Index of this result */
  index: number;
}

/**
 * Options for find-and-replace operations
 */
export interface FindReplaceOptions extends FindOptions {
  /** Replace all occurrences */
  all?: boolean;
}

/**
 * Result of a replace operation
 */
export interface ReplaceResult extends OperationResult {
  /** Number of replacements made */
  count: number;
}

/**
 * Search and replace API
 *
 * @remarks
 * Fully implemented using CodeMirror's SearchCursor and RegExpCursor.
 * Supports both string and RegExp queries with wrap-around searching.
 */
export interface SearchAPI {
  /**
   * Find all occurrences of a query
   */
  find(query: string | RegExp, options?: FindOptions): FindResult[];

  /**
   * Find the next occurrence and select it
   */
  findNext(query: string | RegExp, options?: FindOptions): FindResult | null;

  /**
   * Find the previous occurrence and select it
   */
  findPrevious(query: string | RegExp, options?: FindOptions): FindResult | null;

  /**
   * Replace occurrences of a query
   */
  replace(query: string | RegExp, replacement: string, options?: FindReplaceOptions): ReplaceResult;

  /**
   * Replace all occurrences
   */
  replaceAll(query: string | RegExp, replacement: string): ReplaceResult;
}
