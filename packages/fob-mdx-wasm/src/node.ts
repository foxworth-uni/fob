/**
 * # @fob/mdx-wasm (Node.js)
 *
 * Node.js bindings for fob-mdx - compile MDX to JSX on the server.
 *
 * This module provides the Node.js-specific entry point for fob-mdx-wasm,
 * using the nodejs target build from wasm-pack.
 *
 * ## Usage
 *
 * ```typescript
 * import { compile_mdx, WasmMdxOptions } from '@fob/mdx-wasm';
 *
 * // Create options
 * const options = new WasmMdxOptions();
 * options.set_gfm(true);
 * options.set_math(true);
 *
 * // Compile MDX
 * const result = compile_mdx("# Hello **World**", options);
 * console.log(result.code); // Compiled JSX
 * ```
 *
 * Note: Unlike the browser version, Node.js doesn't require calling init().
 */

// Re-export everything from the Node.js WASM bindings
export * from '../pkg-node/fob_mdx_wasm.js';
