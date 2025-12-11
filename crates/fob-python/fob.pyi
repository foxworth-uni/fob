"""Type stubs for the fob bundler Python bindings."""

from typing import TypedDict, Literal, Optional, Union, List, Dict, Any
from pathlib import Path

# Entry types for flexible entries API
class Entry(TypedDict, total=False):
    """Single entry point - file path or inline content."""
    path: str
    """File path (mutually exclusive with content)."""
    content: str
    """Inline JavaScript/TypeScript code (mutually exclusive with path)."""
    name: str
    """Output name (required for content entries, optional for path entries)."""
    loader: Literal['js', 'ts', 'tsx', 'jsx']
    """File type hint for inline content."""

Entries = Union[str, Path, List[Union[str, Path]], Entry, List[Union[str, Path, Entry]]]
"""Flexible entries input - supports strings, paths, arrays, and entry objects."""

# Config types
class MdxOptions(TypedDict, total=False):
    """MDX compilation options."""
    gfm: bool
    """Enable GitHub Flavored Markdown (default: True)."""
    footnotes: bool
    """Enable footnotes (default: True)."""
    math: bool
    """Enable math support (default: True)."""
    jsx_runtime: str
    """JSX runtime module path (default: 'react/jsx-runtime')."""
    use_default_plugins: bool
    """Use default plugins (default: True)."""

class CodeSplittingConfig(TypedDict, total=False):
    """Configuration for code splitting."""
    min_size: int
    """Minimum chunk size in bytes (default: 20000)."""
    min_imports: int
    """Minimum number of entry points that must import the same module (default: 2)."""

class BundleConfig(TypedDict, total=False):
    """Bundle configuration."""
    entries: Entries
    """Entry points to bundle - strings, paths, arrays, or entry objects with inline content."""
    output_dir: str
    """Output directory (defaults to 'dist')."""
    format: Literal['esm', 'cjs', 'iife']
    """Output format (default: 'esm')."""
    sourcemap: Union[bool, Literal['true', 'false', 'inline', 'hidden', 'external']]
    """Source map generation mode."""
    platform: Literal['browser', 'node']
    """Target platform (default: 'browser')."""
    minify: bool
    """Enable minification (default: False)."""
    cwd: str
    """Working directory for resolution."""
    mdx: MdxOptions
    """MDX compilation options."""
    entry_mode: Literal['shared', 'isolated']
    """Entry mode: 'shared' for shared chunks, 'isolated' for standalone bundles."""
    code_splitting: CodeSplittingConfig
    """Code splitting configuration."""
    external: List[str]
    """Packages to externalize (not bundled)."""
    external_from_manifest: bool
    """Externalize dependencies from package.json."""

class BuildOptions(TypedDict, total=False):
    """Common build options for preset functions."""
    out_dir: str
    """Output directory (defaults to 'dist')."""
    format: Literal['esm', 'cjs', 'iife']
    """Output format (default: 'esm')."""
    sourcemap: Union[bool, Literal['true', 'false', 'inline', 'hidden', 'external']]
    """Source map generation mode."""
    external: List[str]
    """Packages to externalize."""
    platform: Literal['browser', 'node']
    """Target platform (default: 'browser')."""
    minify: bool
    """Enable minification."""
    cwd: str
    """Working directory for resolution."""

class AppOptions(TypedDict, total=False):
    """Options for app builds with code splitting."""
    out_dir: str
    """Output directory (defaults to 'dist')."""
    format: Literal['esm', 'cjs', 'iife']
    """Output format (default: 'esm')."""
    sourcemap: Union[bool, Literal['true', 'false', 'inline', 'hidden', 'external']]
    """Source map generation mode."""
    external: List[str]
    """Packages to externalize."""
    platform: Literal['browser', 'node']
    """Target platform (default: 'browser')."""
    minify: bool
    """Enable minification."""
    cwd: str
    """Working directory for resolution."""
    code_splitting: CodeSplittingConfig
    """Code splitting configuration."""

# Result types
class ModuleInfo(TypedDict):
    """Module information."""
    path: str
    """Module path."""
    size: Optional[int]
    """Module size in bytes."""
    has_side_effects: Optional[bool]
    """Has side effects."""

class ChunkInfo(TypedDict):
    """Detailed chunk information."""
    id: str
    """Chunk identifier."""
    kind: Literal['entry', 'async', 'shared']
    """Chunk type."""
    file_name: str
    """Output file name."""
    code: str
    """Generated code."""
    source_map: Optional[str]
    """Source map."""
    modules: List[ModuleInfo]
    """Modules in this chunk."""
    imports: List[str]
    """Static imports."""
    dynamic_imports: List[str]
    """Dynamic imports."""
    size: int
    """Size in bytes."""

class ChunkMetadata(TypedDict):
    """Chunk metadata."""
    file: str
    imports: List[str]
    dynamic_imports: List[str]
    css: List[str]

class ManifestInfo(TypedDict):
    """Bundle manifest."""
    entries: Dict[str, str]
    """Entry mappings."""
    chunks: Dict[str, ChunkMetadata]
    """Chunk metadata."""
    version: str
    """Version."""

class BuildStatsInfo(TypedDict):
    """Build statistics."""
    total_modules: int
    total_chunks: int
    total_size: int
    duration_ms: int
    cache_hit_rate: float

class AssetInfo(TypedDict):
    """Asset information."""
    public_path: str
    relative_path: str
    size: int
    format: Optional[str]

class BundleResult(TypedDict):
    """Result of a bundle operation."""
    chunks: List[ChunkInfo]
    """Generated chunks."""
    manifest: ManifestInfo
    """Bundle manifest."""
    stats: BuildStatsInfo
    """Build statistics."""
    assets: List[AssetInfo]
    """Static assets."""
    module_count: int
    """Total module count."""

# Functions
def init_logging(level: Optional[Literal['silent', 'error', 'warn', 'info', 'debug']] = None) -> None:
    """Initialize fob logging with specified level.

    Call this once at application startup before any fob operations.

    Args:
        level: Log level (default: 'info')
    """
    ...

def init_logging_from_env() -> None:
    """Initialize logging from RUST_LOG environment variable.

    Falls back to Info level if RUST_LOG is not set or invalid.
    """
    ...

async def bundle_single(
    entry: Union[str, Path],
    output_dir: Union[str, Path],
    format: Optional[Literal['esm', 'cjs', 'iife']] = None
) -> BundleResult:
    """Quick helper to bundle a single entry.

    Args:
        entry: Entry file path
        output_dir: Output directory path
        format: Output format (default: 'esm')

    Returns:
        Bundle result
    """
    ...

def version() -> str:
    """Get the bundler version."""
    ...

# Main class
class Fob:
    """Fob bundler for Python."""

    def __init__(self, config: BundleConfig) -> None:
        """Create a new bundler instance with full configuration control.

        Args:
            config: Bundle configuration dictionary

        Example:
            ```python
            bundler = fob.Fob({
                "entries": ["src/index.ts"],
                "output_dir": "dist",
                "format": "esm"
            })
            ```
        """
        ...

    async def bundle(self) -> BundleResult:
        """Bundle the configured entries and return detailed bundle information.

        Returns:
            Bundle result containing chunks, manifest, stats, and assets
        """
        ...

    @staticmethod
    async def bundle_entry(
        entry: Union[str, Path],
        options: Optional[BuildOptions] = None
    ) -> BundleResult:
        """Build a standalone bundle (single entry, full bundling).

        Best for: Applications, scripts, or single-file outputs.

        Args:
            entry: Entry file path
            options: Build options

        Returns:
            Bundle result
        """
        ...

    @staticmethod
    async def library(
        entry: Union[str, Path],
        options: Optional[BuildOptions] = None
    ) -> BundleResult:
        """Build a library (single entry, externalize dependencies).

        Best for: npm packages, reusable libraries.
        Dependencies are marked as external and not bundled.

        Args:
            entry: Entry file path
            options: Build options

        Returns:
            Bundle result
        """
        ...

    @staticmethod
    async def app(
        entries: List[Union[str, Path]],
        options: Optional[AppOptions] = None
    ) -> BundleResult:
        """Build an app with code splitting (multiple entries, unified output).

        Best for: Web applications with multiple pages/routes.
        Shared dependencies are extracted into common chunks.

        Args:
            entries: Entry file paths
            options: App build options

        Returns:
            Bundle result
        """
        ...

    @staticmethod
    async def components(
        entries: List[Union[str, Path]],
        options: Optional[BuildOptions] = None
    ) -> BundleResult:
        """Build a component library (multiple entries, separate bundles).

        Best for: UI component libraries, design systems.
        Each entry produces an independent bundle with no shared chunks.

        Args:
            entries: Entry file paths
            options: Build options

        Returns:
            Bundle result
        """
        ...
