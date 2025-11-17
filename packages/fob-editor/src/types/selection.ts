import type { Range } from './core.js';
import type { OperationResult } from './result.js';

/**
 * Selection in the editor
 * Represents a range with anchor (start) and head (end/cursor)
 */
export interface Selection {
  /** The anchor point of the selection */
  anchor: number;
  /** The head (cursor) point of the selection */
  head: number;
  /** The start of the selection (min of anchor/head) */
  from: number;
  /** The end of the selection (max of anchor/head) */
  to: number;
}

/**
 * Cursor position with additional metadata
 */
export interface Cursor {
  /** Absolute offset in the document */
  offset: number;
  /** Line number (0-indexed) */
  line: number;
  /** Column number (0-indexed) */
  column: number;
}

/**
 * Direction for cursor movement
 */
export type CursorDirection = 'up' | 'down' | 'left' | 'right';

/**
 * Selection and cursor manipulation API
 */
export interface SelectionAPI {
  // ========================================================================
  // QUERIES
  // ========================================================================

  /** Get the current selection */
  get(): Selection;

  /** Get the current cursor position */
  getCursor(): Cursor;

  /** Get the currently selected text */
  getSelectedText(): string;

  /** Check if there is an active selection */
  hasSelection(): boolean;

  // ========================================================================
  // MUTATIONS
  // ========================================================================

  /** Set the selection to a specific range */
  set(from: number, to: number): OperationResult;

  /** Set the cursor to a specific position */
  setCursor(position: number): OperationResult;
  setCursor(line: number, column: number): OperationResult;

  /** Move the cursor in a direction */
  moveCursor(direction: CursorDirection, amount?: number): OperationResult;

  /** Select all content */
  selectAll(): OperationResult;

  /** Select a specific range */
  selectRange(range: Range): OperationResult;

  /** Select a specific line */
  selectLine(lineNumber: number): OperationResult;

  /** Select multiple lines */
  selectLines(fromLine: number, toLine: number): OperationResult;

  /** Collapse selection to a single cursor position */
  collapse(toStart?: boolean): OperationResult;
}
