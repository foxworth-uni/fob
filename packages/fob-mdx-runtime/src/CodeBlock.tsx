'use client';

/**
 * CodeBlock component with syntax highlighting support
 *
 * Features:
 * - Copy button
 * - Line highlighting
 * - Word highlighting
 * - Optional line numbers
 * - Title display
 * - CSS variable-based theming
 */

import { useState, useCallback } from 'react';
import type { CodeBlockProps } from './types.js';

export function CodeBlock({
  lang,
  code,
  title,
  highlightLines = [],
  highlightWords = [],
  showLineNumbers = false,
  showCopyButton = true,
  className,
  ...props
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  // Handle copy to clipboard
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  }, [code]);

  // Split code into lines
  const lines = code.split('\n');

  // Check if a line should be highlighted
  const isLineHighlighted = (lineNumber: number) => highlightLines.includes(lineNumber);

  // Highlight words in a line
  const highlightWordsInLine = (line: string) => {
    if (highlightWords.length === 0) {
      return line;
    }

    // Simple word highlighting using spans
    let result = line;
    highlightWords.forEach((word) => {
      const regex = new RegExp(`\\b${escapeRegex(word)}\\b`, 'g');
      result = result.replace(regex, (match) => `<mark class="highlighted-word">${match}</mark>`);
    });

    return result;
  };

  return (
    <div className={`code-block ${className || ''}`} data-lang={lang} {...props}>
      {title && (
        <div className="code-block-title">
          <span className="code-block-title-text">{title}</span>
          <span className="code-block-lang">{lang}</span>
        </div>
      )}

      <div className="code-block-container">
        {showCopyButton && (
          <button
            className="code-block-copy-button"
            onClick={handleCopy}
            aria-label="Copy code to clipboard"
            type="button"
          >
            {copied ? (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path
                  d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z"
                  fill="currentColor"
                />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path
                  d="M5.75 1a.75.75 0 00-.75.75v3c0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75v-3a.75.75 0 00-.75-.75h-4.5zm-.75 7.75A.75.75 0 015.75 8h4.5a.75.75 0 01.75.75v5.5a.75.75 0 01-.75.75h-4.5a.75.75 0 01-.75-.75v-5.5z"
                  fill="currentColor"
                />
              </svg>
            )}
          </button>
        )}

        <pre className={`code-block-pre language-${lang}`}>
          <code className={`code-block-code language-${lang}`}>
            {lines.map((line, index) => {
              const lineNumber = index + 1;
              const highlighted = isLineHighlighted(lineNumber);

              return (
                <div
                  key={lineNumber}
                  className={`code-line ${highlighted ? 'code-line-highlighted' : ''}`}
                  data-line={lineNumber}
                >
                  {showLineNumbers && (
                    <span className="code-line-number" aria-hidden="true">
                      {lineNumber}
                    </span>
                  )}
                  <span
                    className="code-line-content"
                    dangerouslySetInnerHTML={{
                      __html: highlightWordsInLine(escapeHtml(line)),
                    }}
                  />
                </div>
              );
            })}
          </code>
        </pre>
      </div>
    </div>
  );
}

// Utility: Escape HTML entities
function escapeHtml(text: string): string {
  const htmlEscapes: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  };

  return text.replace(/[&<>"']/g, (char) => htmlEscapes[char] || char);
}

// Utility: Escape regex special characters
function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
