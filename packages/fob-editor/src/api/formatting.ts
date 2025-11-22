import type { EditorView } from '@codemirror/view';
import {
  indentMore,
  indentLess,
  toggleComment,
  toggleLineComment,
  lineUncomment,
  copyLineDown,
  moveLineUp,
  moveLineDown,
} from '@codemirror/commands';

import type { FormattingAPI, OperationResult } from '../types/index.js';

/**
 * Implementation of the Formatting API
 * Handles text formatting and manipulation operations
 */
export class Formatting implements FormattingAPI {
  constructor(private view: EditorView) {}

  indent(): OperationResult {
    try {
      const success = indentMore(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Indent operation not applicable',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  indentSelection(): OperationResult {
    return this.indent();
  }

  outdent(): OperationResult {
    try {
      const success = indentLess(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Outdent operation not applicable',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  commentLine(): OperationResult {
    try {
      const success = toggleLineComment(this.view);
      return {
        success,
        changes: [],
        error: success
          ? undefined
          : 'Comment operation not applicable (language may not support comments)',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  uncommentLine(): OperationResult {
    try {
      const success = lineUncomment(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'No comments to remove',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  toggleComment(): OperationResult {
    try {
      const success = toggleComment(this.view);
      return {
        success,
        changes: [],
        error: success
          ? undefined
          : 'Toggle comment operation not applicable (language may not support comments)',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  async format(): Promise<OperationResult> {
    // TODO: Implement document formatting
    // Requires integration with external formatter like Prettier
    return {
      success: false,
      changes: [],
      error:
        'format() not yet implemented - requires external formatter integration (e.g., Prettier)',
    };
  }

  async formatRange(_from: number, _to: number): Promise<OperationResult> {
    // TODO: Implement range formatting
    // Requires integration with external formatter like Prettier
    return {
      success: false,
      changes: [],
      error:
        'formatRange() not yet implemented - requires external formatter integration (e.g., Prettier)',
    };
  }

  duplicateLine(): OperationResult {
    try {
      const success = copyLineDown(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Duplicate line operation failed',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  moveLineUp(): OperationResult {
    try {
      const success = moveLineUp(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Cannot move line up (already at top)',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  moveLineDown(): OperationResult {
    try {
      const success = moveLineDown(this.view);
      return {
        success,
        changes: [],
        error: success ? undefined : 'Cannot move line down (already at bottom)',
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  deleteToLineEnd(): OperationResult {
    try {
      const state = this.view.state;
      const head = state.selection.main.head;
      const line = state.doc.lineAt(head);

      let to = line.to;
      if (head === to && line.number < state.doc.lines) {
        to = Math.min(state.doc.length, to + 1);
      }

      if (head >= to) {
        return {
          success: false,
          changes: [],
          error: 'Cursor already at line end',
        };
      }

      const deleted = state.doc.sliceString(head, to);

      this.view.dispatch({
        changes: {
          from: head,
          to,
          insert: '',
        },
        selection: { anchor: head },
      });

      return {
        success: true,
        changes: [
          {
            from: head,
            to,
            inserted: '',
            deleted,
          },
        ],
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  deleteToLineStart(): OperationResult {
    try {
      const state = this.view.state;
      const head = state.selection.main.head;
      const line = state.doc.lineAt(head);

      if (head <= line.from) {
        return {
          success: false,
          changes: [],
          error: 'Cursor already at line start',
        };
      }

      const deleted = state.doc.sliceString(line.from, head);

      this.view.dispatch({
        changes: {
          from: line.from,
          to: head,
          insert: '',
        },
        selection: { anchor: line.from },
      });

      return {
        success: true,
        changes: [
          {
            from: line.from,
            to: head,
            inserted: '',
            deleted,
          },
        ],
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }
}
