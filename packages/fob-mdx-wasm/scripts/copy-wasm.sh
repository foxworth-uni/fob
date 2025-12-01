#!/bin/bash
set -e

# Copy WASM artifacts from crates to packages
echo "📦 Copying WASM artifacts..."

SOURCE_DIR="../../crates/fob-mdx-wasm/pkg"
DEST_PKG="./pkg"
DEST_PLAYGROUND="./fixtures/playground/pkg"

# Create destination directories if they don't exist
mkdir -p "$DEST_PKG"
mkdir -p "$DEST_PLAYGROUND"

# Copy all files from source to destinations
cp -r "$SOURCE_DIR"/* "$DEST_PKG"/
cp -r "$SOURCE_DIR"/* "$DEST_PLAYGROUND"/

echo "✅ WASM artifacts copied successfully!"
echo "   Source:      $SOURCE_DIR"
echo "   Dest (pkg):  $DEST_PKG"
echo "   Dest (play): $DEST_PLAYGROUND"
