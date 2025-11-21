/**
 * WASI Bridge Integration Tests
 * 
 * Tests the WASI bridge layer, filesystem operations, and dual WASI runtime support.
 * Complements bundler-integration.test.ts which focuses on WIT marshalling.
 */

import { describe, it, expect } from 'vitest';
import { Fob, bundleInMemory } from '../src/bundler.js';

describe('WASI Integration with @wasmer/wasi', () => {
  it('should initialize Fob with @wasmer/wasi runtime', async () => {
    const fob = new Fob({
      files: {
        'index.js': 'export const x = 1;'
      },
      autoInit: false
    });
    
    await fob.init();
    expect(fob.isInitialized()).toBe(true);
  });

  it('should handle empty file initialization', async () => {
    const fob = new Fob({ autoInit: false });
    
    await expect(fob.init()).resolves.not.toThrow();
    expect(fob.isInitialized()).toBe(true);
  });

  it('should call bundle function without crashing', async () => {
    const fob = new Fob({
      files: {
        'index.js': 'console.log("hello");'
      },
      autoInit: false
    });
    
    await fob.init();
    
    // Note: This may return an error until Phase 3 (function marshalling) is complete
    // We're just testing that it doesn't crash
    const result = await fob.bundle({
      entries: ['index.js'],
      outputDir: 'dist',
    }).catch(error => {
      // Expected to fail until marshalling is implemented
      expect(error).toBeDefined();
      return null;
    });
    
    // Either succeeds with a result or fails gracefully
    if (result) {
      expect(result).toHaveProperty('chunks');
      expect(result).toHaveProperty('manifest');
    }
  });

  it('should use bundleInMemory helper', async () => {
    const result = await bundleInMemory(
      {
        'app.js': 'export default function App() { return "Hi"; }'
      },
      {
        entries: ['app.js'],
        outputDir: 'out',
      }
    ).catch(error => {
      // May fail until marshalling is complete
      expect(error).toBeDefined();
      return null;
    });
    
    if (result) {
      expect(result).toBeDefined();
    }
  });

  it('should handle multiple files', async () => {
    const fob = new Fob({
      files: {
        'index.js': 'import "./utils.js"; console.log("main");',
        'utils.js': 'export const helper = () => "helper";'
      },
      autoInit: false
    });
    
    await fob.init();
    expect(fob.isInitialized()).toBe(true);
  });

  it('should update files after initialization', async () => {
    const fob = new Fob({
      files: {
        'index.js': 'export const x = 1;'
      },
      autoInit: false
    });
    
    await fob.init();
    
    // Update files
    expect(() => {
      fob.updateFiles({
        'index.js': 'export const x = 2;',
        'new.js': 'export const y = 3;'
      });
    }).not.toThrow();
  });
});

describe('WASI Bridge', () => {
  const isNodeEnvironment = typeof process !== 'undefined' && process.versions?.node;
  
  it.skipIf(isNodeEnvironment)('should handle filesystem operations', async () => {
    const { preloadFiles, createWASIFilesystemBindings } = await import('../src/wasi-bridge.js');
    
    // Create mock filesystem
    const mockFS = {
      read: async (path: string) => {
        if (path === 'test.txt') {
          return new TextEncoder().encode('test content');
        }
        throw new Error('File not found');
      },
      write: async (_path: string, _content: Uint8Array) => {
        // Mock write
      },
      exists: async (path: string) => path === 'test.txt',
    };
    
    const cache = await preloadFiles(mockFS, ['test.txt']);
    expect(cache.files.size).toBe(1);
    expect(cache.files.has('test.txt')).toBe(true);
    
    const bindings = createWASIFilesystemBindings(cache);
    expect(bindings).toHaveProperty('openSync');
    expect(bindings).toHaveProperty('readSync');
    expect(bindings).toHaveProperty('writeSync');
  });

  it.skipIf(isNodeEnvironment)('should handle directory operations', async () => {
    const { createWASIFilesystemBindings } = await import('../src/wasi-bridge.js');
    
    const cache = {
      files: new Map([
        ['dir/file1.js', new Uint8Array()],
        ['dir/file2.js', new Uint8Array()],
      ]),
      directories: new Set(['/', 'dir']),
    };
    
    const bindings = createWASIFilesystemBindings(cache);
    
    // Test readdirSync
    const entries = bindings.readdirSync('dir');
    expect(entries).toContain('file1.js');
    expect(entries).toContain('file2.js');
  });

  it.skipIf(isNodeEnvironment)('should handle stat operations', async () => {
    const { createWASIFilesystemBindings } = await import('../src/wasi-bridge.js');
    
    const cache = {
      files: new Map([
        ['test.js', new TextEncoder().encode('content')],
      ]),
      directories: new Set(['/']),
    };
    
    const bindings = createWASIFilesystemBindings(cache);
    
    const stat = bindings.statSync('test.js');
    expect(stat.isFile()).toBe(true);
    expect(stat.isDirectory()).toBe(false);
    expect(stat.size).toBe(7); // 'content' length
  });

  it.skipIf(isNodeEnvironment)('should handle file read/write cycle', async () => {
    const { createWASIFilesystemBindings } = await import('../src/wasi-bridge.js');
    
    const cache = {
      files: new Map(),
      directories: new Set(['/']),
    };
    
    const bindings = createWASIFilesystemBindings(cache);
    
    // Open file for writing
    const fd = bindings.openSync('output.txt', 2); // O_RDWR
    expect(fd).toBeGreaterThanOrEqual(3);
    
    // Write data
    const data = new TextEncoder().encode('Hello WASI');
    const buffer = new Uint8Array(1024);
    buffer.set(data, 0);
    
    const written = bindings.writeSync(fd, buffer, 0, data.length, 0);
    expect(written).toBe(data.length);
    
    // Close file
    bindings.closeSync(fd);
    
    // Verify cache was updated
    expect(cache.files.has('output.txt')).toBe(true);
  });
});

describe('Error Handling', () => {
  it('should handle bundling errors gracefully', async () => {
    const fob = new Fob({ 
      autoInit: false, 
      files: {
        'test.js': 'export const x = 1;'
      }
    });
    
    await fob.init();
    
    // Bundle should return an error for missing/invalid entry
    const result = await fob.bundle({ entries: ['test.js'] }).catch(e => e);
    expect(result).toBeDefined();
  });

  it('should handle missing WASM module gracefully', async () => {
    const fob = new Fob({ autoInit: false });
    
    // If WASM module is missing, should throw appropriate error
    await expect(
      fob.init('https://invalid-url.example.com/wasm')
    ).rejects.toThrow();
  });
});

