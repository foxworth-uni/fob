import type { EditorView } from '@codemirror/view';
import type {
  SelectionAPI,
  Selection as SelectionType,
  Cursor,
  CursorDirection,
  Range,
  OperationResult,
} from '../types/index.js';

/**
 * Implementation of the Selection API
 * Handles all selection and cursor operations
 */
export class Selection implements SelectionAPI {
  constructor(private view: EditorView) {}

  // ========================================================================
  // QUERIES
  // ========================================================================

  get(): SelectionType {
    const sel = this.view.state.selection.main;
    return {
      anchor: sel.anchor,
      head: sel.head,
      from: sel.from,
      to: sel.to,
    };
  }

  getCursor(): Cursor {
    const pos = this.view.state.selection.main.head;
    const doc = this.view.state.doc;
    const line = doc.lineAt(pos);

    return {
      offset: pos,
      line: line.number - 1, // Convert to 0-indexed
      column: pos - line.from,
    };
  }

  getSelectedText(): string {
    const sel = this.view.state.selection.main;
    return this.view.state.doc.sliceString(sel.from, sel.to);
  }

  hasSelection(): boolean {
    return !this.view.state.selection.main.empty;
  }

  // ========================================================================
  // MUTATIONS
  // ========================================================================

  set(from: number, to: number): OperationResult {
    try {
      this.view.dispatch({
        selection: { anchor: from, head: to },
      });

      return {
        success: true,
        changes: [],
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  setCursor(positionOrLine: number, column?: number): OperationResult {
    try {
      let offset: number;

      if (column !== undefined) {
        // Called as setCursor(line, column)
        const doc = this.view.state.doc;
        const line = doc.line(positionOrLine + 1); // 1-indexed
        offset = line.from + column;
      } else {
        // Called as setCursor(position)
        offset = positionOrLine;
      }

      this.view.dispatch({
        selection: { anchor: offset },
      });

      return {
        success: true,
        changes: [],
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  moveCursor(direction: CursorDirection, amount: number = 1): OperationResult {
    try {
      const doc = this.view.state.doc;
      const currentPos = this.view.state.selection.main.head;
      const currentLine = doc.lineAt(currentPos);
      let newPos = currentPos;

      switch (direction) {
        case 'left':
          newPos = Math.max(0, currentPos - amount);
          break;
        case 'right':
          newPos = Math.min(doc.length, currentPos + amount);
          break;
        case 'up':
          if (currentLine.number > 1) {
            const targetLine = doc.line(currentLine.number - amount);
            const column = currentPos - currentLine.from;
            newPos = Math.min(targetLine.to, targetLine.from + column);
          }
          break;
        case 'down':
          if (currentLine.number < doc.lines) {
            const targetLine = doc.line(Math.min(doc.lines, currentLine.number + amount));
            const column = currentPos - currentLine.from;
            newPos = Math.min(targetLine.to, targetLine.from + column);
          }
          break;
      }

      return this.setCursor(newPos);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  selectAll(): OperationResult {
    try {
      const doc = this.view.state.doc;
      this.view.dispatch({
        selection: { anchor: 0, head: doc.length },
      });

      return {
        success: true,
        changes: [],
      };
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  selectRange(range: Range): OperationResult {
    return this.set(range.from, range.to);
  }

  selectLine(lineNumber: number): OperationResult {
    try {
      const doc = this.view.state.doc;
      if (lineNumber < 0 || lineNumber >= doc.lines) {
        return {
          success: false,
          changes: [],
          error: `Line ${lineNumber} out of bounds`,
        };
      }

      const line = doc.line(lineNumber + 1); // 1-indexed
      return this.set(line.from, line.to);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  selectLines(fromLine: number, toLine: number): OperationResult {
    try {
      const doc = this.view.state.doc;
      if (fromLine < 0 || toLine >= doc.lines || fromLine > toLine) {
        return {
          success: false,
          changes: [],
          error: `Invalid line range: ${fromLine}-${toLine}`,
        };
      }

      const startLine = doc.line(fromLine + 1); // 1-indexed
      const endLine = doc.line(toLine + 1); // 1-indexed
      return this.set(startLine.from, endLine.to);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  collapse(toStart: boolean = false): OperationResult {
    try {
      const sel = this.view.state.selection.main;
      const pos = toStart ? sel.from : sel.to;

      this.view.dispatch({
        selection: { anchor: pos },
      });

      return {
        success: true,
        changes: [],
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
