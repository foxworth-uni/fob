#!/usr/bin/env python3
"""
Advanced Fob Bundler Example

Demonstrates more advanced features:
- Multiple entry points
- Code splitting
- Library mode
- Component library mode
- Custom configuration
"""

import asyncio
import sys
from pathlib import Path

try:
    import fob
except ImportError:
    print("❌ Error: fob module not found!")
    print("\nTo install:")
    print("  cd ../../..")
    print("  maturin develop --manifest-path crates/fob-python/Cargo.toml")
    sys.exit(1)


async def example_bundle_entry():
    """Example: Bundle a single entry point"""
    print("📦 Example 1: Bundle Entry")
    print("-" * 40)
    
    result = await fob.Fob.bundle_entry(
        "src/index.js",
        {
            "out_dir": "dist/bundle-entry",
            "format": "esm",
            "minify": True,
            "sourcemap": "external"
        }
    )
    
    print(f"✅ Bundled {result['stats']['total_modules']} modules")
    print(f"   Output: {result['chunks'][0]['file_name']}\n")


async def example_library():
    """Example: Build a library (externalize dependencies)"""
    print("📚 Example 2: Library Mode")
    print("-" * 40)
    
    result = await fob.Fob.library(
        "src/index.js",
        {
            "out_dir": "dist/library",
            "external": ["react", "react-dom"],  # Can also be a single string
            "format": "esm"
        }
    )
    
    print(f"✅ Library built")
    print(f"   Externalized dependencies from package.json\n")


async def example_app():
    """Example: Build an app with code splitting"""
    print("🌐 Example 3: App Mode (Code Splitting)")
    print("-" * 40)
    
    # Using pathlib.Path for entries
    entries = [Path("src/index.js")]
    
    result = await fob.Fob.app(
        entries,
        {
            "out_dir": "dist/app",
            "code_splitting": {
                "min_size": 1000,  # Minimum chunk size in bytes
                "min_imports": 2   # Minimum shared imports
            },
            "format": "esm"
        }
    )
    
    print(f"✅ App built with code splitting")
    print(f"   Generated {result['stats']['total_chunks']} chunks")
    print(f"   Shared dependencies extracted\n")


async def example_components():
    """Example: Build a component library"""
    print("🧩 Example 4: Component Library")
    print("-" * 40)
    
    result = await fob.Fob.components(
        ["src/index.js"],  # In real usage: ["src/Button.tsx", "src/Card.tsx"]
        {
            "out_dir": "dist/components",
            "format": "esm",
            "external_from_manifest": True
        }
    )
    
    print(f"✅ Component library built")
    print(f"   Each entry produces independent bundle\n")


async def example_custom_config():
    """Example: Full custom configuration"""
    print("⚙️  Example 5: Custom Configuration")
    print("-" * 40)
    
    bundler = fob.Fob({
        "entries": ["src/index.js"],
        "output_dir": "dist/custom",
        "format": "esm",
        "platform": "browser",  # or "node"
        "sourcemap": "inline",
        "minify": False,
        "external": ["lodash"],  # Externalize specific packages
        "cwd": Path.cwd()  # Can use pathlib.Path
    })
    
    result = await bundler.bundle()
    
    print(f"✅ Custom build complete")
    print(f"   Platform: browser")
    print(f"   Sourcemap: inline")
    print(f"   External packages: lodash\n")


async def example_error_handling():
    """Example: Error handling"""
    print("⚠️  Example 6: Error Handling")
    print("-" * 40)
    
    try:
        # This will fail - file doesn't exist
        result = await fob.bundle_single(
            "nonexistent.js",
            "dist",
            "esm"
        )
    except fob.FobError as e:
        print(f"✅ Caught FobError: {e}\n")
    except Exception as e:
        print(f"❌ Unexpected error: {e}\n")


async def main():
    """Run all examples"""
    print("🚀 Advanced Fob Python Examples\n")
    print("=" * 50 + "\n")
    
    # Initialize logging
    fob.init_logging("info")
    
    try:
        await example_bundle_entry()
        await example_library()
        await example_app()
        await example_components()
        await example_custom_config()
        await example_error_handling()
        
        print("=" * 50)
        print("✅ All examples completed!")
        print("\n💡 Check the dist/ directory for outputs")
        
    except Exception as e:
        print(f"\n❌ Example failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
