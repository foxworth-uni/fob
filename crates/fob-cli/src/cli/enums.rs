use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Output format for bundled code
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Format {
    /// ECMAScript modules (import/export syntax)
    ///
    /// Modern format with static imports, enabling tree shaking and code splitting.
    /// Supported in modern browsers and Node.js 14+.
    #[value(name = "esm")]
    Esm,

    /// CommonJS modules (require/module.exports)
    ///
    /// Traditional Node.js format. Use this for maximum compatibility with
    /// older Node.js versions and tools that don't support ESM.
    #[value(name = "cjs")]
    Cjs,

    /// Immediately Invoked Function Expression
    ///
    /// Wraps code in a function that executes immediately. Suitable for
    /// browser script tags and environments without module support.
    #[value(name = "iife")]
    Iife,
}

/// Source map generation mode
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum SourceMapMode {
    /// Inline source maps embedded in the bundle
    ///
    /// The source map is encoded as a base64 data URL and appended to the bundle.
    /// Results in a single file but increases bundle size.
    #[value(name = "inline")]
    Inline,

    /// External source map files (.map)
    ///
    /// Generates separate .map files alongside bundles. Keeps bundles smaller
    /// and allows selective loading of source maps.
    #[value(name = "external")]
    External,

    /// Generate source maps but don't reference them
    ///
    /// Creates .map files but doesn't add sourceMappingURL comments to bundles.
    /// Useful for production builds where you want maps for debugging but
    /// don't want to expose source structure to end users.
    #[value(name = "hidden")]
    Hidden,
}

/// Target platform environment
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Platform {
    /// Browser environment
    ///
    /// Assumes APIs like window, document, fetch are available.
    /// Node.js-specific APIs (fs, path, etc.) are not available.
    #[value(name = "browser")]
    Browser,

    /// Node.js environment
    ///
    /// Assumes Node.js built-in modules are available (fs, path, http, etc.).
    /// Browser-specific APIs are not available.
    #[value(name = "node")]
    Node,
}

/// ECMAScript target version
///
/// Determines which JavaScript language features are available in the output.
/// Later versions enable more modern syntax and APIs but require newer runtimes.
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EsTarget {
    /// ECMAScript 2015 (ES6)
    ///
    /// Maximum compatibility. Supports IE11, Node.js 4+, and all modern browsers.
    /// Features: classes, arrow functions, promises, let/const.
    #[value(name = "es2015")]
    #[serde(rename = "es2015")]
    Es2015,

    /// ECMAScript 2016
    ///
    /// Adds: Array.prototype.includes, exponentiation operator (**)
    #[value(name = "es2016")]
    #[serde(rename = "es2016")]
    Es2016,

    /// ECMAScript 2017
    ///
    /// Adds: async/await, Object.entries/values, string padding
    #[value(name = "es2017")]
    #[serde(rename = "es2017")]
    Es2017,

    /// ECMAScript 2018
    ///
    /// Adds: async iteration, rest/spread properties, Promise.finally
    #[value(name = "es2018")]
    #[serde(rename = "es2018")]
    Es2018,

    /// ECMAScript 2019
    ///
    /// Adds: Array.prototype.flat/flatMap, Object.fromEntries, optional catch
    #[value(name = "es2019")]
    #[serde(rename = "es2019")]
    Es2019,

    /// ECMAScript 2020
    ///
    /// Adds: optional chaining (?.), nullish coalescing (??), BigInt, dynamic import
    /// Good balance between modern features and compatibility. Node.js 14+, modern browsers.
    #[value(name = "es2020")]
    #[serde(rename = "es2020")]
    Es2020,

    /// ECMAScript 2021
    ///
    /// Adds: String.prototype.replaceAll, Promise.any, logical assignment operators
    #[value(name = "es2021")]
    #[serde(rename = "es2021")]
    Es2021,

    /// ECMAScript 2022
    ///
    /// Adds: class fields, top-level await, Array.prototype.at, Object.hasOwn
    #[value(name = "es2022")]
    #[serde(rename = "es2022")]
    Es2022,

    /// Latest ECMAScript features
    ///
    /// Uses the newest JavaScript syntax and APIs. Requires the latest Node.js
    /// and browsers. Output may break in older environments.
    #[value(name = "esnext")]
    #[serde(rename = "esnext")]
    Esnext,
}

/// Output format for generated documentation.
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum DocsFormat {
    #[value(name = "md")]
    Markdown,
    #[value(name = "json")]
    Json,
    #[value(name = "both")]
    Both,
}

/// LLM enhancement mode for documentation
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum DocsEnhanceMode {
    /// Only enhance symbols with no documentation
    ///
    /// Fastest mode - only processes completely undocumented symbols.
    /// Recommended for quick documentation passes.
    #[value(name = "missing")]
    Missing,

    /// Enhance incomplete documentation
    ///
    /// Processes symbols that are missing parameters, return types,
    /// or examples. Good balance between speed and thoroughness.
    #[value(name = "incomplete")]
    Incomplete,

    /// Enhance all symbols
    ///
    /// Most thorough - enhances all symbols, even those with complete
    /// JSDoc, merging LLM output with existing documentation.
    #[value(name = "all")]
    All,
}

/// Merge strategy for writing documentation back to source files.
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum DocsMergeStrategy {
    /// Merge LLM output with existing JSDoc
    ///
    /// Preserves custom tags like @deprecated, @internal, etc.
    /// Adds or updates description, params, returns, and examples.
    #[value(name = "merge")]
    Merge,

    /// Replace existing JSDoc entirely
    ///
    /// Completely replaces existing documentation with LLM output.
    /// Use with caution as it will remove custom tags.
    #[value(name = "replace")]
    Replace,

    /// Skip symbols with existing JSDoc
    ///
    /// Only adds documentation to symbols that have no JSDoc at all.
    /// Useful for preserving all hand-written documentation.
    #[value(name = "skip")]
    Skip,
}

