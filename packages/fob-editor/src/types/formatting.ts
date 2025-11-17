import type { OperationResult } from './result.js';

/**
 * Formatting and text manipulation API
 *
 * @remarks
 * Most methods are implemented using CodeMirror commands.
 * format() and formatRange() require external formatter integration (e.g., Prettier).
 */
export interface FormattingAPI {
  /**
   * Indent the current line or selection
   */
  indent(): OperationResult;

  /**
   * Apply language-aware indentation to the current selection (alias for indent)
   */
  indentSelection(): OperationResult;

  /**
   * Outdent the current line or selection
   */
  outdent(): OperationResult;

  /**
   * Comment the current line or selection
   */
  commentLine(): OperationResult;

  /**
   * Uncomment the current line or selection
   */
  uncommentLine(): OperationResult;

  /**
   * Toggle comment on the current line or selection
   */
  toggleComment(): OperationResult;

  /**
   * Format the entire document
   * @remarks Not yet implemented - requires external formatter integration
   */
  format(): Promise<OperationResult>;

  /**
   * Format a specific range
   * @remarks Not yet implemented - requires external formatter integration
   */
  formatRange(from: number, to: number): Promise<OperationResult>;

  /**
   * Duplicate the current line
   */
  duplicateLine(): OperationResult;

  /**
   * Move the current line up
   */
  moveLineUp(): OperationResult;

  /**
   * Move the current line down
   */
  moveLineDown(): OperationResult;

  /**
   * Delete from the cursor to the end of the current line
   */
  deleteToLineEnd(): OperationResult;

  /**
   * Delete from the cursor to the start of the current line
   */
  deleteToLineStart(): OperationResult;
}
