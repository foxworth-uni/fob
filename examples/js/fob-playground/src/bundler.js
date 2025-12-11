/**
 * Fob Bundler Wrapper
 *
 * Wraps the @fox-uni/fob package for on-demand compilation.
 * Uses temp files since NAPI doesn't expose virtual file API yet.
 */

import pkg from '@fox-uni/fob';
const { Fob, OutputFormat } = pkg;

import { writeFile, rm, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { randomUUID } from 'node:crypto';

const __dirname = dirname(fileURLToPath(import.meta.url));
const CACHE_DIR = join(__dirname, '..', '.fob-cache');

// Ensure cache directory exists
await mkdir(CACHE_DIR, { recursive: true });

/**
 * Bundle code by writing to temp file, bundling, then reading output.
 *
 * @param {string} code - The source code to bundle
 * @param {string} filename - Filename hint for extension (e.g., 'main.tsx')
 * @returns {Promise<BundleResult>} Bundle result with chunks and stats
 */
export async function bundle(code, filename = 'main.tsx') {
  const id = randomUUID().slice(0, 8);
  const inputDir = join(CACHE_DIR, `input-${id}`);
  const outputDir = join(CACHE_DIR, `output-${id}`);
  const inputFile = join(inputDir, filename);

  try {
    // Create temp directories
    await mkdir(inputDir, { recursive: true });
    await mkdir(outputDir, { recursive: true });

    // Write code to temp file
    await writeFile(inputFile, code, 'utf-8');

    // Bundle
    const bundler = new Fob({
      entries: [inputFile],
      output_dir: outputDir,
      format: OutputFormat.Esm,
      platform: 'browser',
      sourcemap: 'inline',
      cwd: inputDir,
    });

    const startTime = performance.now();
    const result = await bundler.bundle();
    const duration = Math.round(performance.now() - startTime);

    // Code is directly on the chunk object
    const outputChunks = (result.chunks || []).map((chunk) => ({
      fileName: chunk.fileName,
      code: chunk.code || '',
      size: chunk.size || 0,
      kind: chunk.kind || 'entry',
    }));

    return {
      chunks: outputChunks,
      stats: {
        totalModules: result.moduleCount || 1,
        totalChunks: outputChunks.length,
        totalSize: result.stats?.totalSize || outputChunks.reduce((sum, c) => sum + c.size, 0),
        durationMs: duration,
        cacheHitRate: result.stats?.cacheHitRate || 0,
      },
    };
  } finally {
    // Clean up temp files
    await rm(inputDir, { recursive: true, force: true }).catch(() => {});
    await rm(outputDir, { recursive: true, force: true }).catch(() => {});
  }
}

/**
 * Bundle multiple files
 *
 * @param {Record<string, string>} files - Map of filename to code
 * @param {string} entry - Entry point filename
 * @returns {Promise<BundleResult>}
 */
export async function bundleMultiple(files, entry) {
  const id = randomUUID().slice(0, 8);
  const inputDir = join(CACHE_DIR, `input-${id}`);
  const outputDir = join(CACHE_DIR, `output-${id}`);

  try {
    // Create temp directories
    await mkdir(inputDir, { recursive: true });
    await mkdir(outputDir, { recursive: true });

    // Write all files
    for (const [name, code] of Object.entries(files)) {
      const filePath = join(inputDir, name);
      // Ensure subdirectories exist
      await mkdir(dirname(filePath), { recursive: true });
      await writeFile(filePath, code, 'utf-8');
    }

    // Bundle
    const bundler = new Fob({
      entries: [join(inputDir, entry)],
      output_dir: outputDir,
      format: OutputFormat.Esm,
      platform: 'browser',
      sourcemap: 'inline',
      cwd: inputDir,
    });

    const startTime = performance.now();
    const result = await bundler.bundle();
    const duration = Math.round(performance.now() - startTime);

    // Code is directly on the chunk object
    const outputChunks = (result.chunks || []).map((chunk) => ({
      fileName: chunk.fileName,
      code: chunk.code || '',
      size: chunk.size || 0,
      kind: chunk.kind || 'entry',
    }));

    return {
      chunks: outputChunks,
      stats: {
        totalModules: result.moduleCount || Object.keys(files).length,
        totalChunks: outputChunks.length,
        totalSize: result.stats?.totalSize || outputChunks.reduce((sum, c) => sum + c.size, 0),
        durationMs: duration,
        cacheHitRate: result.stats?.cacheHitRate || 0,
      },
    };
  } finally {
    // Clean up temp files
    await rm(inputDir, { recursive: true, force: true }).catch(() => {});
    await rm(outputDir, { recursive: true, force: true }).catch(() => {});
  }
}
