import type { EditorView } from '@codemirror/view';
import type { ContentAPI } from './content.js';
import type { SelectionAPI } from './selection.js';
import type { EventAPI } from './events.js';
import type { FormattingAPI } from './formatting.js';
import type { HistoryAPI } from './history.js';
import type { SearchAPI } from './search.js';

/**
 * Position in the editor (line and column based, 0-indexed)
 */
export interface Position {
  /** Line number (0-indexed) */
  line: number;
  /** Column number (0-indexed) */
  column: number;
}

/**
 * Range in the editor (offset based, 0-indexed)
 */
export interface Range {
  /** Start offset */
  from: number;
  /** End offset */
  to: number;
}

/**
 * Configuration options for creating an editor instance
 */
export interface EditorConfig {
  /** The HTML element to render the editor into */
  container: HTMLElement;

  /** Initial content for the editor */
  content?: string;

  /** Language mode for syntax highlighting */
  language?: string;

  /** Whether the editor should be editable (default: true) */
  editable?: boolean;

  /** Placeholder text when editor is empty */
  placeholder?: string;

  /** Tab size in spaces (default: 2) */
  tabSize?: number;

  /** Whether to show line numbers (default: true) */
  lineNumbers?: boolean;

  /** Whether lines should wrap (default: false) */
  lineWrapping?: boolean;

  /** Theme: 'light' or 'dark' (default: 'light') */
  theme?: 'light' | 'dark';
}

/**
 * The main editor instance with all API surfaces
 */
export interface EditorInstance {
  /** Content manipulation API */
  readonly content: ContentAPI;

  /** Selection and cursor API */
  readonly selection: SelectionAPI;

  /** Event system API */
  readonly events: EventAPI;

  /** Formatting and text manipulation API */
  readonly formatting: FormattingAPI;

  /** History (undo/redo) API */
  readonly history: HistoryAPI;

  /** Search and replace API */
  readonly search: SearchAPI;

  /**
   * Direct access to the underlying CodeMirror EditorView
   * Use this for advanced customization or accessing CodeMirror-specific features
   */
  readonly view: EditorView;

  /** Focus the editor */
  focus(): void;

  /** Blur the editor (remove focus) */
  blur(): void;

  /** Enable or disable editing */
  setEditable(editable: boolean): void;

  /** Refresh the editor display */
  refresh(): void;

  /** Destroy the editor instance and clean up */
  destroy(): void;
}

/**
 * Internal editor state (for serialization/debugging)
 */
export interface EditorState {
  content: string;
  selection: {
    anchor: number;
    head: number;
  };
  editable: boolean;
  language: string | null;
}
