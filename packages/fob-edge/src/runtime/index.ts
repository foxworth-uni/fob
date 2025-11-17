import type { FileSystem } from '../types.js';
import { MemoryCache } from './memory-cache.js';
import { FetchFileSystem, type FetchFileSystemOptions } from './fetch-fs.js';
import { MemoryFileSystem } from './memory-fs.js';

export interface EdgeRuntime {
  fs: FileSystem;
  cache: MemoryCache;
}

export function attachRuntime(runtime: EdgeRuntime): EdgeRuntime {
  const { fs, cache } = runtime;
  (globalThis as unknown as { __fobRuntime: unknown }).__fobRuntime = {
    fs: {
      read: (path: string) => fs.read(path),
      write: (path: string, content: Uint8Array) => fs.write(path, content),
      metadata: (path: string) => fs.metadata(path),
      exists: (path: string) => fs.exists(path),
    },
    cache: {
      get: (key: string) => cache.get(key),
      set: (key: string, value: Uint8Array) => cache.set(key, value),
      delete: (key: string) => cache.delete(key),
      clear: () => cache.clear(),
    },
  };

  return runtime;
}

export function installMemoryRuntime(
  initialFiles: Record<string, string | Uint8Array> = {}
): EdgeRuntime {
  return attachRuntime({
    fs: new MemoryFileSystem(initialFiles),
    cache: new MemoryCache(),
  });
}

export function installFetchRuntime(options: FetchFileSystemOptions = {}): EdgeRuntime {
  return attachRuntime({
    fs: new FetchFileSystem(options),
    cache: new MemoryCache(),
  });
}

export function installRuntimeFromFileSystem(fs: FileSystem): EdgeRuntime {
  return attachRuntime({ fs, cache: new MemoryCache() });
}

export { FetchFileSystem, type FetchFileSystemOptions, MemoryCache, MemoryFileSystem };
