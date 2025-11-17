#!/usr/bin/env bash
# Build Fob for wasm32-wasip1 target using WASI Component Model
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building Fob for wasm32-wasip1 (WASI Component Model, no threading)...${NC}"

# Check for required tools
if ! command -v wasm-tools &> /dev/null; then
    echo -e "${YELLOW}wasm-tools not found. Installing...${NC}"
    cargo install wasm-tools
fi

if ! command -v jco &> /dev/null; then
    echo -e "${YELLOW}jco not found. Installing...${NC}"
    npm install -g @bytecodealliance/jco@1.8.0
fi

# Check for wasm32-wasip1 target
if ! rustup target list | grep -q "wasm32-wasip1 (installed)"; then
    echo -e "${YELLOW}wasm32-wasip1 target not installed. Installing...${NC}"
    rustup target add wasm32-wasip1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Determine build profile from first argument (default: release)
PROFILE="${1:-release}"
BUILD_FLAGS=()

if [[ "$PROFILE" == "dev" ]]; then
    echo -e "${YELLOW}Building in DEV mode (fast, no optimization)${NC}"
    BUILD_FLAGS=()
    TARGET_DIR="debug"
elif [[ "$PROFILE" == "release" ]]; then
    echo -e "${YELLOW}Building in RELEASE mode (optimized)${NC}"
    BUILD_FLAGS=("--release")
    TARGET_DIR="release"
else
    echo -e "${RED}Error: Invalid profile '$PROFILE'${NC}"
    echo "Usage: $0 [dev|release]"
    echo "  dev     - Fast builds for development"
    echo "  release - Optimized builds for production"
    exit 1
fi

# Step 1: Build core WASM module with wit-bindgen (NOT using cargo-component)
echo -e "${YELLOW}Step 1: Building core WASM module...${NC}"
cd "$SCRIPT_DIR"
if [[ ${#BUILD_FLAGS[@]} -eq 0 ]]; then
    cargo build --target wasm32-wasip1 --package fob-wasm
else
    cargo build --target wasm32-wasip1 --package fob-wasm "${BUILD_FLAGS[@]}"
fi

CORE_WASM="$REPO_ROOT/target/wasm32-wasip1/$TARGET_DIR/fob_bundler_wasm.wasm"
OUTPUT_DIR="$SCRIPT_DIR/pkg/$TARGET_DIR"
JCO_OUTPUT_DIR="$OUTPUT_DIR/bundler"
COMPONENT_WASM="$OUTPUT_DIR/fob_bundler.component.wasm"

# Check if core module was built
if [[ ! -f "$CORE_WASM" ]]; then
    echo -e "${RED}Error: Core WASM file not found at $CORE_WASM${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Core WASM module built ($(ls -lh "$CORE_WASM" | awk '{print $5}'))${NC}"

# Step 2: Componentize the core module
echo -e "${YELLOW}Step 2: Componentizing WASM module...${NC}"
mkdir -p "$OUTPUT_DIR"

# Try to componentize with wasm-tools (should work now without threading)
if wasm-tools component new "$CORE_WASM" -o "$COMPONENT_WASM" 2>&1 | tee /tmp/wasm-tools-output.log; then
    echo -e "${GREEN}✓ Component created successfully${NC}"
    WASM_TO_USE="$COMPONENT_WASM"
    IS_COMPONENT=true
else
    echo -e "${YELLOW}Warning: Component creation failed${NC}"
    echo -e "${YELLOW}Falling back to raw WASM module approach${NC}"
    cp "$CORE_WASM" "$OUTPUT_DIR/fob_bundler_wasm.wasm"
    WASM_TO_USE="$OUTPUT_DIR/fob_bundler_wasm.wasm"
    IS_COMPONENT=false
fi

# Step 3: Create JavaScript bindings
mkdir -p "$JCO_OUTPUT_DIR"

if [ "$IS_COMPONENT" = true ]; then
    # Try jco transpile for component
    echo -e "${YELLOW}Step 3: Transpiling component with jco...${NC}"
    if jco transpile "$COMPONENT_WASM" -o "$JCO_OUTPUT_DIR" --name fob-bundler 2>&1; then
        echo -e "${GREEN}✓ TypeScript bindings generated${NC}"
        cp "$COMPONENT_WASM" "$JCO_OUTPUT_DIR/fob_bundler.component.wasm"
    else
        echo -e "${YELLOW}Warning: jco transpile failed${NC}"
        IS_COMPONENT=false
    fi
fi

if [ "$IS_COMPONENT" = false ]; then
    # Fallback: Copy WASM file but DON'T overwrite custom bindings if they exist
    echo -e "${YELLOW}Step 3: Copying WASM file (preserving custom bindings if present)...${NC}"
    
    # Copy the WASM file
    cp "$CORE_WASM" "$JCO_OUTPUT_DIR/fob_bundler_wasm.wasm"
    
    # Check if custom bindings already exist (file > 1000 bytes means it's custom)
    if [[ -f "$JCO_OUTPUT_DIR/fob-bundler.js" ]] && [[ $(wc -c < "$JCO_OUTPUT_DIR/fob-bundler.js") -gt 1000 ]]; then
        echo -e "${GREEN}✓ Custom bindings detected, preserving them${NC}"
    else
        echo -e "${YELLOW}Creating stub bindings (replace with custom implementation)${NC}"
        
        # Create a simple JavaScript wrapper
        cat > "$JCO_OUTPUT_DIR/fob-bundler.js" << 'EOF'
// Manual bindings for raw WASM module with WASI
// This file provides a bridge between the WASM module and JavaScript

export async function bundle(config) {
  throw new Error('Raw WASM bindings not yet implemented. Requires WASI runtime setup.');
}

export function getRuntimeVersion() {
  return '0.1.0';
}

// Export for CommonJS compatibility
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { bundle, getRuntimeVersion };
}
EOF

        # Create TypeScript definitions
        cat > "$JCO_OUTPUT_DIR/fob-bundler.d.ts" << 'EOF'
export interface BundleConfig {
  entries: string[];
  outputDir: string;
  format?: string;
  sourcemap?: boolean;
}

export interface BundleResult {
  assetsCount: number;
  success: boolean;
  error?: string;
}

export function bundle(config: BundleConfig): Promise<{ tag: 'ok', val: BundleResult } | { tag: 'err', val: string }>;
export function getRuntimeVersion(): string;
EOF
    fi
fi

# Copy to packages/fob-edge/wasm/bundler
EDGE_BUNDLER_DIR="$REPO_ROOT/packages/fob-edge/wasm/bundler"
mkdir -p "$EDGE_BUNDLER_DIR"
cp -r "$JCO_OUTPUT_DIR"/* "$EDGE_BUNDLER_DIR/" 2>/dev/null || true

echo -e "\n${GREEN}✓ Build complete${NC}"

# Show output sizes
echo -e "\n${YELLOW}Build artifacts:${NC}"
ls -lh "$OUTPUT_DIR"/*.wasm 2>/dev/null | awk '{print $5 "\t" $9}' || true
echo ""
ls -lh "$JCO_OUTPUT_DIR"/*.js 2>/dev/null | awk '{print $5 "\t" $9}' || true

echo -e "\n${GREEN}Output locations:${NC}"
echo "  - Core WASM: $CORE_WASM"
if [ "$IS_COMPONENT" = true ]; then
    echo "  - Component: $COMPONENT_WASM"
fi
echo "  - Bindings: $JCO_OUTPUT_DIR/"
echo "  - Copied to: $EDGE_BUNDLER_DIR/"

if [ "$IS_COMPONENT" = false ]; then
    echo -e "\n${YELLOW}Note: Component Model build failed.${NC}"
    echo -e "${YELLOW}Using raw WASM module. Additional integration work needed.${NC}"
fi

echo -e "\n${GREEN}To test with wasmtime (if component built):${NC}"
echo "  wasmtime $WASM_TO_USE"
