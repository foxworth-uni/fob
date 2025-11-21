import type {
  BundleOptions,
  BundleResult,
  FileSystem,
  ProgressEvent,
} from './types.js';
import { FobError } from './types.js';
import {
  EdgeRuntime,
  installMemoryRuntime,
  installRuntimeFromFileSystem,
} from './runtime/index.js';

// Import the raw WASM bindings with @wasmer/wasi support
import { 
  initialize, 
  bundle as bundleWasm, 
  getRuntimeVersion,
  setFileCache 
} from '../wasm/bundler/fob-bundler.js';

// Import WASI bridge utilities
import { 
  preloadFiles, 
  createWASIFilesystemBindings, 
  flushCache,
  type FileCache 
} from './wasi-bridge.js';

export interface FobOptions {
  wasmUrl?: string;
  wasmBytes?: Uint8Array | ArrayBuffer;
  autoInit?: boolean;
  runtime?: EdgeRuntime;
  files?: Record<string, string | Uint8Array>;
}

export class Fob {
  private initialized = false;
  private runtime: EdgeRuntime | null;
  private fileCache: FileCache | null = null;

  constructor(private readonly options: FobOptions = {}) {
    if (options.files && !options.runtime) {
      this.runtime = installMemoryRuntime(options.files);
    } else {
      this.runtime = options.runtime ?? null;
    }

    if (options.autoInit !== false) {
      void this.init().catch((error) => {
        console.error('Failed to auto-initialize Fob Bundler:', error);
      });
    }
  }

  async init(wasmUrl?: string, wasmBytes?: Uint8Array | ArrayBuffer): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Pre-load files into cache for WASI filesystem
      if (this.runtime?.fs) {
        console.log('[Fob] Pre-loading files for WASI filesystem...');

        // Collect all file paths to pre-load
        const filePaths: string[] = [];
        if (this.options.files) {
          filePaths.push(...Object.keys(this.options.files));
        }

        // Pre-load files from filesystem
        this.fileCache = await preloadFiles(this.runtime.fs, filePaths);
        console.log('[Fob] Pre-loaded', this.fileCache.files.size, 'files');

        // Set the file cache for WASM module
        setFileCache(this.fileCache);
      } else {
        // Create empty cache if no filesystem
        this.fileCache = { files: new Map(), directories: new Set(['/']) };
        setFileCache(this.fileCache);
      }

      // Load the raw WASM module bytes
      let moduleBytes: Uint8Array;

      // Priority 1: Use provided WASM bytes from parameter or options (Cloudflare Workers/Direct provision)
      const providedBytes = wasmBytes ?? this.options.wasmBytes;
      if (providedBytes) {
        console.log('[Fob] Using provided WASM bytes (Cloudflare Workers or direct provision)');
        // Convert ArrayBuffer to Uint8Array if necessary
        moduleBytes = providedBytes instanceof Uint8Array
          ? providedBytes
          : new Uint8Array(providedBytes);
      }
      // Priority 2: Fetch from provided URL
      else {
        const effectiveWasmUrl = wasmUrl ?? this.options.wasmUrl;
        if (effectiveWasmUrl) {
          console.log('[Fob] Fetching WASM from URL:', effectiveWasmUrl);
          const response = await fetch(effectiveWasmUrl);
          if (!response.ok) {
            throw new FobError(
              `Failed to fetch WASM from ${effectiveWasmUrl}: ${response.status} ${response.statusText}`,
              'WASM_FETCH_FAILED'
            );
          }
          moduleBytes = new Uint8Array(await response.arrayBuffer());
        } else {
          // Priority 3: Load from local file (Node.js/Browser default)
          try {
            if (typeof process !== 'undefined' && process.versions?.node) {
              // Node.js environment
              const { readFileSync } = await import('fs');
              const { fileURLToPath } = await import('url');
              const { dirname, join } = await import('path');
              const wasmPath = join(
                dirname(fileURLToPath(import.meta.url)),
                '../wasm/bundler/fob_bundler_wasm.wasm'
              );
              moduleBytes = readFileSync(wasmPath);
            } else {
              // Browser environment: try to fetch using import.meta.url
              const response = await fetch(new URL('../wasm/bundler/fob_bundler_wasm.wasm', import.meta.url));
              if (!response.ok) {
                throw new FobError(
                  `Failed to fetch WASM module: ${response.status} ${response.statusText}`,
                  'WASM_FETCH_FAILED'
                );
              }
              moduleBytes = new Uint8Array(await response.arrayBuffer());
            }
          } catch (e) {
            throw new FobError(
              'WASM module not found. Run `pnpm build:wasm` first, or provide wasmUrl/wasmBytes option. Error: ' + (e as Error).message,
              'WASM_NOT_FOUND'
            );
          }
        }
      }

      // Create WASI filesystem bindings from cache
      const fsBindings = createWASIFilesystemBindings(this.fileCache);

      // Initialize the WASM module with environment-appropriate WASI
      // (Node.js built-in WASI for Node/tests, @wasmer/wasi for browsers/edge)
      console.log('[Fob] Initializing WASM with auto-detected WASI...');
      await initialize(moduleBytes, fsBindings);
      this.initialized = true;
      console.log('[Fob] WASM initialized successfully');
    } catch (error) {
      throw new FobError(
        `Failed to initialize WASM: ${(error as Error).message}`,
        'WASM_INIT_FAILED'
      );
    }
  }

  async bundle(options: BundleOptions, filesystem?: FileSystem): Promise<BundleResult> {
    if (!this.initialized) {
      await this.init();
    }

    if (filesystem) {
      this.runtime = installRuntimeFromFileSystem(filesystem);
      
      // Re-load files for the new filesystem
      if (this.runtime?.fs) {
        const filePaths = options.entries;
        this.fileCache = await preloadFiles(this.runtime.fs, filePaths);
        setFileCache(this.fileCache);
      }
    } else if (!this.runtime) {
      throw new FobError('Filesystem instance is required');
    }

    try {
      // Convert BundleOptions to WASM-compatible config
      const config = {
        entries: options.entries,
        outputDir: options.outputDir || 'dist',
        format: options.format === 'preserve-modules' ? 'esm' : options.format || 'esm',
        sourcemap: options.sourceMaps !== 'none' ? true : false,
      };

      console.log('[Fob] Calling WASM bundle with config:', config);

      // Call WASM bundle function
      const result = await bundleWasm(config);

      // Handle result
      if (result.tag === 'err') {
        throw new FobError(result.val, 'BUNDLE_FAILED');
      }

      // Flush cache back to filesystem (write output files)
      if (this.fileCache && this.runtime?.fs) {
        console.log('[Fob] Flushing cache back to filesystem...');
        await flushCache(this.fileCache, this.runtime.fs);
      }

      // WASM integration complete - currently returns minimal bundle metadata
      // Full chunk/manifest parsing will be implemented when Rust side returns detailed bundle data
      const bundleResult: BundleResult = {
        chunks: [],
        manifest: {
          entries: {},
          chunks: {},
          version: getRuntimeVersion(),
        },
        stats: {
          totalModules: 0,
          totalChunks: result.val.assetsCount || 0,
          totalSize: 0,
          durationMs: 0,
          cacheHitRate: 0,
        },
        assets: [],
      };

      return bundleResult;
    } catch (error) {
      if (error instanceof FobError) {
        throw error;
      }
      throw new FobError(`Bundle failed: ${(error as Error).message}`);
    }
  }

  async bundleWithProgress(
    options: BundleOptions,
    filesystem: FileSystem,
    onProgress: (event: ProgressEvent) => void
  ): Promise<BundleResult> {
    void onProgress;
    return this.bundle(options, filesystem);
  }

  async bundleInMemory(
    files: Record<string, string | Uint8Array>,
    options: BundleOptions
  ): Promise<BundleResult> {
    const runtime = installMemoryRuntime(files);
    const prevRuntime = this.runtime;
    this.runtime = runtime;
    try {
      return await this.bundle(options);
    } finally {
      this.runtime = prevRuntime;
    }
  }

  updateFiles(files: Record<string, string | Uint8Array>): void {
    if (!this.runtime?.fs) {
      throw new FobError('No filesystem initialized');
    }

    for (const [path, content] of Object.entries(files)) {
      const bytes = typeof content === 'string' ? new TextEncoder().encode(content) : content;
      void this.runtime.fs.write(path, bytes);
    }
  }

  isInitialized(): boolean {
    return this.initialized;
  }
}

export async function bundle(
  options: BundleOptions,
  filesystem: FileSystem
): Promise<BundleResult> {
  const bundler = new Fob({ autoInit: true });
  await bundler.init();
  const result = await bundler.bundle(options, filesystem);
  normalizeBundleStats(result);
  return result;
}

export async function bundleInMemory(
  files: Record<string, string | Uint8Array>,
  options: BundleOptions,
  wasmBytes?: Uint8Array | ArrayBuffer
): Promise<BundleResult> {
  const bundler = new Fob({ files, wasmBytes, autoInit: false });
  await bundler.init(undefined, wasmBytes);
  const result = await bundler.bundle(options);
  normalizeBundleStats(result);
  return result;
}

/**
 * Get the fob-edge package version
 * This returns a static version string and doesn't require WASM initialization
 */
export function version(): string {
  return '0.1.0'; // Static package version
}

/**
 * Get the runtime version from the WASM module
 * Requires WASM to be initialized first
 */
export function wasmVersion(): string {
  return getRuntimeVersion();
}

function normalizeBundleStats(result: unknown): void {
  if (typeof result !== 'object' || result === null) {
    return;
  }

  const statsContainer = result as { stats?: unknown };
  if (typeof statsContainer.stats !== 'object' || statsContainer.stats === null) {
    return;
  }

  const stats = statsContainer.stats as Record<string, unknown>;
  const ensureAlias = (camel: string, snake: string) => {
    if (stats[camel] === undefined && stats[snake] !== undefined) {
      stats[camel] = stats[snake];
    }
  };

  ensureAlias('totalModules', 'total_modules');
  ensureAlias('totalChunks', 'total_chunks');
  ensureAlias('totalSize', 'total_size');
  ensureAlias('durationMs', 'duration_ms');
  ensureAlias('cacheHitRate', 'cache_hit_rate');
}
