#!/bin/bash

# Copy Node.js WASM build to package directory
mkdir -p ./pkg-node
cp -r ../../crates/fob-mdx-wasm/pkg-node/* ./pkg-node/

# Also copy to playground for testing
mkdir -p ./fixtures/playground/pkg-node
cp -r ../../crates/fob-mdx-wasm/pkg-node/* ./fixtures/playground/pkg-node/

echo "✅ Node.js WASM files copied to pkg-node/"
