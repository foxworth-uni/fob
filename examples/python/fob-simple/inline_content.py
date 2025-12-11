#!/usr/bin/env python3
"""
Test inline content feature with Fob bundler (Python)

Demonstrates bundling inline JavaScript/TypeScript code without file I/O.
"""

import asyncio
import sys
import tempfile
import shutil
from pathlib import Path

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
    """Main test function"""
    print("🚀 Testing inline content with Fob (Python)...\n")

    # Create a temporary directory for output
    temp_dir = tempfile.mkdtemp(prefix='fob-inline-test-')
    print(f"📁 Output directory: {temp_dir}\n")

    try:
        # Initialize logging
        fob.init_logging("info")

        # Test 1: Single inline content entry
        print("Test 1: Single inline content entry")
        bundler1 = fob.Fob({
            "entries": [{
                "content": "console.log('Hello from Python inline content!');",
                "name": "main.js",
            }],
            "output_dir": f"{temp_dir}/test1",
            "format": "esm",
        })
        result1 = await bundler1.bundle()
        chunk1 = result1["chunks"][0]
        print(f"✅ Generated: {chunk1['file_name']} ({chunk1['size']} bytes)\n")

        # Test 2: Multiple inline content entries
        print("Test 2: Multiple inline content entries")
        bundler2 = fob.Fob({
            "entries": [
                {
                    "content": "console.log('Entry 1: Hello from inline!');",
                    "name": "entry1.js",
                },
                {
                    "content": "console.log('Entry 2: Another inline file!');",
                    "name": "entry2.js",
                }
            ],
            "output_dir": f"{temp_dir}/test2",
            "format": "esm",
        })
        result2 = await bundler2.bundle()
        print("✅ Chunks generated:")
        for chunk in result2["chunks"]:
            print(f"  - {chunk['file_name']} ({chunk['size']} bytes)")
        print()

        # Test 3: Mixed inline and file entries
        print("Test 3: Mixed inline content and file path")
        script_dir = Path(__file__).parent.absolute()
        bundler3 = fob.Fob({
            "entries": [
                str(script_dir / "src" / "index.js"),  # File path
                {
                    "content": "console.log('Plus inline content!');",
                    "name": "inline.js",
                }
            ],
            "output_dir": f"{temp_dir}/test3",
            "format": "esm",
        })
        result3 = await bundler3.bundle()
        print("✅ Chunks generated:")
        for chunk in result3["chunks"]:
            print(f"  - {chunk['file_name']} ({chunk['size']} bytes)")
        print()

        # Test 4: TypeScript inline content
        print("Test 4: TypeScript inline content")
        bundler4 = fob.Fob({
            "entries": [{
                "content": "const message: string = 'TypeScript works!'; console.log(message);",
                "name": "typed.ts",
                "loader": "ts",
            }],
            "output_dir": f"{temp_dir}/test4",
            "format": "esm",
        })
        result4 = await bundler4.bundle()
        chunk4 = result4["chunks"][0]
        print(f"✅ Generated: {chunk4['file_name']} ({chunk4['size']} bytes)\n")

        print("✅ All tests passed!\n")
        print("📊 Summary:")
        print(f"  Test 1: {result1['stats']['total_modules']} modules, {result1['stats']['total_size']} bytes")
        print(f"  Test 2: {result2['stats']['total_modules']} modules, {result2['stats']['total_size']} bytes")
        print(f"  Test 3: {result3['stats']['total_modules']} modules, {result3['stats']['total_size']} bytes")
        print(f"  Test 4: {result4['stats']['total_modules']} modules, {result4['stats']['total_size']} bytes")

    except fob.FobError as e:
        print(f"❌ Test failed: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        # Clean up temp directory
        print(f"\n🧹 Cleaning up {temp_dir}")
        shutil.rmtree(temp_dir, ignore_errors=True)


if __name__ == "__main__":
    # Run the async main function
    asyncio.run(main())
