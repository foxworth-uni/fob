use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::cli::enums::*;
use crate::cli::validation::parse_global;

/// Available Fob subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Build a library or application
    ///
    /// Bundles entry points into optimized output with support for multiple
    /// formats, TypeScript declarations, source maps, and tree shaking.
    Build(BuildArgs),

    /// Start development server with watch mode
    ///
    /// Runs a development server with hot module replacement and automatic
    /// rebuilding when source files change.
    Dev(DevArgs),

    /// Initialize a new Fob project
    ///
    /// Creates a new project with sensible defaults and optional templates
    /// for common project types (library, application, monorepo).
    Init(InitArgs),

    /// Validate configuration and dependencies
    ///
    /// Checks fob.config.json for errors and validates that all dependencies
    /// are correctly installed and compatible.
    Check(CheckArgs),
}

/// Arguments for the build command
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Entry points to bundle
    ///
    /// Specify one or more entry points. Each entry point will be bundled
    /// into a separate output file.
    ///
    /// Examples:
    ///   fob build src/index.ts
    ///   fob build src/main.ts src/worker.ts
    #[arg(required = true, value_name = "ENTRY")]
    pub entry: Vec<String>,

    /// Output format for the bundle
    ///
    /// - esm: ECMAScript modules (modern, tree-shakeable)
    /// - cjs: CommonJS (Node.js compatible)
    /// - iife: Immediately Invoked Function Expression (browser script tag)
    #[arg(short = 'f', long, value_enum, default_value = "esm")]
    pub format: Format,

    /// Output directory for bundled files
    ///
    /// All generated files (bundles, source maps, declarations) will be
    /// written to this directory. Created if it doesn't exist.
    #[arg(short = 'd', long, default_value = "dist", value_name = "DIR")]
    pub out_dir: PathBuf,

    /// Generate TypeScript declaration files (.d.ts)
    ///
    /// Extracts type definitions from TypeScript source files. Useful for
    /// library authors who want to provide type safety to consumers.
    #[arg(long)]
    pub dts: bool,

    /// Bundle declaration files into a single .d.ts file
    ///
    /// Combines all declaration files into one output file, making it easier
    /// for consumers to import types. Requires --dts to be enabled.
    #[arg(long, requires = "dts")]
    pub dts_bundle: bool,

    /// Generate API documentation from JSDoc / TSDoc comments
    ///
    /// Emits Markdown or JSON files describing exported symbols.
    #[arg(long)]
    pub docs: bool,

    /// Format for generated documentation (md, json, both)
    #[arg(long, value_enum, value_name = "FORMAT", requires = "docs")]
    pub docs_format: Option<DocsFormat>,

    /// Output directory for generated documentation files
    #[arg(long, value_name = "DIR", requires = "docs")]
    pub docs_dir: Option<PathBuf>,

    /// Include symbols annotated with @internal
    #[arg(long, requires = "docs")]
    pub docs_include_internal: bool,

    /// Enable LLM-powered documentation enhancement
    ///
    /// Uses Ollama to automatically generate comprehensive documentation
    /// for symbols with missing or incomplete JSDoc comments.
    /// Requires Ollama to be installed and running (ollama serve).
    #[arg(long, requires = "docs")]
    pub docs_enhance: bool,

    /// LLM enhancement mode (missing, incomplete, all)
    ///
    /// - missing: Only enhance symbols with no JSDoc at all (fastest)
    /// - incomplete: Enhance symbols missing params, returns, or examples
    /// - all: Enhance all symbols, merging with existing docs (most thorough)
    #[arg(long, value_enum, value_name = "MODE", requires = "docs_enhance")]
    pub docs_enhance_mode: Option<DocsEnhanceMode>,

    /// LLM model to use for enhancement
    ///
    /// Specifies which Ollama model to use for generating documentation.
    /// Examples: llama3.2:3b (default, fast), codellama:7b (better quality),
    /// deepseek-coder:6.7b (code-focused), qwen2.5-coder:7b (latest)
    #[arg(long, value_name = "MODEL", requires = "docs_enhance")]
    pub docs_llm_model: Option<String>,

    /// Disable LLM response caching
    ///
    /// By default, LLM responses are cached using BLAKE3-based smart invalidation
    /// that auto-invalidates when code changes. Use this flag to disable caching
    /// and always query the LLM (useful for testing different prompts).
    #[arg(long, requires = "docs_enhance")]
    pub docs_no_cache: bool,

    /// Custom Ollama server URL
    ///
    /// Specify a custom Ollama server endpoint. Useful for remote Ollama instances
    /// or non-standard ports. Defaults to http://localhost:11434
    #[arg(long, value_name = "URL", requires = "docs_enhance")]
    pub docs_llm_url: Option<String>,

    /// Write enhanced documentation back to source files
    ///
    /// Instead of generating external docs, this modifies the original
    /// source files by adding or updating JSDoc comments with LLM-enhanced
    /// documentation. Creates .bak backup files by default.
    #[arg(long, requires = "docs_enhance")]
    pub docs_write_back: bool,

    /// JSDoc merge strategy when writing back
    ///
    /// Controls how to handle existing JSDoc comments:
    /// - merge: Merge LLM output with existing JSDoc (default, preserves custom tags)
    /// - replace: Replace existing JSDoc entirely with LLM output
    /// - skip: Skip symbols that already have JSDoc
    #[arg(long, value_enum, value_name = "STRATEGY", requires = "docs_write_back")]
    pub docs_merge_strategy: Option<DocsMergeStrategy>,

    /// Skip creating backup files when writing back
    ///
    /// By default, original files are backed up with .bak extension before modification.
    /// Use this flag to skip backup creation (useful for version-controlled code).
    #[arg(long, requires = "docs_write_back")]
    pub docs_no_backup: bool,

    /// External packages to exclude from bundle
    ///
    /// Dependencies listed here will not be included in the output bundle.
    /// They must be provided by the consuming environment.
    ///
    /// Examples:
    ///   --external react --external react-dom
    ///   --external react,react-dom
    #[arg(short, long, value_name = "PACKAGE")]
    pub external: Vec<String>,

    /// Target platform environment
    ///
    /// - browser: Optimizes for browser environments (no Node.js APIs)
    /// - node: Optimizes for Node.js (enables require, process, etc.)
    #[arg(long, value_enum, default_value = "browser")]
    pub platform: Platform,

    /// Source map generation mode
    ///
    /// - inline: Embeds source maps in the bundle (larger file, single file)
    /// - external: Generates separate .map files (smaller bundle, two files)
    /// - hidden: Generates maps but doesn't reference them (debugging builds)
    #[arg(long, value_enum, value_name = "MODE")]
    pub sourcemap: Option<SourceMapMode>,

    /// Enable minification of output
    ///
    /// Reduces bundle size by removing whitespace, shortening variable names,
    /// and applying other optimizations. Recommended for production builds.
    #[arg(short = 'm', long)]
    pub minify: bool,

    /// JavaScript language target version
    ///
    /// Determines which JavaScript features are available in the output.
    /// Later targets produce smaller, faster code but require modern runtimes.
    #[arg(long, value_enum, default_value = "es2020", value_name = "TARGET")]
    pub target: EsTarget,

    /// Global variable name for IIFE/UMD bundles
    ///
    /// When using IIFE format, this is the global variable name that will
    /// contain the exported module. Must be a valid JavaScript identifier.
    ///
    /// Example: --global-name MyLibrary
    #[arg(long, value_parser = parse_global, value_name = "NAME")]
    pub global_name: Option<String>,

    /// Bundle dependencies into output
    ///
    /// - true: Include all dependencies in the bundle (standalone app/library)
    /// - false: Externalize dependencies (require them at runtime)
    ///
    /// Use false for npm packages, true for browser apps.
    #[arg(long, default_value = "true")]
    pub bundle: bool,

    /// Enable code splitting
    ///
    /// Splits the bundle into multiple chunks for lazy loading. Useful for
    /// large applications to reduce initial load time. Only works with ESM format.
    /// Requires --bundle.
    #[arg(long)]
    pub splitting: bool,

    /// Disable tree shaking optimizations
    ///
    /// By default, unused exports are removed. This flag preserves all code,
    /// which may be useful for debugging or specific compatibility requirements.
    #[arg(long)]
    pub no_treeshake: bool,

    /// Clean output directory before build
    ///
    /// Removes all files from the output directory before starting the build.
    /// Ensures no stale artifacts remain from previous builds.
    #[arg(long)]
    pub clean: bool,

    /// Working directory for the build
    ///
    /// All relative paths in the build process are resolved relative to this
    /// directory. Defaults to the current working directory.
    #[arg(long, value_name = "DIR")]
    pub cwd: Option<PathBuf>,
}

/// Arguments for the dev command (development server)
#[derive(Args, Debug)]
pub struct DevArgs {
    /// Entry point for development server
    ///
    /// The main file to serve. If not specified, reads from fob.config.json.
    /// The dev server will rebuild this entry point and its dependencies
    /// whenever files change.
    #[arg(value_name = "ENTRY")]
    pub entry: Option<PathBuf>,

    /// Port for development server
    ///
    /// The HTTP port to listen on. The server will automatically find an
    /// available port if this one is in use.
    #[arg(short, long, default_value = "3000", value_name = "PORT")]
    pub port: u16,

    /// Enable HTTPS with automatic certificate generation
    ///
    /// Generates a self-signed certificate for local HTTPS development.
    /// Useful for testing service workers or other HTTPS-only features.
    #[arg(long)]
    pub https: bool,

    /// Open browser automatically on server start
    ///
    /// Launches the default web browser and navigates to the dev server URL.
    #[arg(long)]
    pub open: bool,

    /// Working directory for the dev server (defaults to auto-detected project root)
    ///
    /// All relative paths are resolved relative to this directory. If not specified,
    /// fob will automatically detect the project root by finding the nearest package.json.
    #[arg(long, value_name = "DIR")]
    pub cwd: Option<PathBuf>,
}

/// Arguments for the init command (project scaffolding)
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name
    ///
    /// Name of the project to create. If omitted, uses the current directory
    /// name. Must be a valid npm package name.
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    /// Template to use for project initialization
    ///
    /// Available templates:
    /// - library: TypeScript library with type declarations
    /// - app: Web application with dev server
    /// - component-library: React component library (aliases: components)
    /// - meta-framework: Educational framework example (aliases: framework)
    ///
    /// If not specified, an interactive prompt will ask for preferences.
    #[arg(short, long, value_name = "TEMPLATE")]
    pub template: Option<String>,

    /// Skip interactive prompts and use defaults
    ///
    /// Creates a basic project without asking for customization options.
    /// Useful for automated workflows or quick prototyping.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Use npm as package manager (default)
    #[arg(long, conflicts_with_all = ["use_yarn", "use_pnpm"])]
    pub use_npm: bool,

    /// Use Yarn as package manager
    #[arg(long, conflicts_with_all = ["use_npm", "use_pnpm"])]
    pub use_yarn: bool,

    /// Use pnpm as package manager
    #[arg(long, conflicts_with_all = ["use_npm", "use_yarn"])]
    pub use_pnpm: bool,
}

/// Arguments for the check command (configuration validation)
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Path to fob.config.json
    ///
    /// Specify a custom configuration file location. If not provided,
    /// searches for fob.config.json in the current directory.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Also validate package.json dependencies
    ///
    /// Checks that all dependencies are installed and that their versions
    /// are compatible with the project requirements.
    #[arg(long)]
    pub deps: bool,

    /// Show warnings in addition to errors
    ///
    /// Displays potential issues that won't prevent building but might
    /// indicate configuration problems or suboptimal settings.
    #[arg(short, long)]
    pub warnings: bool,
}

