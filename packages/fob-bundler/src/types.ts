/**
 * Fob Bundler TypeScript definitions
 */

export interface BundleOptions {
  /** Entry points (must be ESM modules) */
  entries: string[];

  /** Directory containing static assets to copy into the output directory */
  staticDir?: string | null;

  /** Output directory for generated chunks */
  outputDir?: string;

  /** Output format */
  format?: 'esm' | 'preserve-modules';

  /** Platform target */
  platform?: 'browser' | 'node' | 'worker' | 'deno';

  /** Enable code splitting (dynamic imports become separate chunks) */
  codeSplitting?: boolean;

  /** Enable minification */
  minify?: boolean;

  /** Source map generation */
  sourceMaps?: 'none' | 'inline' | 'external' | 'external-with-content';

  /** Shared chunk threshold (bytes) */
  sharedChunkThreshold?: number;

  /** External modules (not bundled, treated as runtime imports) */
  external?: string[];

  /** Resolve conditions (e.g., "browser", "development", "production") */
  conditions?: string[];

  /** Experimental features */
  experimental?: {
    css?: boolean;
    json?: boolean;
    analysis?: boolean;
  };

  /** Configured plugins */
  plugins?: PluginOptions[];

  /** Cache configuration */
  cacheConfig?: CacheConfig;

  /** Transform/transpilation options */
  transform?: TransformOptions;

  /** TypeScript configuration */
  typescriptConfig?: TypeScriptConfig | null;

  /** HTML generation configuration */
  html?: HtmlOptions | null;

  /** Virtual module configuration */
  virtualModules?: VirtualModuleConfig | null;

  /** Inline transform functions */
  inlineTransforms?: InlineTransform[] | null;

  /** CSS processing configuration */
  css?: CssOptions;

  /** Path aliases for import resolution */
  pathAliases?: Record<string, string>;
}

export interface BundleResult {
  /** Generated chunks */
  chunks: Chunk[];

  /** Bundle manifest */
  manifest: Manifest;

  /** Build statistics */
  stats: BuildStats;

  /** Optimized asset files emitted by plugins */
  assets: BundleAsset[];
}

export interface BundleAsset {
  /** Path exposed to the runtime (e.g., "/assets/images/foo.png") */
  publicPath: string;

  /** Path relative to the output directory (e.g., "assets/images/foo.png") */
  relativePath: string;

  /** Size in bytes of the emitted asset */
  size: number;

  /** Optional format hint (png, jpeg, webp, etc.) */
  format?: string;
}

export interface Chunk {
  /** Chunk identifier (stable, content-hashed) */
  id: string;

  /** Kind of chunk */
  kind: 'entry' | 'async' | 'shared';

  /** File name (e.g., "main-abc123.js") */
  fileName: string;

  /** Generated code */
  code: string;

  /** Source map (if enabled) */
  sourceMap?: string;

  /** Modules included in this chunk */
  modules: ModuleInfo[];

  /** Chunks this one imports (for preload hints) */
  imports: string[];

  /** Chunks this one dynamically imports */
  dynamicImports: string[];

  /** Size in bytes */
  size: number;
}

export interface ModuleInfo {
  /** Module path (relative to project root) */
  path: string;

  /** Size in bytes (pre-minification) */
  size: number;

  /** Whether this module has side effects */
  hasSideEffects: boolean;
}

export interface Manifest {
  /** Entry chunk mapping */
  entries: Record<string, string>;

  /** All chunks */
  chunks: Record<string, ChunkMetadata>;

  /** Version */
  version: string;
}

export interface ChunkMetadata {
  file: string;
  imports?: string[];
  dynamicImports?: string[];
  css?: string[];
}

export interface BuildStats {
  /** Total modules processed */
  totalModules: number;

  /** Total chunks generated */
  totalChunks: number;

  /** Total output size (bytes) */
  totalSize: number;

  /** Build duration (milliseconds) */
  durationMs: number;

  /** Cache hit rate (0.0 - 1.0) */
  cacheHitRate: number;
}

export interface ProgressEvent {
  /** Progress type */
  type: 'parse' | 'resolve' | 'transform' | 'bundle' | 'emit';

  /** Current step */
  current: number;

  /** Total steps */
  total: number;

  /** Current module being processed (if applicable) */
  module?: string;

  /** Message */
  message?: string;
}

export interface FileMetadata {
  size: number;
  modifiedMs: number;
}

/**
 * Filesystem abstraction for Fob bundler
 */
export interface FileSystem {
  /** Read a file */
  read(path: string): Promise<Uint8Array>;

  /** Write a file */
  write(path: string, content: Uint8Array): Promise<void>;

  /** Get file metadata */
  metadata(path: string): Promise<FileMetadata>;

  /** Check if file exists */
  exists(path: string): boolean;
}

/** Plugin configuration */
export interface PluginOptions {
  name?: string;
  backend?: 'extism' | null;
  path: string;
  config?: unknown;
  order?: number;
  enabled?: boolean;
  poolSize?: number;
  maxMemoryBytes?: number;
  timeoutMs?: number;
  profiles?: Record<string, unknown>;
}

/** Cache configuration */
export interface CacheConfig {
  enabled?: boolean;
  cacheDir?: string | null;
  maxSize?: number;
}

/** Transform/transpilation options */
export interface TransformOptions {
  typescript?: boolean;
  jsx?: boolean;
  target?: string;
  typeCheck?: 'none';
  jsxRuntime?: 'classic' | 'automatic';
  jsxImportSource?: string | null;
  jsxDev?: boolean;
  define?: Record<string, string>;
  enableSsrTransform?: boolean;
  mode?: string;
  publicEnv?: Record<string, string>;
  minify?: boolean;
}

/** TypeScript configuration */
export interface TypeScriptConfig {
  configPath?: string | null;
  jsxImportSource?: string | null;
  allowJs?: boolean;
}

/** HTML generation configuration */
export interface HtmlOptions {
  template?: string | null;
  templateType?: 'spa' | 'mpa';
  filename?: string;
  title?: string | null;
  description?: string | null;
  keywords?: string | null;
  lang?: string;
  favicon?: string | null;
  body?: string | null;
  head?: string | null;
  variables?: Record<string, unknown>;
}

/** Virtual module configuration */
export interface VirtualModuleConfig {
  modules?: Record<string, string>;
}

/** Inline transform configuration */
export interface InlineTransform {
  test: string;
  transform: string;
}

/** CSS processing configuration */
export interface CssOptions {
  enabled?: boolean;
  modules?: boolean;
  tailwind?: TailwindOptions | null;
  postcssConfig?: string | null;
}

/** Tailwind CSS configuration */
export interface TailwindOptions {
  enabled?: boolean;
  config?: string | null;
  input?: string | null;
  output?: string;
  content?: string[] | null;
  watch?: boolean;
}

/**
 * Structured error details from Fob bundler.
 * This discriminated union preserves all diagnostic information from Rust.
 */
export type FobErrorDetails =
  | MdxSyntaxError
  | MissingExportError
  | TransformError
  | CircularDependencyError
  | InvalidEntryError
  | NoEntriesError
  | PluginError
  | RuntimeError
  | ValidationError;

/**
 * MDX syntax error with source location and suggestions.
 */
export interface MdxSyntaxError {
  type: 'mdx_syntax';
  message: string;
  file?: string;
  line?: number;
  column?: number;
  context?: string;
  suggestion?: string;
}

/**
 * Missing export error with available alternatives and suggestions.
 */
export interface MissingExportError {
  type: 'missing_export';
  export_name: string;
  module_id: string;
  available_exports: string[];
  suggestion?: string;
}

/**
 * TypeScript/JSX transformation error with diagnostics.
 */
export interface TransformError {
  type: 'transform';
  path: string;
  diagnostics: TransformDiagnostic[];
}

/**
 * Diagnostic information for transformation errors.
 */
export interface TransformDiagnostic {
  message: string;
  line: number;
  column: number;
  severity: 'error' | 'warning';
  help?: string;
}

/**
 * Circular dependency error with full cycle path.
 */
export interface CircularDependencyError {
  type: 'circular_dependency';
  cycle_path: string[];
}

/**
 * Invalid entry point error.
 */
export interface InvalidEntryError {
  type: 'invalid_entry';
  path: string;
}

/**
 * No entry points specified error.
 */
export interface NoEntriesError {
  type: 'no_entries';
}

/**
 * Plugin execution error.
 */
export interface PluginError {
  type: 'plugin';
  name: string;
  message: string;
}

/**
 * Generic runtime error.
 */
export interface RuntimeError {
  type: 'runtime';
  message: string;
}

/**
 * Validation error (catch-all for other validation issues).
 */
export interface ValidationError {
  type: 'validation';
  message: string;
}

/**
 * Error thrown by Fob bundler with structured error details.
 *
 * The `details` property contains structured error information that can be
 * used to provide better error messages and diagnostics to users.
 *
 * @example
 * ```typescript
 * try {
 *   await bundle(options);
 * } catch (error) {
 *   if (error instanceof FobError && error.details) {
 *     if (error.details.type === 'missing_export') {
 *       console.error(`Missing export '${error.details.export_name}' in ${error.details.module_id}`);
 *       console.error(`Available: ${error.details.available_exports.join(', ')}`);
 *       if (error.details.suggestion) {
 *         console.error(`Did you mean '${error.details.suggestion}'?`);
 *       }
 *     }
 *   }
 * }
 * ```
 */
export class FobError extends Error {
  constructor(
    message: string,
    public readonly details?: FobErrorDetails
  ) {
    super(message);
    this.name = 'FobError';
  }
}

/**
 * Format a bundle error into a user-friendly message.
 *
 * @param error - The error details to format
 * @returns A formatted error message string
 */
export function formatFobError(error: FobErrorDetails): string {
  switch (error.type) {
    case 'mdx_syntax': {
      let msg = `MDX Error: ${error.message}`;
      if (error.file) msg += `\n  in ${error.file}`;
      if (error.line && error.column) {
        msg += `\n  at line ${error.line}, column ${error.column}`;
      }
      if (error.context) msg += `\n\n${error.context}`;
      if (error.suggestion) msg += `\n\n💡 Suggestion: ${error.suggestion}`;
      return msg;
    }

    case 'missing_export': {
      let msg = `Named export '${error.export_name}' not found in module '${error.module_id}'`;
      if (error.available_exports.length > 0) {
        msg += `\n\nAvailable exports: ${error.available_exports.join(', ')}`;
      } else {
        msg += `\n\nModule has no exports`;
      }
      if (error.suggestion) {
        msg += `\n\nDid you mean '${error.suggestion}'?`;
      }
      return msg;
    }

    case 'transform': {
      let msg = `Transform failed in ${error.path}`;
      if (error.diagnostics.length > 0) {
        msg += '\n\nDiagnostics:';
        for (const diag of error.diagnostics) {
          msg += `\n  [${diag.severity}] ${diag.message} (line ${diag.line}, col ${diag.column})`;
          if (diag.help) msg += `\n    Help: ${diag.help}`;
        }
      }
      return msg;
    }

    case 'circular_dependency':
      return `Circular dependency detected:\n  ${error.cycle_path.join(' → ')}`;

    case 'invalid_entry':
      return `Invalid entry point: ${error.path}`;

    case 'no_entries':
      return 'No entry points specified';

    case 'plugin':
      return `Plugin '${error.name}' failed: ${error.message}`;

    case 'runtime':
      return `Runtime error: ${error.message}`;

    case 'validation':
      return `Validation error: ${error.message}`;

    default:
      // Ensure exhaustive check
      const _exhaustive: never = error;
      return `Unknown error: ${JSON.stringify(_exhaustive)}`;
  }
}
