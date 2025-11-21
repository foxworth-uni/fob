import { EditorView, basicSetup } from 'codemirror';
import { javascript } from '@codemirror/lang-javascript';
import { EditorState, Compartment } from '@codemirror/state';
import { placeholder as placeholderExtension } from '@codemirror/view';
import { history } from '@codemirror/commands';

import { Content } from './api/content.js';
import { Selection } from './api/selection.js';
import { Events } from './api/events.js';
import { Formatting } from './api/formatting.js';
import { History } from './api/history.js';
import { Search } from './api/search.js';

import type {
  EditorInstance,
  EditorConfig,
  ContentAPI,
  SelectionAPI,
  EventAPI,
  FormattingAPI,
  HistoryAPI,
  SearchAPI,
  ChangeContext,
  SelectionContext,
  FocusContext,
  BlurContext,
  ChangeDescription,
} from './types/index.js';

/**
 * Internal Editor class implementation
 */
class Editor implements EditorInstance {
  private editableCompartment: Compartment;
  private language: string | null;

  // Public API surfaces
  readonly content: ContentAPI;
  readonly selection: SelectionAPI;
  readonly events: EventAPI;
  readonly formatting: FormattingAPI;
  readonly history: HistoryAPI;
  readonly search: SearchAPI;
  readonly view: EditorView;

  constructor(config: EditorConfig) {
    const {
      container,
      content = '',
      language = null,
      editable = true,
      placeholder = '',
      tabSize = 2,
      lineWrapping = false,
    } = config;

    // Store language for event contexts
    this.language = language;

    // Initialize event system first
    this.events = new Events();

    // Create compartment for dynamic configuration
    this.editableCompartment = new Compartment();

    // Create update listener for change events
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        // Get old and new content
        const oldContent = update.startState.doc.toString();
        const newContent = update.state.doc.toString();

        // Build change context
        const changes: ChangeDescription[] = [];

        // Extract changes from the update
        update.changes.iterChanges((fromA, toA, _fromB, _toB, inserted) => {
          changes.push({
            from: fromA,
            to: toA,
            inserted: inserted.toString(),
            deleted: oldContent.slice(fromA, toA),
          });
        });

        const changeContext: ChangeContext = {
          changes,
          getContent: () => newContent,
          source: 'user', // TODO: Detect source better
          isComposing: update.transactions.some((tr) => tr.isUserEvent('input.compose')),
          state: {
            content: newContent,
            selection: {
              anchor: update.state.selection.main.anchor,
              head: update.state.selection.main.head,
            },
            editable: update.state.facet(EditorView.editable),
            language: this.language,
          },
          previousState: {
            content: oldContent,
            selection: {
              anchor: update.startState.selection.main.anchor,
              head: update.startState.selection.main.head,
            },
            editable: update.startState.facet(EditorView.editable),
            language: this.language,
          },
        };

        // Emit change event
        this.events.emit('change', changeContext);
      }

      // Handle selection changes
      if (update.selectionSet) {
        const sel = update.state.selection.main;
        const selectionContext: SelectionContext = {
          main: {
            anchor: sel.anchor,
            head: sel.head,
            from: sel.from,
            to: sel.to,
          },
          ranges: update.state.selection.ranges.map((r) => ({
            anchor: r.anchor,
            head: r.head,
            from: r.from,
            to: r.to,
          })),
          hasSelection: () => !sel.empty,
          getText: () => update.state.doc.sliceString(sel.from, sel.to),
          state: {
            content: update.state.doc.toString(),
            selection: {
              anchor: sel.anchor,
              head: sel.head,
            },
            editable: update.state.facet(EditorView.editable),
            language: this.language,
          },
        };

        this.events.emit('selectionChange', selectionContext);
      }
    });

    // Create focus/blur listeners
    const focusListener = EditorView.focusChangeEffect.of((state, focusing) => {
      if (focusing) {
        const focusContext: FocusContext = {
          state: {
            content: state.doc.toString(),
            selection: {
              anchor: state.selection.main.anchor,
              head: state.selection.main.head,
            },
            editable: state.facet(EditorView.editable),
            language: this.language,
          },
        };
        this.events.emit('focus', focusContext);
      } else {
        const blurContext: BlurContext = {
          state: {
            content: state.doc.toString(),
            selection: {
              anchor: state.selection.main.anchor,
              head: state.selection.main.head,
            },
            editable: state.facet(EditorView.editable),
            language: this.language,
          },
        };
        this.events.emit('blur', blurContext);
      }
      return null;
    });

    // Create editor state
    const state = EditorState.create({
      doc: content,
      extensions: [
        basicSetup,
        javascript(),
        history(), // Add history support
        updateListener,
        focusListener,
        this.editableCompartment.of(EditorView.editable.of(editable)),
        ...(lineWrapping ? [EditorView.lineWrapping] : []),
        EditorState.tabSize.of(tabSize),
        ...(placeholder ? [placeholderExtension(placeholder)] : []),
      ],
    });

    // Create editor view
    this.view = new EditorView({
      state,
      parent: container,
    });

    // Initialize API surfaces
    this.content = new Content(this.view);
    this.selection = new Selection(this.view);
    this.formatting = new Formatting(this.view);
    this.history = new History(this.view);
    this.search = new Search(this.view);
  }

  focus(): void {
    this.view.focus();
  }

  blur(): void {
    this.view.contentDOM.blur();
  }

  setEditable(editable: boolean): void {
    this.view.dispatch({
      effects: this.editableCompartment.reconfigure(
        EditorView.editable.of(editable)
      ),
    });
  }

  refresh(): void {
    this.view.requestMeasure();
  }

  destroy(): void {
    // Clear all event listeners
    this.events.clear();
    this.view.destroy();
  }
}

/**
 * Create a new Mirror Editor instance
 *
 * @param config - Configuration options for the editor
 * @returns EditorInstance with namespaced API surfaces
 *
 * @example
 * ```typescript
 * const editor = MirrorEditor({
 *   container: document.getElementById('editor'),
 *   content: 'console.log("Hello World");',
 * });
 *
 * // Use the namespaced APIs
 * const text = editor.content.get();
 * editor.content.insert('// Comment\n', 0);
 * editor.selection.setCursor(0);
 *
 * // Subscribe to events
 * editor.events.on('change', (ctx) => {
 *   console.log('Content changed:', ctx.changes);
 * });
 * ```
 */
export function MirrorEditor(config: EditorConfig): EditorInstance {
  return new Editor(config);
}
