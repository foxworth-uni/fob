/**
 * Error handling integration tests for Fob bundler
 *
 * Tests the discriminated union error pattern from Rust → NAPI → TypeScript
 */

import { test, expect } from 'vitest';

// Import after setup-native to get native mock
import { getNativeMock } from './helpers/setup-native.js';
import * as native from '../dist/native/index.js';
import { FobError, formatFobError } from '../dist/types.js';

const mockNative = getNativeMock();

test('throws FobError with MDX syntax error details', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'mdx_syntax',
    message: 'Unexpected token <',
    file: 'src/page.mdx',
    line: 5,
    column: 10,
    context: '> 5 | <Component',
    suggestion: 'Check your JSX syntax',
  });

  try {
    await native.bundle({ entries: ['src/page.mdx'] });
  } catch (error) {
    expect(error instanceof FobError, 'Error should be FobError').toBe(true);
    expect(error.details, 'Error should have details').toBeTruthy();
    expect(error.details.type).toBe('mdx_syntax');
    expect(error.details.message).toBe('Unexpected token <');
    expect(error.details.file).toBe('src/page.mdx');
    expect(error.details.line).toBe(5);
    expect(error.details.column).toBe(10);
  }
});

test('throws FobError with missing export error details', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'missing_export',
    export_name: 'Button',
    module_id: 'components/Button.tsx',
    available_exports: ['default', 'Icon'],
    suggestion: 'default',
  });

  try {
    await native.bundle({ entries: ['src/index.js'] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('missing_export');
    expect(error.details.export_name).toBe('Button');
    expect(error.details.module_id).toBe('components/Button.tsx');
    expect(error.details.available_exports).toEqual(['default', 'Icon']);
    expect(error.details.suggestion).toBe('default');
  }
});

test('throws FobError with transform error diagnostics', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'transform',
    path: 'src/app.tsx',
    diagnostics: [
      {
        message: 'Type error',
        line: 10,
        column: 5,
        severity: 'error',
        help: 'Check type annotation',
      },
      {
        message: 'Unused variable',
        line: 15,
        column: 8,
        severity: 'warning',
      },
    ],
  });

  try {
    await native.bundle({ entries: ['src/app.tsx'] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('transform');
    expect(error.details.path).toBe('src/app.tsx');
    expect(error.details.diagnostics.length).toBe(2);
    expect(error.details.diagnostics[0].message).toBe('Type error');
    expect(error.details.diagnostics[0].severity).toBe('error');
    expect(error.details.diagnostics[0].help).toBe('Check type annotation');
  }
});

test('throws FobError with circular dependency error', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'circular_dependency',
    cycle_path: ['src/a.js', 'src/b.js', 'src/c.js', 'src/a.js'],
  });

  try {
    await native.bundle({ entries: ['src/a.js'] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('circular_dependency');
    expect(error.details.cycle_path).toEqual(['src/a.js', 'src/b.js', 'src/c.js', 'src/a.js']);
  }
});

test('throws FobError with invalid entry error', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'invalid_entry',
    path: 'src/nonexistent.js',
  });

  try {
    await native.bundle({ entries: ['src/nonexistent.js'] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('invalid_entry');
    expect(error.details.path).toBe('src/nonexistent.js');
  }
});

test('throws FobError with no entries error', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'no_entries',
  });

  try {
    await native.bundle({ entries: [] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('no_entries');
  }
});

test('throws FobError with plugin error', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'plugin',
    name: 'my-plugin',
    message: 'Plugin execution failed',
  });

  try {
    await native.bundle({ entries: ['src/index.js'] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('plugin');
    expect(error.details.name).toBe('my-plugin');
    expect(error.details.message).toBe('Plugin execution failed');
  }
});

test('throws FobError with runtime error', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'runtime',
    message: 'Out of memory',
  });

  try {
    await native.bundle({ entries: ['src/index.js'] });
  } catch (error) {
    expect(error instanceof FobError).toBe(true);
    expect(error.details).toBeTruthy();
    expect(error.details.type).toBe('runtime');
    expect(error.details.message).toBe('Out of memory');
  }
});

test('formatFobError formats MDX error with context', () => {
  const error = {
    type: 'mdx_syntax',
    message: 'Unexpected token <',
    file: 'src/page.mdx',
    line: 5,
    column: 10,
    context: '> 5 | <Component',
    suggestion: 'Check your JSX syntax',
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('MDX Error: Unexpected token <');
  expect(formatted).toContain('in src/page.mdx');
  expect(formatted).toContain('at line 5, column 10');
  expect(formatted).toContain('> 5 | <Component');
  expect(formatted).toContain('💡 Suggestion: Check your JSX syntax');
});

test('formatFobError formats missing export with suggestions', () => {
  const error = {
    type: 'missing_export',
    export_name: 'Button',
    module_id: 'components/Button.tsx',
    available_exports: ['default', 'Icon'],
    suggestion: 'default',
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain("Named export 'Button' not found");
  expect(formatted).toContain("module 'components/Button.tsx'");
  expect(formatted).toContain('Available exports: default, Icon');
  expect(formatted).toContain("Did you mean 'default'?");
});

test('formatFobError formats transform error with diagnostics', () => {
  const error = {
    type: 'transform',
    path: 'src/app.tsx',
    diagnostics: [
      {
        message: 'Type error',
        line: 10,
        column: 5,
        severity: 'error',
        help: 'Check type annotation',
      },
      {
        message: 'Unused variable',
        line: 15,
        column: 8,
        severity: 'warning',
      },
    ],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('Transform failed in src/app.tsx');
  expect(formatted).toContain('Diagnostics:');
  expect(formatted).toContain('[error] Type error (line 10, col 5)');
  expect(formatted).toContain('Help: Check type annotation');
  expect(formatted).toContain('[warning] Unused variable (line 15, col 8)');
});

test('formatFobError formats circular dependency with path', () => {
  const error = {
    type: 'circular_dependency',
    cycle_path: ['src/a.js', 'src/b.js', 'src/c.js', 'src/a.js'],
  };

  const formatted = formatFobError(error);

  expect(formatted).toContain('Circular dependency detected:');
  expect(formatted).toContain('src/a.js → src/b.js → src/c.js → src/a.js');
});

test('FobError.details contains structured error data', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'missing_export',
    export_name: 'Button',
    module_id: 'components/Button.tsx',
    available_exports: ['default', 'Icon'],
    suggestion: 'default',
  });

  try {
    await native.bundle({ entries: ['src/index.js'] });
  } catch (error) {
    expect(error.details).toEqual({
      type: 'missing_export',
      export_name: 'Button',
      module_id: 'components/Button.tsx',
      available_exports: ['default', 'Icon'],
      suggestion: 'default',
    });
  }
});

test('can programmatically access error fields (type switch)', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'mdx_syntax',
    message: 'Parse error',
    file: 'src/page.mdx',
    line: 10,
    column: 5,
  });

  try {
    await native.bundle({ entries: ['src/page.mdx'] });
  } catch (error) {
    if (error.details) {
      switch (error.details.type) {
        case 'mdx_syntax':
          expect(error.details.message).toBe('Parse error');
          expect(error.details.file).toBe('src/page.mdx');
          expect(error.details.line).toBe(10);
          expect(error.details.column).toBe(5);
          break;
        default:
      }
    }
  }
});

test('error.message contains user-friendly formatted text', async () => {
  mockNative.__resetNativeMockState();
  mockNative.__setErrorResponse({
    type: 'circular_dependency',
    cycle_path: ['src/a.js', 'src/b.js', 'src/a.js'],
  });

  try {
    await native.bundle({ entries: ['src/a.js'] });
  } catch (error) {
    expect(error.message).toContain('Circular dependency detected');
    expect(error.message).toContain('src/a.js → src/b.js → src/a.js');
  }
});
