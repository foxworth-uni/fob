/**
 * WASI Bridge Module
 * 
 * Bridges between @wasmer/wasi and the Edge runtime FileSystem.
 * Handles the async/sync mismatch by pre-loading files into memory.
 */

import type { FileSystem } from './types.js';

export interface FileCache {
  files: Map<string, Uint8Array>;
  directories: Set<string>;
  rootPath?: string; // For Node.js WASI preopens
}

export interface WASIBridgeOptions {
  filesystem?: FileSystem;
  fileCache?: FileCache;
  args?: string[];
  env?: Record<string, string>;
}

/**
 * Detect if we should use Node.js built-in WASI
 */
export function shouldUseNodeWASI(): boolean {
  return typeof process !== 'undefined' && 
         process.versions?.node !== undefined;
}

/**
 * Pre-load files from async FileSystem into a synchronous cache
 */
export async function preloadFiles(
  filesystem: FileSystem,
  paths: string[]
): Promise<FileCache> {
  const files = new Map<string, Uint8Array>();
  const directories = new Set<string>();

  for (const path of paths) {
    try {
      if (await filesystem.exists(path)) {
        const content = await filesystem.read(path);
        files.set(path, content);
        
        // Track parent directories
        const parts = path.split('/');
        for (let i = 1; i < parts.length; i++) {
          directories.add(parts.slice(0, i).join('/') || '/');
        }
      }
    } catch (e) {
      console.warn(`Failed to preload file: ${path}`, e);
    }
  }

  // Add rootPath for Node.js WASI preopens
  const rootPath = shouldUseNodeWASI() ? process.cwd() : undefined;

  return { files, directories, rootPath };
}

/**
 * Create WASI filesystem bindings from a pre-loaded file cache
 * 
 * Note: For Node.js built-in WASI, this returns a minimal object with rootPath,
 * as Node.js WASI uses its own filesystem implementation with preopens.
 */
export function createWASIFilesystemBindings(cache: FileCache) {
  // For Node.js WASI, we just need to provide the rootPath for preopens
  // Node.js WASI will use its own filesystem implementation
  if (shouldUseNodeWASI()) {
    return {
      rootPath: cache.rootPath || process.cwd()
    };
  }
  
  // For @wasmer/wasi, we need full custom filesystem bindings
  const openFiles = new Map<number, {
    path: string;
    position: number;
    data: Uint8Array | null;
    writeBuffer?: Uint8Array;
  }>();
  
  let nextFd = 3; // 0, 1, 2 are stdin, stdout, stderr

  return {
    // File operations
    openSync(path: string, flags: number, mode?: number): number {
      console.log('[WASI] openSync:', path, flags);
      
      const fd = nextFd++;
      const data = cache.files.get(path) || null;
      
      openFiles.set(fd, {
        path,
        position: 0,
        data: data ? new Uint8Array(data) : null, // Copy to allow modifications
      });
      
      return fd;
    },

    readSync(
      fd: number,
      buffer: Uint8Array,
      offset: number,
      length: number,
      position: number | null
    ): number {
      const file = openFiles.get(fd);
      if (!file || !file.data) {
        return 0;
      }

      const readPos = position !== null ? position : file.position;
      const bytesToRead = Math.min(length, file.data.length - readPos);
      
      if (bytesToRead <= 0) {
        return 0;
      }

      // Copy data from file to buffer
      buffer.set(
        file.data.subarray(readPos, readPos + bytesToRead),
        offset
      );

      // Update position if not absolute read
      if (position === null) {
        file.position += bytesToRead;
      }

      return bytesToRead;
    },

    writeSync(
      fd: number,
      buffer: Uint8Array,
      offset: number,
      length: number,
      position: number | null
    ): number {
      // Handle stdout/stderr
      if (fd === 1 || fd === 2) {
        const text = new TextDecoder().decode(
          buffer.subarray(offset, offset + length)
        );
        console.log('[WASI stdout]:', text);
        return length;
      }

      const file = openFiles.get(fd);
      if (!file) {
        return 0;
      }

      const writePos = position !== null ? position : file.position;
      
      // Initialize or expand data buffer
      if (!file.data) {
        file.data = new Uint8Array(writePos + length);
      } else if (file.data.length < writePos + length) {
        const newData = new Uint8Array(writePos + length);
        newData.set(file.data);
        file.data = newData;
      }

      // Write data
      file.data.set(buffer.subarray(offset, offset + length), writePos);

      // Update position
      if (position === null) {
        file.position += length;
      }

      // Update cache
      cache.files.set(file.path, file.data);

      return length;
    },

    closeSync(fd: number): void {
      const file = openFiles.get(fd);
      if (file) {
        console.log('[WASI] closeSync:', file.path);
        openFiles.delete(fd);
      }
    },

    // Stat operations
    statSync(path: string) {
      const isFile = cache.files.has(path);
      const isDir = cache.directories.has(path);
      const size = cache.files.get(path)?.length || 0;

      return {
        isFile: () => isFile,
        isDirectory: () => isDir,
        isBlockDevice: () => false,
        isCharacterDevice: () => false,
        isSymbolicLink: () => false,
        isFIFO: () => false,
        isSocket: () => false,
        dev: 0,
        ino: 0,
        mode: isFile ? 33188 : 16877, // 0644 for files, 0755 for dirs
        nlink: 1,
        uid: 1000,
        gid: 1000,
        rdev: 0,
        size,
        blksize: 4096,
        blocks: Math.ceil(size / 512),
        atimeMs: Date.now(),
        mtimeMs: Date.now(),
        ctimeMs: Date.now(),
        birthtimeMs: Date.now(),
        atime: new Date(),
        mtime: new Date(),
        ctime: new Date(),
        birthtime: new Date(),
      };
    },

    fstatSync(fd: number) {
      const file = openFiles.get(fd);
      if (!file) {
        throw new Error(`Bad file descriptor: ${fd}`);
      }
      return this.statSync(file.path);
    },

    // Directory operations
    readdirSync(path: string): string[] {
      console.log('[WASI] readdirSync:', path);
      
      // Find all files/dirs that are direct children of this path
      const normalizedPath = path.endsWith('/') ? path : path + '/';
      const children = new Set<string>();

      for (const filePath of cache.files.keys()) {
        if (filePath.startsWith(normalizedPath)) {
          const remainder = filePath.substring(normalizedPath.length);
          const nextSlash = remainder.indexOf('/');
          const child = nextSlash === -1 ? remainder : remainder.substring(0, nextSlash);
          if (child) {
            children.add(child);
          }
        }
      }

      return Array.from(children);
    },

    mkdirSync(path: string, options?: { recursive?: boolean }): void {
      console.log('[WASI] mkdirSync:', path);
      cache.directories.add(path);
      
      if (options?.recursive) {
        const parts = path.split('/');
        for (let i = 1; i < parts.length; i++) {
          cache.directories.add(parts.slice(0, i).join('/') || '/');
        }
      }
    },

    existsSync(path: string): boolean {
      return cache.files.has(path) || cache.directories.has(path);
    },

    // Path operations
    realpathSync(path: string): string {
      return path;
    },

    // Access checks
    accessSync(path: string): void {
      if (!this.existsSync(path)) {
        throw new Error(`ENOENT: no such file or directory: ${path}`);
      }
    },
  };
}

/**
 * Collect files written by WASM back to async FileSystem
 */
export async function flushCache(
  cache: FileCache,
  filesystem: FileSystem
): Promise<void> {
  for (const [path, content] of cache.files) {
    try {
      await filesystem.write(path, content);
    } catch (e) {
      console.error(`Failed to flush file: ${path}`, e);
    }
  }
}

