/**
 * Integration tests for structured error handling in bundler-node.
 *
 * These tests verify that errors from the NAPI layer are correctly parsed,
 * typed, and formatted for consumption by TypeScript/JavaScript applications.
 */

import { test, expect } from 'vitest';
import {
  FobError,
  formatFobError,
  type FobErrorDetails,
  type MdxSyntaxError,
  type MissingExportError,
  type TransformError,
  type CircularDependencyError,
  type MultipleDiagnostics,
} from '../dist/types.js';

// Test: FobError class with structured details
test('FobError stores structured error details', () => {
  const details: MdxSyntaxError = {
    type: 'mdx_syntax',
    message: 'Unexpected token',
    file: 'src/content/post.mdx',
    line: 12,
    column: 8,
    context: '> 12 | import { foo',
    suggestion: 'Check your MDX syntax',
  };

  const error = new FobError('MDX Error: Unexpected token', details);

  expect(error.name).toBe('FobError');
  expect(error.message).toBe('MDX Error: Unexpected token');
  expect(error.details).toBeTruthy();
  expect(error.details?.type).toBe('mdx_syntax');

  if (error.details?.type === 'mdx_syntax') {
    expect(error.details.message).toBe('Unexpected token');
    expect(error.details.file).toBe('src/content/post.mdx');
    expect(error.details.line).toBe(12);
    expect(error.details.column).toBe(8);
  }
});

// Test: formatFobError for MDX syntax errors
test('formatFobError formats MDX syntax error correctly', () => {
  const error: MdxSyntaxError = {
    type: 'mdx_syntax',
    message: 'Unexpected token',
    file: 'src/content/post.mdx',
    line: 12,
    column: 8,
    context: '> 12 | import { foo',
    suggestion: 'Check your MDX syntax',
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('MDX Error: Unexpected token');
  expect(formatted).toContain('src/content/post.mdx');
  expect(formatted).toContain('line 12, column 8');
  expect(formatted).toContain('> 12 | import { foo');
  expect(formatted).toContain('💡 Suggestion: Check your MDX syntax');
});

// Test: formatFobError for missing export errors
test('formatFobError formats missing export error correctly', () => {
  const error: MissingExportError = {
    type: 'missing_export',
    export_name: 'Button',
    module_id: 'components/Button.tsx',
    available_exports: ['default', 'Icon'],
    suggestion: 'default',
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain("Named export 'Button' not found");
  expect(formatted).toContain('components/Button.tsx');
  expect(formatted).toContain('Available exports: default, Icon');
  expect(formatted).toContain("Did you mean 'default'?");
});

// Test: formatFobError for transform errors
test('formatFobError formats transform error correctly', () => {
  const error: TransformError = {
    type: 'transform',
    path: 'src/app.tsx',
    diagnostics: [
      {
        message: 'Type mismatch',
        line: 20,
        column: 15,
        severity: 'error',
        help: 'Check the type annotation',
      },
      {
        message: 'Unused variable',
        line: 30,
        column: 5,
        severity: 'warning',
      },
    ],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('Transform failed in src/app.tsx');
  expect(formatted).toContain('Diagnostics:');
  expect(formatted).toContain('[error] Type mismatch (line 20, col 15)');
  expect(formatted).toContain('Help: Check the type annotation');
  expect(formatted).toContain('[warning] Unused variable (line 30, col 5)');
});

// Test: formatFobError for circular dependency errors
test('formatFobError formats circular dependency error correctly', () => {
  const error: CircularDependencyError = {
    type: 'circular_dependency',
    cycle_path: ['src/a.js', 'src/b.js', 'src/a.js'],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('Circular dependency detected:');
  expect(formatted).toContain('src/a.js → src/b.js → src/a.js');
});

// Test: formatFobError for no entries error
test('formatFobError formats no entries error correctly', () => {
  const error: FobErrorDetails = {
    type: 'no_entries',
  };

  const formatted = formatFobError(error);

  expect(formatted).toBe('No entry points specified');
});

// Test: formatFobError for invalid entry error
test('formatFobError formats invalid entry error correctly', () => {
  const error: FobErrorDetails = {
    type: 'invalid_entry',
    path: 'src/index.tsx',
  };

  const formatted = formatFobError(error);

  expect(formatted).toBe('Invalid entry point: src/index.tsx');
});

// Test: formatFobError for plugin error
test('formatFobError formats plugin error correctly', () => {
  const error: FobErrorDetails = {
    type: 'plugin',
    name: 'image-optimizer',
    message: 'Failed to optimize image.png',
  };

  const formatted = formatFobError(error);

  expect(formatted).toBe("Plugin 'image-optimizer' failed: Failed to optimize image.png");
});

// Test: formatFobError for runtime error
test('formatFobError formats runtime error correctly', () => {
  const error: FobErrorDetails = {
    type: 'runtime',
    message: 'Out of memory',
  };

  const formatted = formatFobError(error);

  expect(formatted).toBe('Runtime error: Out of memory');
});

// Test: formatFobError for validation error
test('formatFobError formats validation error correctly', () => {
  const error: FobErrorDetails = {
    type: 'validation',
    message: 'Invalid configuration',
  };

  const formatted = formatFobError(error);

  expect(formatted).toBe('Validation error: Invalid configuration');
});

// Test: Error serialization round-trip
test('Error details serialize and deserialize correctly', () => {
  const error: FobErrorDetails = {
    type: 'mdx_syntax',
    message: 'Parse error',
    file: 'content/blog.mdx',
    line: 5,
    column: 10,
  };

  const json = JSON.stringify(error);
  const deserialized = JSON.parse(json) as FobErrorDetails;

  expect(deserialized.type).toBe('mdx_syntax');
  if (deserialized.type === 'mdx_syntax') {
    expect(deserialized.message).toBe('Parse error');
    expect(deserialized.file).toBe('content/blog.mdx');
    expect(deserialized.line).toBe(5);
    expect(deserialized.column).toBe(10);
  }
});

// Test: Missing export error with no suggestion
test('formatFobError handles missing export without suggestion', () => {
  const error: MissingExportError = {
    type: 'missing_export',
    export_name: 'CustomComponent',
    module_id: 'components/Custom.tsx',
    available_exports: [],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain("Named export 'CustomComponent' not found");
  expect(formatted).toContain('Module has no exports');
  expect(formatted.includes('Did you mean')).toBe(false);
});

// Test: MDX error with minimal fields
test('formatFobError handles MDX error with minimal fields', () => {
  const error: MdxSyntaxError = {
    type: 'mdx_syntax',
    message: 'Parse error',
  };

  const formatted = formatFobError(error);

  expect(formatted).toBe('MDX Error: Parse error');
});

// Test: Transform error with help text
test('formatFobError includes help text in diagnostics', () => {
  const error: TransformError = {
    type: 'transform',
    path: 'src/main.tsx',
    diagnostics: [
      {
        message: 'Cannot find module',
        line: 10,
        column: 5,
        severity: 'error',
        help: 'Install the missing package',
      },
    ],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('Help: Install the missing package');
});

// Test: formatFobError for multiple diagnostics
test('formatFobError formats multiple diagnostics correctly', () => {
  const error: MultipleDiagnostics = {
    type: 'multiple',
    primary_message: 'Multiple bundler errors',
    errors: [
      {
        type: 'missing_export',
        export_name: 'Button',
        module_id: 'components/Button.tsx',
        available_exports: ['default'],
      },
      {
        type: 'transform',
        path: 'src/app.tsx',
        diagnostics: [
          {
            message: 'Syntax error',
            line: 10,
            column: 5,
            severity: 'error',
          },
        ],
      },
    ],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('Multiple errors (2):');
  expect(formatted).toContain("1. Named export 'Button' not found");
  expect(formatted).toContain('2. Transform failed in src/app.tsx');
});

// Test: Multiple diagnostics serialization round-trip
test('Multiple diagnostics serialize and deserialize correctly', () => {
  const error: MultipleDiagnostics = {
    type: 'multiple',
    primary_message: 'Multiple errors occurred',
    errors: [
      {
        type: 'missing_export',
        export_name: 'Foo',
        module_id: 'bar.js',
        available_exports: [],
      },
      {
        type: 'runtime',
        message: 'Something went wrong',
      },
    ],
  };

  const json = JSON.stringify(error);
  const deserialized = JSON.parse(json) as FobErrorDetails;

  expect(deserialized.type).toBe('multiple');
  if (deserialized.type === 'multiple') {
    expect(deserialized.primary_message).toBe('Multiple errors occurred');
    expect(deserialized.errors).toHaveLength(2);
    expect(deserialized.errors[0].type).toBe('missing_export');
    expect(deserialized.errors[1].type).toBe('runtime');
  }
});
