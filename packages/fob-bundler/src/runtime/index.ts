import type { FileSystem } from '../types.js';
import { DiskCache, type DiskCacheOptions } from './disk-cache.js';
import { MemoryCache } from './memory-cache.js';
import { MemoryFileSystem } from './memory-fs.js';
import { NativeFileSystem, type NativeFileSystemOptions } from './native-fs.js';

export interface NodeRuntime {
  fs: FileSystem;
  cache: DiskCache | MemoryCache;
}

export interface InstallNodeRuntimeOptions {
  filesystem?: FileSystem;
  fsOptions?: NativeFileSystemOptions;
  cache?: DiskCache;
  cacheOptions?: DiskCacheOptions;
}

export function installNodeRuntime(options: InstallNodeRuntimeOptions = {}): NodeRuntime {
  const fsImpl = options.filesystem ?? new NativeFileSystem(options.fsOptions);
  const cacheImpl = options.cache ?? new DiskCache(options.cacheOptions);

  (globalThis as unknown as { __fobRuntime: unknown }).__fobRuntime = {
    fs: {
      read: (path: string) => fsImpl.read(path),
      write: (path: string, content: Uint8Array) => fsImpl.write(path, content),
      metadata: (path: string) => fsImpl.metadata(path),
      exists: (path: string) => fsImpl.exists(path),
    },
    cache: {
      get: (key: string) => cacheImpl.getSync(key),
      set: (key: string, value: Uint8Array) => cacheImpl.setSync(key, value),
      delete: (key: string) => cacheImpl.deleteSync(key),
      clear: () => cacheImpl.clearSync(),
    },
  };

  return { fs: fsImpl, cache: cacheImpl };
}

export function installMemoryRuntime(files: Record<string, string | Uint8Array> = {}): NodeRuntime {
  const fs = MemoryFileSystem.fromObject(files);
  const cache = new MemoryCache();

  (globalThis as unknown as { __fobRuntime: unknown }).__fobRuntime = {
    fs: {
      read: (path: string) => fs.read(path),
      write: (path: string, content: Uint8Array) => fs.write(path, content),
      metadata: (path: string) => fs.metadata(path),
      exists: (path: string) => fs.exists(path),
    },
    cache: {
      get: (key: string) => cache.getSync(key),
      set: (key: string, value: Uint8Array) => cache.setSync(key, value),
      delete: (key: string) => cache.deleteSync(key),
      clear: () => cache.clearSync(),
    },
  };

  return { fs, cache };
}

export { DiskCache, MemoryCache, MemoryFileSystem, NativeFileSystem };
