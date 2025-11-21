import type { Selection } from './selection.js';
import type { ChangeDescription } from './result.js';
import type { EditorState } from './core.js';

/**
 * Context provided to change event handlers
 */
export interface ChangeContext {
  /** Array of changes that occurred */
  changes: readonly ChangeDescription[];

  /** Get the current content */
  getContent(): string;

  /** Source of the change */
  source: 'user' | 'api' | 'undo' | 'redo' | 'paste';

  /** Whether the user is currently composing (IME) */
  isComposing: boolean;

  /** Current editor state after changes */
  state: EditorState;

  /** Previous editor state before changes */
  previousState: EditorState;
}

/**
 * Context provided to selection change event handlers
 */
export interface SelectionContext {
  /** Main selection */
  main: Selection;

  /** All selection ranges (for multi-cursor) */
  ranges: Selection[];

  /** Check if there's an active selection */
  hasSelection(): boolean;

  /** Get the selected text */
  getText(): string;

  /** Current editor state */
  state: EditorState;
}

/**
 * Context for focus events
 */
export interface FocusContext {
  /** Current editor state */
  state: EditorState;
}

/**
 * Context for blur events
 */
export interface BlurContext {
  /** Current editor state */
  state: EditorState;
}

/**
 * Type-safe event handler map
 */
export interface EventHandlers {
  /** Fired when content changes */
  change: (context: ChangeContext) => void;

  /** Fired when selection changes */
  selectionChange: (context: SelectionContext) => void;

  /** Fired when editor receives focus */
  focus: (context: FocusContext) => void;

  /** Fired when editor loses focus */
  blur: (context: BlurContext) => void;
}

/**
 * Event names (union type for type safety)
 */
export type EditorEvent = keyof EventHandlers;

/**
 * Unsubscribe function returned by event listeners
 */
export type Unsubscribe = () => void;

/**
 * Event system API
 * Subscribe to and emit events
 */
export interface EventAPI {
  /**
   * Subscribe to an event
   * @returns Unsubscribe function
   */
  on<E extends EditorEvent>(
    event: E,
    handler: EventHandlers[E]
  ): Unsubscribe;

  /**
   * Unsubscribe from an event
   */
  off<E extends EditorEvent>(
    event: E,
    handler: EventHandlers[E]
  ): void;

  /**
   * Subscribe to an event for one occurrence only
   * @returns Unsubscribe function
   */
  once<E extends EditorEvent>(
    event: E,
    handler: EventHandlers[E]
  ): Unsubscribe;

  /**
   * Emit an event (for internal use or custom events)
   */
  emit<E extends EditorEvent>(
    event: E,
    context: Parameters<EventHandlers[E]>[0]
  ): void;

  /**
   * Clear handlers for a specific event or all events
   */
  clear(event?: EditorEvent): void;
}
