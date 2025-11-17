#!/bin/bash
# Test script for Cloudflare Worker (Rust) example
set -e

echo "🦀 Testing Cloudflare Worker (Rust) Example"
echo "=================================================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Please install from https://rustup.rs"
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Check if worker-build is installed
if ! command -v worker-build &> /dev/null; then
    echo "📦 Installing worker-build..."
    cargo install worker-build
fi

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ wrangler not found. Please install: npm install -g wrangler"
    exit 1
fi

echo "✅ Building Rust worker..."
echo ""

# Build the worker
worker-build --release

echo ""
echo "✅ Starting Wrangler dev server..."
echo ""

# Start wrangler in background
wrangler dev > /tmp/wrangler-rust-dev.log 2>&1 &
WRANGLER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 10

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    kill $WRANGLER_PID 2>/dev/null || true
    wait $WRANGLER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test 1: GET / (HTML page)
echo "📝 Test 1: GET / (HTML demo page)"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8787/)
if [ "$HTTP_CODE" = "200" ]; then
    echo "   ✅ Status: $HTTP_CODE"
    echo "   ✅ HTML page loads successfully"
else
    echo "   ❌ Status: $HTTP_CODE (expected 200)"
    cat /tmp/wrangler-rust-dev.log
    exit 1
fi
echo ""

# Test 2: GET /api/bundle (JSON response)
echo "📝 Test 2: GET /api/bundle (JSON API)"
RESPONSE=$(curl -s http://localhost:8787/api/bundle)
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8787/api/bundle)

if [ "$HTTP_CODE" = "200" ]; then
    echo "   ✅ Status: $HTTP_CODE"
    
    # Check if response is valid JSON
    if echo "$RESPONSE" | jq . > /dev/null 2>&1; then
        echo "   ✅ Valid JSON response"
        
        # Extract and display key fields
        SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
        DURATION=$(echo "$RESPONSE" | jq -r '.meta.duration_ms')
        WORKER=$(echo "$RESPONSE" | jq -r '.meta.worker')
        ASSETS=$(echo "$RESPONSE" | jq -r '.result.assets_count')
        
        echo "   📊 Success: $SUCCESS"
        echo "   ⏱️  Duration: ${DURATION}ms"
        echo "   🔧 Worker: $WORKER"
        echo "   📦 Assets: $ASSETS"
    else
        echo "   ❌ Invalid JSON response"
        echo "$RESPONSE"
        exit 1
    fi
else
    echo "   ❌ Status: $HTTP_CODE (expected 200)"
    cat /tmp/wrangler-rust-dev.log
    exit 1
fi
echo ""

# Test 3: POST /api/bundle (custom code)
echo "📝 Test 3: POST /api/bundle (custom bundling)"
POST_DATA='{
  "files": {
    "index.js": "export const greeting = \"Hello from Rust test!\";",
    "utils.js": "export const add = (a, b) => a + b;"
  },
  "entries": ["index.js"],
  "format": "esm"
}'

RESPONSE=$(curl -s -X POST http://localhost:8787/api/bundle \
  -H "Content-Type: application/json" \
  -d "$POST_DATA")
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8787/api/bundle \
  -H "Content-Type: application/json" \
  -d "$POST_DATA")

if [ "$HTTP_CODE" = "200" ]; then
    echo "   ✅ Status: $HTTP_CODE"
    
    if echo "$RESPONSE" | jq . > /dev/null 2>&1; then
        echo "   ✅ Valid JSON response"
        
        SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
        DURATION=$(echo "$RESPONSE" | jq -r '.meta.duration_ms')
        
        echo "   📊 Success: $SUCCESS"
        echo "   ⏱️  Duration: ${DURATION}ms"
    else
        echo "   ❌ Invalid JSON response"
        echo "$RESPONSE"
        exit 1
    fi
else
    echo "   ❌ Status: $HTTP_CODE (expected 200)"
    cat /tmp/wrangler-rust-dev.log
    exit 1
fi
echo ""

echo "=================================================="
echo "✅ All tests passed!"
echo ""
echo "🌐 Worker is running at: http://localhost:8787"
echo "📖 View the demo: open http://localhost:8787"
echo ""

