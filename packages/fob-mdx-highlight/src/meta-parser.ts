/**
 * Parse fence metadata from code block info strings
 *
 * Supports:
 * - title="filename.ts" or title='filename.ts'
 * - {1,3-5,7} for line highlights
 * - word:foo,bar for word highlights
 */

export interface FenceMeta {
  title?: string;
  highlightLines: number[];
  highlightWords: string[];
}

/**
 * Parse fence metadata string
 *
 * @example
 * parseFenceMeta('title="app.ts" {1,3-5}')
 * // => { title: "app.ts", highlightLines: [1, 3, 4, 5], highlightWords: [] }
 */
export function parseFenceMeta(meta: string): FenceMeta {
  const result: FenceMeta = {
    highlightLines: [],
    highlightWords: [],
  };

  if (!meta || meta.trim().length === 0) {
    return result;
  }

  const trimmed = meta.trim();

  // Parse title="..." or title='...'
  const titleMatch = trimmed.match(/title=["']([^"']+)["']/);
  if (titleMatch && titleMatch[1]) {
    result.title = titleMatch[1];
  }

  // Parse line highlights {1,3-5,7}
  const lineMatch = trimmed.match(/\{([^}]+)\}/);
  if (lineMatch && lineMatch[1]) {
    const lineSpec = lineMatch[1];
    const parts = lineSpec.split(',').map((p) => p.trim());

    for (const part of parts) {
      if (part.includes('-')) {
        // Range: 3-5
        const splitParts = part.split('-').map((s) => s.trim());
        const startStr = splitParts[0];
        const endStr = splitParts[1];
        if (!startStr || !endStr) continue;
        const start = parseInt(startStr, 10);
        const end = parseInt(endStr, 10);

        if (!isNaN(start) && !isNaN(end)) {
          for (let i = start; i <= end; i++) {
            result.highlightLines.push(i);
          }
        }
      } else {
        // Single line: 1
        const line = parseInt(part, 10);
        if (!isNaN(line)) {
          result.highlightLines.push(line);
        }
      }
    }
  }

  // Parse word highlights word:foo,bar
  const wordMatch = trimmed.match(/word:([^\s]+)/);
  if (wordMatch && wordMatch[1]) {
    const words = wordMatch[1].split(',').map((w) => w.trim());
    result.highlightWords = words.filter((w) => w.length > 0);
  }

  return result;
}
