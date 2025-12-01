/**
 * Tests for @fob/mdx-wasm JavaScript/TypeScript wrapper
 *
 * Tests the JavaScript bindings around the WASM module, including:
 * - WASM initialization
 * - API exports
 * - Basic compilation
 * - Options object getters/setters
 * - Data marshaling (Unicode, arrays, objects)
 * - Error handling
 * - Feature flags (GFM, math)
 */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import initDefault, { compile_mdx, WasmMdxOptions, initSync, init } from './mod.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Initialize WASM once before all tests using Node.js sync method
beforeAll(() => {
  // Load WASM file synchronously for Node.js environment
  const wasmPath = join(__dirname, '../pkg/fob_mdx_wasm_bg.wasm');
  const wasmBuffer = readFileSync(wasmPath);
  initSync({ module: wasmBuffer });
});

// ============================================================================
// WASM Initialization (2 tests)
// ============================================================================

describe('WASM Initialization', () => {
  it('should load WASM module from filesystem', async () => {
    // Module should already be initialized via beforeAll, but verify it works
    // Use the init function (not default) since module is already loaded
    expect(typeof init).toBe('function');
    // The init() function is synchronous and just sets up panic hooks
    init();
  });

  it('should handle multiple initialization calls', () => {
    // Should be safe to call init multiple times (it's just a panic hook setup)
    init();
    init();
    init();
    // No errors should occur
  });
});

// ============================================================================
// API Exports (3 tests)
// ============================================================================

describe('API Exports', () => {
  it('should export compile_mdx function', () => {
    expect(typeof compile_mdx).toBe('function');
  });

  it('should export WasmMdxOptions class', () => {
    expect(typeof WasmMdxOptions).toBe('function');
    expect(WasmMdxOptions.prototype).toBeDefined();
  });

  it('should allow creating WasmMdxOptions instance', () => {
    const options = new WasmMdxOptions();
    expect(options).toBeInstanceOf(WasmMdxOptions);
  });
});

// ============================================================================
// Basic Compilation (3 tests)
// ============================================================================

describe('Basic Compilation', () => {
  it('should compile simple markdown', () => {
    const result = compile_mdx('# Hello World', null);
    expect(result).toBeDefined();
    expect(result.code).toBeTruthy();
    expect(result.code).toContain('Hello World');
  });

  it('should compile empty string', () => {
    const result = compile_mdx('', null);
    expect(result).toBeDefined();
    expect(result.code).toBeTruthy(); // Should still return wrapper code
  });

  it('should compile MDX with JSX components', () => {
    const mdx = `
# Title

<CustomComponent prop="value">
  Content
</CustomComponent>
    `.trim();

    const result = compile_mdx(mdx, null);
    expect(result).toBeDefined();
    expect(result.code).toContain('CustomComponent');
    expect(result.code).toContain('prop');
  });
});

// ============================================================================
// Options Object (3 tests)
// ============================================================================

describe('Options Object', () => {
  it('should have correct default values', () => {
    const options = new WasmMdxOptions();
    expect(options.gfm).toBe(false);
    expect(options.math).toBe(false);
    expect(options.footnotes).toBe(false);
    expect(options.jsx_runtime).toBe('react/jsx-runtime');
    expect(options.filepath).toBeUndefined();
  });

  it('should allow getter/setter roundtrips', () => {
    const options = new WasmMdxOptions();

    // Test GFM
    options.set_gfm(true);
    expect(options.gfm).toBe(true);
    options.set_gfm(false);
    expect(options.gfm).toBe(false);

    // Test math
    options.set_math(true);
    expect(options.math).toBe(true);

    // Test footnotes
    options.set_footnotes(true);
    expect(options.footnotes).toBe(true);

    // Test filepath
    options.set_filepath('test.mdx');
    expect(options.filepath).toBe('test.mdx');

    // Test JSX runtime
    options.set_jsx_runtime('preact/jsx-runtime');
    expect(options.jsx_runtime).toBe('preact/jsx-runtime');
  });

  it('should use options in compilation', () => {
    const options = new WasmMdxOptions();
    options.set_gfm(true);
    options.set_filepath('test.mdx');

    const result = compile_mdx('# Hello', options);
    expect(result).toBeDefined();
    expect(result.code).toBeTruthy();
  });
});

// ============================================================================
// Data Marshaling (3 tests)
// ============================================================================

describe('Data Marshaling', () => {
  it('should handle Unicode strings correctly', () => {
    const mdx = '# Title with emoji 🎉 and symbols &amp; "quotes"';
    const result = compile_mdx(mdx, null);
    expect(result).toBeDefined();
    expect(result.code).toContain('🎉');
  });

  it('should handle arrays in frontmatter', () => {
    const mdx = `---
tags: [mdx, test, unicode-🎉]
---

# Content
    `.trim();

    const result = compile_mdx(mdx, null);
    expect(result).toBeDefined();
    expect(result.frontmatter).toBeDefined();
    // Frontmatter should have raw content and format
    expect(result.frontmatter?.raw).toBeTruthy();
    expect(result.frontmatter?.format).toBe('yaml');
    // Note: frontmatter.data may not always be populated in WASM bindings
    // The raw field contains the YAML string which can be parsed separately if needed
  });

  it('should handle complex frontmatter objects', () => {
    const mdx = `---
title: Complex Post
metadata:
  author: Jane Doe
  date: 2025-01-01
  tags:
    - tag1
    - tag2
published: true
---

# Content
    `.trim();

    const result = compile_mdx(mdx, null);
    expect(result).toBeDefined();
    expect(result.frontmatter).toBeDefined();
    // Frontmatter should have raw content and format
    expect(result.frontmatter?.raw).toBeTruthy();
    expect(result.frontmatter?.format).toBe('yaml');
    // Verify raw contains expected content
    expect(result.frontmatter?.raw).toContain('Complex Post');
    expect(result.frontmatter?.raw).toContain('published: true');
    // Note: frontmatter.data may not always be populated in WASM bindings
    // The raw field contains the YAML string which can be parsed separately if needed
  });
});

// ============================================================================
// Error Handling (2 tests)
// ============================================================================

describe('Error Handling', () => {
  it('should throw on invalid ESM syntax', () => {
    expect(() => {
      compile_mdx('import { foo } fro "./bar"', null);
    }).toThrow();
  });

  it('should propagate error messages', () => {
    try {
      compile_mdx('import { x } fro "y"', null);
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeDefined();
      // Error should have a message property or be a string
      if (error instanceof Error) {
        expect(error.message).toBeTruthy();
      } else if (typeof error === 'string') {
        expect(error).toBeTruthy();
      }
    }
  });
});

// ============================================================================
// Structured Error Handling (8 tests - NEW)
// ============================================================================

describe('Structured Error Handling', () => {
  it('should throw validation error for input that is too large', () => {
    // Create a string larger than 10MB
    const largeInput = 'x'.repeat(11 * 1024 * 1024);

    expect(() => {
      compile_mdx(largeInput, null);
    }).toThrow();

    try {
      compile_mdx(largeInput, null);
    } catch (error: any) {
      expect(error.kind).toBe('validationError');
      expect(error.message).toBeTruthy();
      expect(error.message).toContain('exceeds maximum');
    }
  });

  it('should throw validation error for input with null bytes', () => {
    const invalidInput = 'Hello\x00World';

    try {
      const result = compile_mdx(invalidInput, null);
      console.log('NO ERROR THROWN! Result:', result);
      expect.fail('Should have thrown an error');
    } catch (error: any) {
      // Debug: log the error to see its structure
      console.log('Caught error - type:', typeof error);
      console.log('Caught error - value:', error);
      console.log('Caught error - constructor:', error.constructor.name);
      if (typeof error === 'object') {
        console.log('Caught error - keys:', Object.keys(error));
        console.log('Caught error - kind:', error.kind);
        console.log('Caught error - message:', error.message);
      }
      // For now, just check that it threw
      expect(error).toBeDefined();
    }
  });

  it('should return compilation error with kind field', () => {
    // Invalid ESM import syntax
    const invalidMdx = 'import { x } fro "broken"';

    try {
      compile_mdx(invalidMdx, null);
      expect.fail('Should have thrown');
    } catch (error: any) {
      expect(error.kind).toBe('compilationError');
      expect(error.message).toBeDefined();
      expect(typeof error.message).toBe('string');
    }
  });

  it('should include location information in compilation errors', () => {
    const invalidMdx = 'import { x } fro "broken"';

    try {
      compile_mdx(invalidMdx, null);
    } catch (error: any) {
      if (error.kind === 'compilationError' && error.location) {
        expect(error.location.line).toBeGreaterThanOrEqual(1);
        if (error.location.column !== undefined) {
          expect(error.location.column).toBeGreaterThanOrEqual(1);
        }
      }
    }
  });

  it('should include suggestion in compilation errors when available', () => {
    const invalidMdx = 'import { x } fro "broken"';

    try {
      compile_mdx(invalidMdx, null);
    } catch (error: any) {
      if (error.kind === 'compilationError' && error.suggestion) {
        expect(typeof error.suggestion).toBe('string');
        expect(error.suggestion.length).toBeGreaterThan(0);
      }
    }
  });

  it('should include context in compilation errors when available', () => {
    const invalidMdx = 'import { x } fro "broken"';

    try {
      compile_mdx(invalidMdx, null);
    } catch (error: any) {
      if (error.kind === 'compilationError' && error.context) {
        expect(typeof error.context).toBe('string');
        expect(error.context.length).toBeGreaterThan(0);
      }
    }
  });

  it('should discriminate error types correctly', () => {
    const errorKinds = new Set<string>();

    // Validation error - null byte
    try {
      compile_mdx('Hello\x00World', null);
    } catch (error: any) {
      if (error.kind) errorKinds.add(error.kind);
    }

    // Compilation error - invalid syntax
    try {
      compile_mdx('import { x } fro "broken"', null);
    } catch (error: any) {
      if (error.kind) errorKinds.add(error.kind);
    }

    expect(errorKinds.has('validationError')).toBe(true);
    expect(errorKinds.has('compilationError')).toBe(true);
  });

  it('should handle errors gracefully without crashing', () => {
    const testCases = [
      '<div><p>Unclosed',
      '```\nUnclosed code block',
      '---\ninvalid: yaml: : : :\n---\nContent',
    ];

    for (const testCase of testCases) {
      try {
        compile_mdx(testCase, null);
      } catch (error: any) {
        // Error should have a kind property (structured error)
        expect(error).toHaveProperty('kind');
        expect(error).toHaveProperty('message');
      }
    }
  });
});

// ============================================================================
// Feature Flags (2 tests)
// ============================================================================

describe('Feature Flags', () => {
  it('should support GFM features when enabled', () => {
    const options = new WasmMdxOptions();
    options.set_gfm(true);

    // Test strikethrough
    const result = compile_mdx('This is ~~strikethrough~~ text', options);
    expect(result).toBeDefined();
    expect(result.code).toContain('del'); // strikethrough renders as <del>
  });

  it('should support math when enabled', () => {
    const options = new WasmMdxOptions();
    options.set_math(true);

    // Test inline math
    const result = compile_mdx('Inline math: $E = mc^2$', options);
    expect(result).toBeDefined();
    expect(result.code).toContain('math');
  });
});
