import { test, expect, describe, beforeEach, afterEach } from 'vitest';
import { bundle } from '../dist/native/index.js';
import { FobError } from '../dist/types.js';
import { mkdirSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const TMP_DIR = join(process.cwd(), 'tests/temp-error-handling');

function createTestFiles(files) {
  if (!existsSync(TMP_DIR)) {
    mkdirSync(TMP_DIR, { recursive: true });
  }
  for (const [name, content] of Object.entries(files)) {
    writeFileSync(join(TMP_DIR, name), content);
  }
}

describe('Error Handling Integration Tests', () => {
  beforeEach(() => {
    try {
      rmSync(TMP_DIR, { recursive: true, force: true });
    } catch {}
    mkdirSync(TMP_DIR, { recursive: true });
  });

  afterEach(() => {
    try {
      rmSync(TMP_DIR, { recursive: true, force: true });
    } catch {}
  });

  test('throws FobError with missing export error', async () => {
    createTestFiles({
      'index.js': 'import { NonExistent } from "./other.js";',
      'other.js': 'export const Foo = 1;',
    });

    try {
      await bundle({
        entries: [join(TMP_DIR, 'index.js')],
        outputDir: join(TMP_DIR, 'dist'),
      });
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeInstanceOf(FobError);
      expect(error.details).toBeTruthy();
      expect(error.details.type).toBe('missing_export');
      expect(error.details.export_name).toBeTruthy();
      expect(error.details.module_id).toBeTruthy();
      expect(error.details.available_exports).toBeInstanceOf(Array);
      // The export name should contain "NonExistent" or similar
      expect(error.details.export_name.toLowerCase()).toMatch(/nonexistent|non.existent/i);
    }
  });

  test('throws FobError with syntax error (transform)', async () => {
    createTestFiles({
      'index.js': 'const x = ;', // Syntax error
    });

    try {
      await bundle({
        entries: [join(TMP_DIR, 'index.js')],
        outputDir: join(TMP_DIR, 'dist'),
      });
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeInstanceOf(FobError);
      expect(error.details).toBeTruthy();
      // Should be either transform or runtime error depending on how Rolldown reports it
      expect(['transform', 'runtime']).toContain(error.details.type);
      if (error.details.type === 'transform') {
        expect(error.details.path).toBeTruthy();
        expect(error.details.diagnostics).toBeInstanceOf(Array);
        expect(error.details.diagnostics.length).toBeGreaterThan(0);
        expect(error.details.diagnostics[0].line).toBeGreaterThan(0);
        expect(error.details.diagnostics[0].message).toBeTruthy();
      }
    }
  });

  test('throws FobError with invalid entry error', async () => {
    const invalidPath = join(TMP_DIR, 'nonexistent.js');
    try {
      await bundle({
        entries: [invalidPath],
        outputDir: join(TMP_DIR, 'dist'),
      });
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeInstanceOf(FobError);
      expect(error.details).toBeTruthy();
      expect(error.details.type).toBe('invalid_entry');
      // The path in the error might be the full message or extracted path depending on my implementation
      // In lib.rs I used msg.clone() as path, so it will be the full message.
      expect(error.details.path).toContain('nonexistent.js');
    }
  });

  test('throws FobError with no entries error', async () => {
    try {
      await bundle({
        entries: [],
        outputDir: join(TMP_DIR, 'dist'),
      });
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeInstanceOf(FobError);
      expect(error.details).toBeTruthy();
      expect(error.details.type).toBe('no_entries');
    }
  });

  test('handles multiple errors when present', async () => {
    createTestFiles({
      'index.js': `
        import { NonExistent1 } from "./other1.js";
        import { NonExistent2 } from "./other2.js";
      `,
      'other1.js': 'export const Foo = 1;',
      'other2.js': 'export const Bar = 2;',
    });

    try {
      await bundle({
        entries: [join(TMP_DIR, 'index.js')],
        outputDir: join(TMP_DIR, 'dist'),
      });
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeInstanceOf(FobError);
      // If multiple errors are reported, they might be in a Multiple variant
      // or as separate errors. Check if details exist.
      if (error.details && error.details.type === 'multiple') {
        expect(error.details.errors).toBeInstanceOf(Array);
        expect(error.details.errors.length).toBeGreaterThan(0);
        expect(error.details.primary_message).toBeTruthy();
      } else {
        // Single error is also acceptable
        expect(error.details).toBeTruthy();
      }
    }
  });

  test('handles circular dependency errors', async () => {
    createTestFiles({
      'a.js': 'import { b } from "./b.js"; export const a = 1;',
      'b.js': 'import { a } from "./a.js"; export const b = 2;',
      'index.js': 'import { a } from "./a.js";',
    });

    try {
      await bundle({
        entries: [join(TMP_DIR, 'index.js')],
        outputDir: join(TMP_DIR, 'dist'),
      });
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error).toBeInstanceOf(FobError);
      // Circular dependency might be reported as runtime error or specific type
      expect(error.details).toBeTruthy();
      // Check if it's a circular dependency error or runtime error mentioning cycle
      if (error.details.type === 'circular_dependency') {
        expect(error.details.cycle_path).toBeInstanceOf(Array);
        expect(error.details.cycle_path.length).toBeGreaterThan(0);
      } else {
        // Might be reported as runtime error
        expect(['circular_dependency', 'runtime']).toContain(error.details.type);
      }
    }
  });
});
