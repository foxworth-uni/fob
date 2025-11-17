import type { Position, Range } from './core.js';
import type { OperationResult } from './result.js';

/**
 * Content manipulation API
 * All methods for reading and modifying editor content
 */
export interface ContentAPI {
  // ========================================================================
  // QUERIES - Synchronous, no side effects
  // ========================================================================

  /** Get the entire content of the editor */
  get(): string;

  /** Get a specific line (0-indexed) */
  getLine(lineNumber: number): string | null;

  /** Get multiple lines */
  getLines(fromLine: number, toLine: number): string[];

  /** Get content in a specific range */
  getRange(range: Range): string;

  /** Get the total number of lines */
  lineCount(): number;

  /** Get the total character count */
  length(): number;

  /** Check if the editor is empty */
  isEmpty(): boolean;

  // ========================================================================
  // MUTATIONS - Return OperationResult
  // ========================================================================

  /** Set the entire content (replaces everything) */
  set(content: string): OperationResult;

  /** Insert text at a specific offset or cursor position */
  insert(text: string, at?: number): OperationResult;

  /** Delete a range of text */
  delete(from: number, to: number): OperationResult;

  /** Replace a range with new text */
  replace(from: number, to: number, text: string): OperationResult;

  /** Clear all content */
  clear(): OperationResult;

  // ========================================================================
  // LINE-BASED OPERATIONS
  // ========================================================================

  /** Insert text at the end of a specific line */
  appendToLine(lineNumber: number, text: string): OperationResult;

  /** Insert text at the beginning of a specific line */
  prependToLine(lineNumber: number, text: string): OperationResult;

  /** Insert a new line after the specified line */
  insertLine(lineNumber: number, text: string): OperationResult;

  /** Replace an entire line */
  replaceLine(lineNumber: number, text: string): OperationResult;

  /** Delete a specific line */
  deleteLine(lineNumber: number): OperationResult;

  // ========================================================================
  // POSITION UTILITIES
  // ========================================================================

  /** Convert offset to line/column position */
  positionAt(offset: number): Position;

  /** Convert line/column position to offset */
  offsetAt(position: Position): number;
  offsetAt(line: number, column: number): number;
}
