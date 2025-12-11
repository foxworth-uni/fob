#!/usr/bin/env python3
"""
Simple Fob Bundler Example

This demonstrates the most basic way to use Fob from Python.
Perfect for getting started with JavaScript bundling in Python!
"""

import asyncio
import sys
from pathlib import Path
import os

# Import fob Python bindings
try:
    import fob
except ImportError:
    print("❌ Error: fob module not found!")
    print("\nTo install:")
    print("  cd ../../..")
    print("  maturin develop --manifest-path crates/fob-python/Cargo.toml")
    sys.exit(1)


async def main():
    """Main bundling function"""
    print("🚀 Building with Fob...\n")

    try:
        # Initialize logging (optional, defaults to info level)
        fob.init_logging("info")

        # Method 1: Using the simple helper function
        # This is the easiest way to bundle a single file
        # Get absolute paths based on script location
        script_dir = Path(__file__).parent.absolute()
        entry_path = script_dir / "src" / "index.js"
        output_dir = script_dir / "dist"
        
        # Change to script directory so relative imports work correctly
        original_cwd = os.getcwd()
        os.chdir(script_dir)
        
        try:
            result = await fob.bundle_single(
                entry=str(entry_path),
                output_dir=str(output_dir),
                format="esm"
            )
        finally:
            os.chdir(original_cwd)

        # Method 2: Using the Fob class for more control
        # Uncomment to try this instead:
        # bundler = fob.Fob({
        #     "entries": [str(entry_path)],
        #     "output_dir": str(output_dir),
        #     "format": "esm",
        #     "sourcemap": "external",
        # })
        # result = await bundler.bundle()

        # Method 3: Using preset methods
        # Uncomment to try this instead:
        # result = await fob.Fob.bundle_entry(
        #     str(entry_path),
        #     {"out_dir": str(output_dir), "format": "esm"}
        # )

        # Display results
        print("✅ Build complete!\n")

        print("📦 Chunks generated:")
        for chunk in result["chunks"]:
            print(f"  - {chunk['file_name']} ({chunk['size']} bytes)")

        print("\n📊 Build stats:")
        stats = result["stats"]
        print(f"  Modules: {stats['total_modules']}")
        print(f"  Chunks: {stats['total_chunks']}")
        print(f"  Total size: {stats['total_size']} bytes")
        print(f"  Duration: {stats['duration_ms']}ms")
        print(f"  Cache hit rate: {stats['cache_hit_rate']:.1%}")

        if result["assets"]:
            print("\n📁 Assets:")
            for asset in result["assets"]:
                print(f"  - {asset['relative_path']} ({asset['size']} bytes)")

        print(f"\n💡 Output written to: dist/")
        print("\n🚀 Try it:")
        print("   node dist/index.js")

    except fob.FobError as e:
        print(f"❌ Build failed: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    # Run the async main function
    asyncio.run(main())
