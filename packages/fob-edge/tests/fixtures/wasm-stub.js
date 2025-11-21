/**
 * WASM stub for testing @fob/edge
 * Simulates the fob-bundler.js bindings without actual WASM
 * Returns WIT-compatible result types
 */

// Stub global state
let wasmInitialized = false;
let fileCache = null;

/**
 * Set file cache (stub)
 */
export function setFileCache(cache) {
  fileCache = cache;
}

/**
 * Get file cache (stub)
 */
export function getFileCache() {
  return fileCache;
}

/**
 * Initialize WASM (stub - just sets flag)
 */
export async function initialize(_wasmBytes, _fsBindings) {
  console.log('[WASM-STUB] Initialize called');
  wasmInitialized = true;
  return Promise.resolve();
}

/**
 * Bundle function (stub - returns WIT result<bundle-result, string>)
 */
export async function bundle(config) {
  console.log('[WASM-STUB] Bundle called with config:', config);
  
  if (!wasmInitialized) {
    return {
      tag: 'err',
      val: 'WASM not initialized'
    };
  }
  
  // Return WIT result<bundle-result, string> format
  return {
    tag: 'ok',
    val: {
      assetsCount: config.entries?.length || 1,
      success: true,
      error: null
    }
  };
}

/**
 * Get runtime version (stub)
 */
export function getRuntimeVersion() {
  return 'wasm-stub-0.1.0';
}

// Legacy exports for backward compatibility (if needed)
export default async function init() {
  await initialize(new Uint8Array(), {});
  return Promise.resolve();
}

export class Fob {
  constructor(options) {
    this.options = options;
  }

  async bundle() {
    // Return minimal valid BundleResult (legacy format)
    return {
      chunks: [
        {
          id: 'stub-chunk',
          kind: 'entry',
          fileName: 'index-stub.js',
          code: 'export default "stub";',
          modules: [
            {
              path: this.options.entries?.[0] || '/src/index.js',
              size: 100,
              hasSideEffects: false,
            },
          ],
          imports: [],
          dynamicImports: [],
          size: 100,
        },
      ],
      manifest: {
        entries: {
          [this.options.entries?.[0] || '/src/index.js']: 'index-stub.js',
        },
        chunks: {
          'stub-chunk': {
            file: 'index-stub.js',
          },
        },
        version: 'wasm-stub',
      },
      stats: {
        totalModules: 1,
        totalChunks: 1,
        totalSize: 100,
        durationMs: 0,
        cacheHitRate: 0,
      },
    };
  }
}
