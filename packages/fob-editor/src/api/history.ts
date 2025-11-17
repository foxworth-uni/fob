import type { EditorView } from '@codemirror/view';
import { undo, redo, undoDepth, redoDepth } from '@codemirror/commands';

import type { HistoryAPI, OperationResult, HistoryEntry } from '../types/index.js';

/**
 * Implementation of the History API
 * Handles undo/redo operations
 */
export class History implements HistoryAPI {
  constructor(private view: EditorView) {}

  undo(): OperationResult {
    try {
      const success = undo(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Nothing to undo',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  redo(): OperationResult {
    try {
      const success = redo(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Nothing to redo',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  canUndo(): boolean {
    return undoDepth(this.view.state) > 0;
  }

  canRedo(): boolean {
    return redoDepth(this.view.state) > 0;
  }

  getHistory(): HistoryEntry[] {
    // TODO: Not yet implemented
    // CodeMirror's history is private and cannot be retrieved directly
    // This would require custom history tracking if needed
    console.warn(
      'getHistory() not implemented: CodeMirror history is internal and cannot be accessed'
    );
    return [];
  }

  clearHistory(): void {
    // TODO: Not yet implemented
    // No direct API to clear history in CodeMirror
    // Would require recreating the editor state to reset history
    console.warn(
      'clearHistory() not implemented: Requires recreating editor state'
    );
  }
}
