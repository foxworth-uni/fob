import { existsSync, promises as fs } from 'node:fs';
import { dirname, isAbsolute, join, resolve as resolvePath } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { FileMetadata, FileSystem } from '../types.js';

export interface NativeFileSystemOptions {
  root?: string;
}

export class NativeFileSystem implements FileSystem {
  private readonly root: string;

  constructor(options: NativeFileSystemOptions = {}) {
    this.root = options.root ? resolvePath(options.root) : process.cwd();
  }

  async read(path: string): Promise<Uint8Array> {
    const target = this.resolve(path);
    return fs.readFile(target);
  }

  async write(path: string, content: Uint8Array): Promise<void> {
    const target = this.resolve(path);
    await fs.mkdir(dirname(target), { recursive: true });
    await fs.writeFile(target, content);
  }

  async metadata(path: string): Promise<FileMetadata> {
    const target = this.resolve(path);
    const stats = await fs.stat(target);
    return {
      size: Number(stats.size),
      modifiedMs: stats.mtimeMs,
    };
  }

  exists(path: string): boolean {
    const target = this.resolve(path);
    return existsSync(target);
  }

  private resolve(path: string): string {
    if (path.startsWith('file://')) {
      return fileURLToPath(path);
    }
    return isAbsolute(path) ? path : join(this.root, path);
  }
}
