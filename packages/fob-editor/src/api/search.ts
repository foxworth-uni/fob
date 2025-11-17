import type { EditorView } from '@codemirror/view';
import { SearchCursor, RegExpCursor } from '@codemirror/search';
import { EditorSelection } from '@codemirror/state';

import type {
  SearchAPI,
  FindOptions,
  FindResult,
  FindReplaceOptions,
  ReplaceResult,
} from '../types/index.js';

/**
 * Implementation of the Search API
 * Handles find and replace operations
 */
export class Search implements SearchAPI {
  constructor(private view: EditorView) {}

  find(query: string | RegExp, options: FindOptions = {}): FindResult[] {
    const { caseSensitive = false, regex = false, wholeWord = false } = options;

    // If it's a RegExp or regex flag is set, use RegExpCursor
    if (query instanceof RegExp || regex) {
      return this.findRegex(query, { caseSensitive, wholeWord });
    }

    // For whole word, convert to regex
    if (wholeWord) {
      const pattern = new RegExp(`\\b${this.escapeRegex(query)}\\b`, caseSensitive ? 'g' : 'gi');
      return this.findRegex(pattern, { caseSensitive });
    }

    // Use SearchCursor for plain text
    const cursor = new SearchCursor(
      this.view.state.doc,
      query,
      0,
      this.view.state.doc.length,
      caseSensitive ? undefined : (x) => x.toLowerCase()
    );

    const results: FindResult[] = [];
    while (!cursor.next().done) {
      results.push({
        range: {
          from: cursor.value.from,
          to: cursor.value.to,
        },
        text: this.view.state.doc.sliceString(cursor.value.from, cursor.value.to),
        index: results.length,
      });
    }

    return results;
  }

  findNext(query: string | RegExp, options: FindOptions = {}): FindResult | null {
    const { caseSensitive = false, regex = false, wholeWord = false } = options;
    const currentPos = this.view.state.selection.main.head;

    // Build the actual query based on options
    let searchQuery: string | RegExp = query;
    let useRegex = query instanceof RegExp || regex;

    if (!(query instanceof RegExp)) {
      if (wholeWord) {
        searchQuery = new RegExp(`\\b${this.escapeRegex(query as string)}\\b`, caseSensitive ? 'g' : 'gi');
        useRegex = true;
      }
    }

    if (useRegex) {
      const pattern = searchQuery instanceof RegExp ? searchQuery.source : searchQuery as string;
      const ignoreCase = searchQuery instanceof RegExp
        ? searchQuery.flags.includes('i')
        : !caseSensitive;

      const cursor = new RegExpCursor(this.view.state.doc, pattern, {
        ignoreCase,
      }, currentPos);

      if (!cursor.next().done) {
        const result: FindResult = {
          range: {
            from: cursor.value.from,
            to: cursor.value.to,
          },
          text: this.view.state.doc.sliceString(cursor.value.from, cursor.value.to),
          index: 0,
        };

        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(result.range.from, result.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();

        return result;
      }

      // Wrap around to beginning
      const wrapCursor = new RegExpCursor(this.view.state.doc, pattern, {
        ignoreCase,
      }, 0);

      if (!wrapCursor.next().done && wrapCursor.value.from < currentPos) {
        const result: FindResult = {
          range: {
            from: wrapCursor.value.from,
            to: wrapCursor.value.to,
          },
          text: this.view.state.doc.sliceString(wrapCursor.value.from, wrapCursor.value.to),
          index: 0,
        };

        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(result.range.from, result.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();

        return result;
      }
    } else {
      const cursor = new SearchCursor(
        this.view.state.doc,
        searchQuery as string,
        currentPos,
        undefined,
        caseSensitive ? undefined : (x) => x.toLowerCase()
      );

      if (!cursor.next().done) {
        const result: FindResult = {
          range: {
            from: cursor.value.from,
            to: cursor.value.to,
          },
          text: this.view.state.doc.sliceString(cursor.value.from, cursor.value.to),
          index: 0,
        };

        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(result.range.from, result.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();

        return result;
      }

      // Wrap around to beginning
      const wrapCursor = new SearchCursor(
        this.view.state.doc,
        searchQuery as string,
        0,
        undefined,
        caseSensitive ? undefined : (x) => x.toLowerCase()
      );

      if (!wrapCursor.next().done && wrapCursor.value.from < currentPos) {
        const result: FindResult = {
          range: {
            from: wrapCursor.value.from,
            to: wrapCursor.value.to,
          },
          text: this.view.state.doc.sliceString(wrapCursor.value.from, wrapCursor.value.to),
          index: 0,
        };

        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(result.range.from, result.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();

        return result;
      }
    }

    return null;
  }

  findPrevious(query: string | RegExp, options: FindOptions = {}): FindResult | null {
    const { caseSensitive = false, regex = false, wholeWord = false } = options;
    const currentPos = this.view.state.selection.main.head;

    // Build the actual query based on options
    let searchQuery: string | RegExp = query;
    let useRegex = query instanceof RegExp || regex;

    if (!(query instanceof RegExp)) {
      if (wholeWord) {
        searchQuery = new RegExp(`\\b${this.escapeRegex(query as string)}\\b`, caseSensitive ? 'g' : 'gi');
        useRegex = true;
      }
    }

    // Search backwards by finding all matches before cursor
    if (useRegex) {
      const pattern = searchQuery instanceof RegExp ? searchQuery.source : searchQuery as string;
      const ignoreCase = searchQuery instanceof RegExp
        ? searchQuery.flags.includes('i')
        : !caseSensitive;

      const cursor = new RegExpCursor(this.view.state.doc, pattern, {
        ignoreCase,
      }, 0);

      let lastMatch: FindResult | null = null;

      while (!cursor.next().done && cursor.value.from < currentPos) {
        lastMatch = {
          range: {
            from: cursor.value.from,
            to: cursor.value.to,
          },
          text: this.view.state.doc.sliceString(cursor.value.from, cursor.value.to),
          index: 0,
        };
      }

      if (lastMatch) {
        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(lastMatch.range.from, lastMatch.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();
        return lastMatch;
      }

      // Wrap around to end
      const wrapCursor = new RegExpCursor(
        this.view.state.doc,
        pattern,
        { ignoreCase },
        0
      );

      let lastWrapMatch: FindResult | null = null;
      while (!wrapCursor.next().done) {
        lastWrapMatch = {
          range: {
            from: wrapCursor.value.from,
            to: wrapCursor.value.to,
          },
          text: this.view.state.doc.sliceString(wrapCursor.value.from, wrapCursor.value.to),
          index: 0,
        };
      }

      if (lastWrapMatch) {
        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(lastWrapMatch.range.from, lastWrapMatch.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();
      }

      return lastWrapMatch;
    } else {
      const cursor = new SearchCursor(
        this.view.state.doc,
        searchQuery as string,
        0,
        currentPos,
        caseSensitive ? undefined : (x) => x.toLowerCase()
      );

      let lastMatch: FindResult | null = null;

      while (!cursor.next().done) {
        lastMatch = {
          range: {
            from: cursor.value.from,
            to: cursor.value.to,
          },
          text: this.view.state.doc.sliceString(cursor.value.from, cursor.value.to),
          index: 0,
        };
      }

      if (lastMatch) {
        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(lastMatch.range.from, lastMatch.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();
        return lastMatch;
      }

      // Wrap around to end
      const wrapCursor = new SearchCursor(
        this.view.state.doc,
        searchQuery as string,
        0,
        this.view.state.doc.length,
        caseSensitive ? undefined : (x) => x.toLowerCase()
      );

      let lastWrapMatch: FindResult | null = null;
      while (!wrapCursor.next().done) {
        lastWrapMatch = {
          range: {
            from: wrapCursor.value.from,
            to: wrapCursor.value.to,
          },
          text: this.view.state.doc.sliceString(wrapCursor.value.from, wrapCursor.value.to),
          index: 0,
        };
      }

      if (lastWrapMatch) {
        // Select the found text
        this.view.dispatch({
          selection: EditorSelection.create([
            EditorSelection.range(lastWrapMatch.range.from, lastWrapMatch.range.to)
          ]),
          scrollIntoView: true,
        });
        this.view.focus();
      }

      return lastWrapMatch;
    }
  }

  replace(
    query: string | RegExp,
    replacement: string,
    options: FindReplaceOptions = {}
  ): ReplaceResult {
    const { caseSensitive = false, regex = false, wholeWord = false, all = false } = options;

    try {
      // Find matches
      const matches = this.find(query, { caseSensitive, regex, wholeWord });

      if (matches.length === 0) {
        return {
          success: true,
          count: 0,
          changes: [],
        };
      }

      // Collect changes with deleted text captured BEFORE dispatch
      const matchesToReplace = all ? matches : matches.slice(0, 1);
      const changeDescriptions = matchesToReplace.map((match) => ({
        from: match.range.from,
        to: match.range.to,
        inserted: replacement,
        deleted: this.view.state.doc.sliceString(match.range.from, match.range.to),
      }));

      // Prepare changes for dispatch
      const changes = matchesToReplace.map((match) => ({
        from: match.range.from,
        to: match.range.to,
        insert: replacement,
      }));

      // Apply changes in a single transaction
      this.view.dispatch({ changes });

      return {
        success: true,
        count: changeDescriptions.length,
        changes: changeDescriptions,
      };
    } catch (error) {
      return {
        success: false,
        count: 0,
        changes: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  replaceAll(query: string | RegExp, replacement: string): ReplaceResult {
    return this.replace(query, replacement, { all: true });
  }

  // Helper methods

  private findRegex(query: string | RegExp, options: { caseSensitive?: boolean; wholeWord?: boolean }): FindResult[] {
    const { caseSensitive = false } = options;

    const pattern = query instanceof RegExp ? query.source : query;
    const cursor = new RegExpCursor(this.view.state.doc, pattern, {
      ignoreCase: !caseSensitive,
    });

    const results: FindResult[] = [];
    while (!cursor.next().done) {
      results.push({
        range: {
          from: cursor.value.from,
          to: cursor.value.to,
        },
        text: this.view.state.doc.sliceString(cursor.value.from, cursor.value.to),
        index: results.length,
      });
    }

    return results;
  }

  private escapeRegex(str: string): string {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }
}
