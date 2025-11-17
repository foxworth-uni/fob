/**
 * Type definitions for the Mirror Editor API
 * @packageDocumentation
 */

// Core types
export type {
  Position,
  Range,
  EditorConfig,
  EditorInstance,
  EditorState,
} from './core.js';

// Result types
export type { ChangeDescription, OperationResult } from './result.js';

// Content API
export type { ContentAPI } from './content.js';

// Selection API
export type {
  Selection,
  Cursor,
  CursorDirection,
  SelectionAPI,
} from './selection.js';

// Event API
export type {
  ChangeContext,
  SelectionContext,
  FocusContext,
  BlurContext,
  EventHandlers,
  EditorEvent,
  Unsubscribe,
  EventAPI,
} from './events.js';

// Formatting, History, and Search APIs
export type { FormattingAPI } from './formatting.js';
export type { HistoryAPI, HistoryEntry } from './history.js';
export type {
  FindOptions,
  FindResult,
  FindReplaceOptions,
  ReplaceResult,
  SearchAPI,
} from './search.js';
