/**
 * Core bundler integration for fob-next
 * 
 * Bundles MDX files using @fox-uni/fob and provides caching layer
 */

import pkg from "@fox-uni/fob";
import type { ChunkInfo, BundleConfig } from "@fox-uni/fob";
import { mkdir, writeFile, stat, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import type React from "react";

const { Fob } = pkg;

export interface BundleMdxOptions {
  /** Absolute path to MDX file */
  filePath: string;
  /** Output directory (defaults to system temp) */
  outputDir?: string;
  /** Enable caching (default: true) */
  cache?: boolean;
  /** MDX compilation options */
  mdx?: {
    gfm?: boolean;
    footnotes?: boolean;
    math?: boolean;
    jsxRuntime?: string;
    useDefaultPlugins?: boolean;
  };
  /** External packages to exclude from bundle */
  external?: string[];
  /** Working directory for resolution */
  cwd?: string;
}

export interface BundledMdxModule {
  /** Default export component */
  default: React.ComponentType<{
    components?: Record<string, React.ComponentType<any>>;
  }>;
  /** Named exports (e.g., frontmatter) */
  [key: string]: unknown;
}

/**
 * Get cache key for MDX file based on path and mtime
 */
async function getCacheKey(filePath: string): Promise<string> {
  try {
    const stats = await stat(filePath);
    const content = readFileSync(filePath, "utf-8");
    const hash = createHash("sha256")
      .update(filePath)
      .update(stats.mtimeMs.toString())
      .update(content)
      .digest("hex");
    return hash.slice(0, 16);
  } catch {
    // Fallback to path-based key if stat fails
    return createHash("sha256").update(filePath).digest("hex").slice(0, 16);
  }
}

/**
 * Bundle an MDX file and return the compiled module
 */
export async function bundleMdx(
  options: BundleMdxOptions
): Promise<BundledMdxModule> {
  const { filePath, outputDir, cache = true, mdx, external, cwd } = options;

  // Resolve absolute paths
  const absFilePath = path.resolve(filePath);
  const absOutputDir = outputDir
    ? path.resolve(outputDir)
    : path.join(tmpdir(), "fob-next-mdx");

  // Create cache-aware output directory
  let finalOutputDir = absOutputDir;
  if (cache) {
    const cacheKey = await getCacheKey(absFilePath);
    finalOutputDir = path.join(absOutputDir, cacheKey);
  }

  await mkdir(finalOutputDir, { recursive: true });

  // Check if already cached (simple check - file exists)
  const manifestPath = path.join(finalOutputDir, ".manifest.json");

  if (cache) {
    try {
      const manifestContent = await readFile(manifestPath, "utf-8");
      const manifest = JSON.parse(manifestContent) as { entryChunk?: string };
      const entryChunk = manifest.entryChunk;
      if (entryChunk) {
        const modulePath = path.join(finalOutputDir, entryChunk);
        try {
          const mod = await import(pathToFileURL(modulePath).href);
          return mod as BundledMdxModule;
        } catch {
          // Cache miss or invalid, continue to rebuild
        }
      }
    } catch {
      // No cache, continue to build
    }
  }

  // Bundle with fob
  // Type assertion needed: runtime accepts strings but TypeScript types expect enums
  // The runtime conversion layer handles string-to-enum conversion
  const bundlerConfig = {
    entries: [absFilePath],
    outputDir: finalOutputDir,
    format: "esm",
    platform: "node",
    sourcemap: "false",
    entryMode: "isolated",
    external: external || ["react", "react-dom", "@fob/mdx-runtime"],
    cwd: cwd || path.dirname(absFilePath),
    ...(mdx && {
      mdx: {
        ...(mdx.gfm !== undefined && { gfm: mdx.gfm }),
        ...(mdx.footnotes !== undefined && { footnotes: mdx.footnotes }),
        ...(mdx.math !== undefined && { math: mdx.math }),
        ...(mdx.jsxRuntime !== undefined && { jsxRuntime: mdx.jsxRuntime }),
        ...(mdx.useDefaultPlugins !== undefined && {
          useDefaultPlugins: mdx.useDefaultPlugins,
        }),
      },
    }),
  } as unknown as BundleConfig;


  const bundler = new Fob(bundlerConfig);

  const result = await bundler.bundle();

  // Write all chunks
  await Promise.all(
    result.chunks.map((chunk: ChunkInfo) =>
      writeFile(path.join(finalOutputDir, chunk.fileName), chunk.code, "utf8")
    )
  );

  // Save manifest for cache lookup
  const entryChunk = result.chunks.find(
    (c: ChunkInfo) => c.kind === "entry"
  )?.fileName;
  if (entryChunk && cache) {
    await writeFile(
      manifestPath,
      JSON.stringify({ entryChunk }, null, 2),
      "utf8"
    );
  }

  // Import the entry module
  if (!entryChunk) {
    throw new Error("No entry chunk found in bundle result");
  }

  const modulePath = path.join(finalOutputDir, entryChunk);
  const module = await import(pathToFileURL(modulePath).href);

  return module as BundledMdxModule;
}

