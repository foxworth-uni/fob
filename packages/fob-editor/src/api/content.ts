import type { EditorView } from '@codemirror/view';
import type { ContentAPI, Position, Range, OperationResult } from '../types/index.js';

/**
 * Implementation of the Content API
 * Handles all content manipulation operations
 */
export class Content implements ContentAPI {
  constructor(private view: EditorView) {}

  // ========================================================================
  // QUERIES
  // ========================================================================

  get(): string {
    return this.view.state.doc.toString();
  }

  getLine(lineNumber: number): string | null {
    const doc = this.view.state.doc;
    if (lineNumber < 0 || lineNumber >= doc.lines) {
      return null;
    }
    return doc.line(lineNumber + 1).text; // CodeMirror lines are 1-indexed
  }

  getLines(fromLine: number, toLine: number): string[] {
    const lines: string[] = [];
    for (let i = fromLine; i <= toLine; i++) {
      const line = this.getLine(i);
      if (line !== null) {
        lines.push(line);
      }
    }
    return lines;
  }

  getRange(range: Range): string {
    const doc = this.view.state.doc;
    return doc.sliceString(range.from, range.to);
  }

  lineCount(): number {
    return this.view.state.doc.lines;
  }

  length(): number {
    return this.view.state.doc.length;
  }

  isEmpty(): boolean {
    return this.view.state.doc.length === 0;
  }

  // ========================================================================
  // MUTATIONS
  // ========================================================================

  set(content: string): OperationResult {
    try {
      const doc = this.view.state.doc;
      const oldContent = doc.toString();

      this.view.dispatch({
        changes: {
          from: 0,
          to: doc.length,
          insert: content,
        },
      });

      return {
        success: true,
        changes: [
          {
            from: 0,
            to: doc.length,
            inserted: content,
            deleted: oldContent,
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

  insert(text: string, at?: number): OperationResult {
    try {
      const pos = at ?? this.view.state.selection.main.head;

      this.view.dispatch({
        changes: {
          from: pos,
          to: pos,
          insert: text,
        },
        selection: { anchor: pos + text.length },
      });

      return {
        success: true,
        changes: [
          {
            from: pos,
            to: pos,
            inserted: text,
            deleted: '',
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

  delete(from: number, to: number): OperationResult {
    try {
      const doc = this.view.state.doc;
      const deleted = doc.sliceString(from, to);

      this.view.dispatch({
        changes: {
          from,
          to,
          insert: '',
        },
      });

      return {
        success: true,
        changes: [
          {
            from,
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

  replace(from: number, to: number, text: string): OperationResult {
    try {
      const doc = this.view.state.doc;
      const deleted = doc.sliceString(from, to);

      this.view.dispatch({
        changes: {
          from,
          to,
          insert: text,
        },
      });

      return {
        success: true,
        changes: [
          {
            from,
            to,
            inserted: text,
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

  clear(): OperationResult {
    return this.set('');
  }

  // ========================================================================
  // LINE-BASED OPERATIONS
  // ========================================================================

  appendToLine(lineNumber: number, text: string): OperationResult {
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
      const pos = line.to;

      return this.insert(text, pos);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  prependToLine(lineNumber: number, text: string): OperationResult {
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
      const pos = line.from;

      return this.insert(text, pos);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  insertLine(lineNumber: number, text: string): OperationResult {
    try {
      const doc = this.view.state.doc;
      if (lineNumber < 0 || lineNumber > doc.lines) {
        return {
          success: false,
          changes: [],
          error: `Line ${lineNumber} out of bounds`,
        };
      }

      const isAppend = lineNumber >= doc.lines;
      const insertPos = isAppend ? doc.length : doc.line(lineNumber + 1).to;
      const needsLeadingNewline =
        insertPos > 0 && doc.sliceString(insertPos - 1, insertPos) !== '\n';
      const insertText = `${needsLeadingNewline ? '\n' : ''}${text}`;

      return this.insert(insertText, insertPos);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  replaceLine(lineNumber: number, text: string): OperationResult {
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
      return this.replace(line.from, line.to, text);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  deleteLine(lineNumber: number): OperationResult {
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
      // Include the newline character if not the last line
      const to = lineNumber < doc.lines - 1 ? line.to + 1 : line.to;

      return this.delete(line.from, to);
    } catch (error) {
      return {
        success: false,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // ========================================================================
  // POSITION UTILITIES
  // ========================================================================

  positionAt(offset: number): Position {
    const doc = this.view.state.doc;
    const line = doc.lineAt(offset);
    return {
      line: line.number - 1, // Convert to 0-indexed
      column: offset - line.from,
    };
  }

  offsetAt(positionOrLine: Position | number, column?: number): number {
    if (typeof positionOrLine === 'number') {
      // Called as offsetAt(line, column)
      const doc = this.view.state.doc;
      const line = doc.line(positionOrLine + 1); // 1-indexed
      return line.from + (column ?? 0);
    } else {
      // Called as offsetAt(position)
      const doc = this.view.state.doc;
      const line = doc.line(positionOrLine.line + 1); // 1-indexed
      return line.from + positionOrLine.column;
    }
  }
}
