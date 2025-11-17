import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, rmSync, unlinkSync, writeFileSync } from 'node:fs';
import { promises as fs } from 'node:fs';
import { join, resolve as resolvePath } from 'node:path';

export interface DiskCacheOptions {
  directory?: string;
}

export class DiskCache {
  private readonly directory: string;

  constructor(options: DiskCacheOptions = {}) {
    this.directory = resolvePath(options.directory ?? '.joy-cache');
  }

  getSync(key: string): Uint8Array | null {
    const location = this.pathFor(key);
    try {
      const contents = readFileSync(location);
      return new Uint8Array(contents);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return null;
      }
      throw error;
    }
  }

  async get(key: string): Promise<Uint8Array | null> {
    const location = this.pathFor(key);
    try {
      const contents = await fs.readFile(location);
      return new Uint8Array(contents);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return null;
      }
      throw error;
    }
  }

  setSync(key: string, value: Uint8Array): void {
    this.ensureDirSync();
    writeFileSync(this.pathFor(key), value);
  }

  async set(key: string, value: Uint8Array): Promise<void> {
    await this.ensureDir();
    await fs.writeFile(this.pathFor(key), value);
  }

  deleteSync(key: string): void {
    try {
      unlinkSync(this.pathFor(key));
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
        throw error;
      }
    }
  }

  async delete(key: string): Promise<void> {
    try {
      await fs.unlink(this.pathFor(key));
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
        throw error;
      }
    }
  }

  clearSync(): void {
    if (!existsSync(this.directory)) {
      return;
    }
    rmSync(this.directory, { recursive: true, force: true });
  }

  async clear(): Promise<void> {
    await fs.rm(this.directory, { recursive: true, force: true });
  }

  private async ensureDir(): Promise<void> {
    await fs.mkdir(this.directory, { recursive: true });
  }

  private ensureDirSync(): void {
    if (!existsSync(this.directory)) {
      mkdirSync(this.directory, { recursive: true });
    }
  }

  private pathFor(key: string): string {
    const digest = createHash('sha256').update(key).digest('hex');
    return join(this.directory, `${digest}.bin`);
  }
}
