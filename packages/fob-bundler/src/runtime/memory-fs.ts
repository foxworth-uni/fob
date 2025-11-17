import type { FileSystem, FileMetadata } from '../types.js';

export class MemoryFileSystem implements FileSystem {
  private files: Map<string, Uint8Array>;
  private metadataMap: Map<string, FileMetadata>;

  constructor(files: Map<string, Uint8Array> = new Map()) {
    this.files = files;
    this.metadataMap = new Map();

    for (const [path, content] of files) {
      this.metadataMap.set(path, {
        size: content.byteLength,
        modifiedMs: Date.now(),
      });
    }
  }

  async read(path: string): Promise<Uint8Array> {
    const content = this.files.get(path);
    if (!content) {
      throw new Error(`File not found: ${path}`);
    }
    return content;
  }

  async write(path: string, content: Uint8Array): Promise<void> {
    this.files.set(path, content);
    this.metadataMap.set(path, {
      size: content.byteLength,
      modifiedMs: Date.now(),
    });
  }

  async metadata(path: string): Promise<FileMetadata> {
    const meta = this.metadataMap.get(path);
    if (!meta) {
      throw new Error(`File not found: ${path}`);
    }
    return meta;
  }

  exists(path: string): boolean {
    return this.files.has(path);
  }

  static fromObject(files: Record<string, string | Uint8Array>): MemoryFileSystem {
    const fileMap = new Map<string, Uint8Array>();

    for (const [path, content] of Object.entries(files)) {
      if (typeof content === 'string') {
        fileMap.set(path, new TextEncoder().encode(content));
      } else {
        fileMap.set(path, content);
      }
    }

    return new MemoryFileSystem(fileMap);
  }
}
