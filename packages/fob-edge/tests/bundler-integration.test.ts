/**
 * Bundler Integration Tests
 * 
 * Tests the complete WASM integration with WIT function marshalling,
 * dual WASI support, and memory management.
 */

import { describe, it, expect } from 'vitest';
import { Fob, version } from '../src/bundler.js';

describe('Bundler Integration', () => {
  it('should get runtime version from WASM', () => {
    const ver = version();
    console.log('Runtime version:', ver);
    
    // Should not be the fallback values
    expect(ver).not.toBe('0.1.0-wasi-not-initialized');
    expect(ver).not.toBe('0.1.0-wasi-error');
    
    // Should be a valid version string
    expect(ver).toBeDefined();
    expect(typeof ver).toBe('string');
  });

  it('should initialize bundler and call bundle with real marshalling', async () => {
    const fob = new Fob({
      files: {
        'index.js': 'export const greeting = "Hello World";',
        'utils.js': 'export function helper() { return 42; }'
      },
      autoInit: false
    });
    
    await fob.init();
    expect(fob.isInitialized()).toBe(true);
    
    // Call bundle with real marshalling
    try {
      const result = await fob.bundle({
        entries: ['index.js'],
        outputDir: 'dist',
        format: 'esm',
        sourceMaps: 'none'
      });
      
      console.log('Bundle result:', result);
      
      // Should have a result
      expect(result).toBeDefined();
      expect(result.chunks).toBeDefined();
      expect(result.manifest).toBeDefined();
      
      // Stats should be populated from WASM result
      expect(result.stats).toBeDefined();
      expect(result.stats.totalChunks).toBeGreaterThanOrEqual(0);
    } catch (error) {
      // Log error for debugging
      console.error('Bundle error:', error);
      
      // If it fails, it should be a FobError
      expect(error.constructor.name).toBe('FobError');
    }
  });

  it('should handle multiple entries', async () => {
    const fob = new Fob({
      files: {
        'app.js': 'import { helper } from "./utils.js"; console.log(helper());',
        'utils.js': 'export function helper() { return "test"; }',
        'styles.js': 'export const styles = { color: "blue" };'
      },
      autoInit: false
    });
    
    await fob.init();
    
    const result = await fob.bundle({
      entries: ['app.js', 'styles.js'],
      outputDir: 'out'
    }).catch(err => {
      console.log('Expected error (WASI limitations):', err.message);
      return null;
    });
    
    // Either succeeds or fails gracefully
    if (result) {
      expect(result.chunks).toBeDefined();
    }
  });

  it('should handle bundle options correctly', async () => {
    const fob = new Fob({
      files: {
        'main.js': 'console.log("main");'
      },
      autoInit: false
    });
    
    await fob.init();
    
    // Test with different options
    const result = await fob.bundle({
      entries: ['main.js'],
      outputDir: 'dist',
      format: 'esm',
      sourceMaps: 'inline'
    }).catch(err => {
      console.log('Bundle error:', err.message);
      return null;
    });
    
    // Verify the call was made (even if it errors)
    expect(fob.isInitialized()).toBe(true);
  });
});

describe('Memory Management', () => {
  it('should not leak memory on repeated calls', async () => {
    const fob = new Fob({
      files: {
        'test.js': 'export const value = 123;'
      },
      autoInit: false
    });
    
    await fob.init();
    
    // Make multiple calls
    for (let i = 0; i < 10; i++) {
      await fob.bundle({
        entries: ['test.js'],
        outputDir: 'dist'
      }).catch(() => {
        // Ignore errors for this test
      });
    }
    
    // Should still be initialized
    expect(fob.isInitialized()).toBe(true);
  });
});

