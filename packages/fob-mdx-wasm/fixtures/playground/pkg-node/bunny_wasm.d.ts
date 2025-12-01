/* tslint:disable */
/* eslint-disable */
/**
 * Initialize panic hook for better error messages in console
 */
export function init(): void;
/**
 * Compile MDX source to JSX
 *
 * # Arguments
 *
 * * `source` - MDX source code as string (max 10MB)
 * * `options` - Compilation options (optional, uses defaults if None)
 *
 * # Returns
 *
 * * `Ok(WasmMdxResult)` - Compiled JSX and metadata
 * * `Err(JsValue)` - Structured error object with kind, message, location, etc.
 *
 * # Errors
 *
 * Returns structured error objects that can be discriminated by `kind`:
 * - `"validationError"` - Input validation failed (size limit, null bytes)
 * - `"compilationError"` - MDX syntax error (with location and suggestion)
 * - `"serializationError"` - Failed to serialize result to JavaScript
 *
 * # Example
 *
 * ```javascript
 * import { compile_mdx, WasmMdxOptions } from './pkg/bunny_wasm.js';
 *
 * const options = new WasmMdxOptions();
 * options.set_gfm(true);
 *
 * try {
 *   const result = compile_mdx("# Hello **World**", options);
 *   console.log(result.code);
 * } catch (error) {
 *   if (error.kind === "compilationError") {
 *     console.error(error.message);
 *     if (error.location) {
 *       console.error(`At ${error.location.line}:${error.location.column}`);
 *     }
 *     if (error.suggestion) {
 *       console.log(`Suggestion: ${error.suggestion}`);
 *     }
 *   }
 * }
 * ```
 */
export function compile_mdx(source: string, options?: WasmMdxOptions | null): any;
/**
 * WASM-compatible MDX compilation options
 *
 * This is a JS-friendly wrapper around `MdxCompileOptions` that can be
 * constructed and configured from JavaScript.
 */
export class WasmMdxOptions {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Set the filepath (for error messages)
   */
  set_filepath(filepath: string): void;
  /**
   * Enable/disable footnotes
   */
  set_footnotes(enabled: boolean): void;
  /**
   * Set JSX runtime (default: "react/jsx-runtime")
   */
  set_jsx_runtime(runtime: string): void;
  /**
   * Set output format ("program" or "function-body")
   */
  set_output_format(format: string): void;
  /**
   * Create new options with defaults
   */
  constructor();
  /**
   * Enable/disable GFM (GitHub Flavored Markdown)
   */
  set_gfm(enabled: boolean): void;
  /**
   * Enable/disable math
   */
  set_math(enabled: boolean): void;
  /**
   * Get JSX runtime
   */
  readonly jsx_runtime: string;
  /**
   * Get output format
   */
  readonly output_format: string;
  /**
   * Get GFM setting
   */
  readonly gfm: boolean;
  /**
   * Get math setting
   */
  readonly math: boolean;
  /**
   * Get the filepath
   */
  readonly filepath: string | undefined;
  /**
   * Get footnotes setting
   */
  readonly footnotes: boolean;
}
