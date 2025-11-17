import type { OperationResult } from './result.js';

/**
 * History entry for the transaction log
 *
 * @remarks
 * Not currently available - CodeMirror's history is private
 */
export interface HistoryEntry {
  /** Timestamp of the change */
  timestamp: number;
  /** Source of the change */
  source: string;
  /** Content before the change */
  before: string;
  /** Content after the change */
  after: string;
}

/**
 * History and undo/redo API
 *
 * @remarks
 * Undo/redo operations are implemented using CodeMirror's history extension.
 * getHistory() and clearHistory() are not implemented due to CodeMirror limitations.
 */
export interface HistoryAPI {
  /**
   * Undo the last change
   */
  undo(): OperationResult;

  /**
   * Redo the last undone change
   */
  redo(): OperationResult;

  /**
   * Check if undo is available
   */
  canUndo(): boolean;

  /**
   * Check if redo is available
   */
  canRedo(): boolean;

  /**
   * Get the history entries
   * @remarks Not implemented - CodeMirror's history state is not publicly accessible
   */
  getHistory(): HistoryEntry[];

  /**
   * Clear the history
   * @remarks Not implemented - would require recreating the editor state
   */
  clearHistory(): void;
}
