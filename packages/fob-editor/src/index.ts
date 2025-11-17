/**
 * Mirror Editor - Vanilla JavaScript editor built on CodeMirror
 * @packageDocumentation
 */

import './styles/editor.css';

// Main factory function
export { MirrorEditor } from './editor.js';

// Core types
export type {
  EditorInstance,
  EditorConfig,
  EditorState,
  Position,
  Range,
} from './types/index.js';

// API types
export type { ContentAPI } from './types/index.js';
export type { SelectionAPI, Selection, Cursor } from './types/index.js';
export type {
  EventAPI,
  EditorEvent,
  EventHandlers,
  Unsubscribe,
  ChangeContext,
  SelectionContext,
  FocusContext,
  BlurContext,
} from './types/index.js';

// Operation result types
export type { OperationResult, ChangeDescription } from './types/index.js';

// Formatting API (implemented)
export type { FormattingAPI } from './types/index.js';

// History API (implemented)
export type { HistoryAPI, HistoryEntry } from './types/index.js';

// Search API (implemented)
export type {
  SearchAPI,
  FindOptions,
  FindResult,
  FindReplaceOptions,
  ReplaceResult,
} from './types/index.js';
