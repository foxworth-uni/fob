/**
 * Fob Bundler Wrapper for Next.js
 *
 * Server-side module for on-demand code compilation.
 */

import pkg from '@fox-uni/fob';
const { Fob } = pkg;

import { writeFile, rm, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { tmpdir } from 'node:os';

// Use system temp directory for Next.js (avoids .next directory issues)
const CACHE_DIR = join(tmpdir(), 'fob-nextjs-playground');

// Ensure cache directory exists on module load
mkdir(CACHE_DIR, { recursive: true }).catch(() => {});

export interface BundleChunk {
  fileName: string;
  code: string;
  size: number;
  kind: string;
}

export interface BundleStats {
  totalModules: number;
  totalChunks: number;
  totalSize: number;
  durationMs: number;
  cacheHitRate: number;
}

export interface BundleResult {
  chunks: BundleChunk[];
  stats: BundleStats;
}

/**
 * Bundle code on-demand
 */
export async function bundle(code: string, filename = 'main.tsx'): Promise<BundleResult> {
  const id = randomUUID().slice(0, 8);
  const inputDir = join(CACHE_DIR, `input-${id}`);
  const outputDir = join(CACHE_DIR, `output-${id}`);
  const inputFile = join(inputDir, filename);

  try {
    await mkdir(inputDir, { recursive: true });
    await mkdir(outputDir, { recursive: true });
    await writeFile(inputFile, code, 'utf-8');

    const bundler = new Fob({
      entries: [inputFile],
      outputDir,
      format: 'esm',
      platform: 'browser',
      sourcemap: 'inline',
      cwd: inputDir,
    });

    const startTime = performance.now();
    const result = await bundler.bundle();
    const duration = Math.round(performance.now() - startTime);

    const outputChunks: BundleChunk[] = (result.chunks || []).map((chunk: unknown) => {
      const c = chunk as { fileName?: string; code?: string; size?: number; kind?: string };
      return {
        fileName: c.fileName ?? '',
        code: c.code ?? '',
        size: c.size ?? 0,
        kind: c.kind ?? 'entry',
      };
    });

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
    await rm(inputDir, { recursive: true, force: true }).catch(() => {});
    await rm(outputDir, { recursive: true, force: true }).catch(() => {});
  }
}

// Build history for stats tracking
interface BuildInfo {
  timestamp: number;
  duration: number;
  modules: number;
  chunks: number;
  size: number;
  cacheHitRate: number;
  success: boolean;
  error?: string;
}

const buildHistory: BuildInfo[] = [];
const MAX_HISTORY = 50;

export function recordBuild(info: BuildInfo) {
  buildHistory.push(info);
  if (buildHistory.length > MAX_HISTORY) {
    buildHistory.shift();
  }
}

export function getBuildStats() {
  const successful = buildHistory.filter((b) => b.success);
  const avgDuration =
    successful.length > 0
      ? successful.reduce((sum, b) => sum + b.duration, 0) / successful.length
      : 0;
  const avgCacheHitRate =
    successful.length > 0
      ? successful.reduce((sum, b) => sum + (b.cacheHitRate || 0), 0) / successful.length
      : 0;

  return {
    totalBuilds: buildHistory.length,
    successfulBuilds: successful.length,
    failedBuilds: buildHistory.length - successful.length,
    avgDuration: Math.round(avgDuration),
    avgCacheHitRate: Math.round(avgCacheHitRate * 100),
    recentBuilds: buildHistory.slice(-10),
  };
}
