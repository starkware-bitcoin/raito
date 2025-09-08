#!/bin/bash

# Build script for raito-spv-verify-wasm
# This script builds the WASM crate for different targets

set -e

export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'

echo "Building raito-spv-verify-wasm..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed. Please install it first:"
    echo "cargo install wasm-pack"
    exit 1
fi

# Build for web browsers
echo "Building for web browsers..."
wasm-pack build --release --target web --out-dir pkg-web

# Build for Node.js
echo "Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg-node

# Create a combined pkg directory
echo "Creating combined package..."
rm -rf pkg
mkdir pkg

# Copy web files
cp pkg-web/*.wasm pkg/
cp pkg-web/*.js pkg/

# Copy Node.js files (overwrite web files with Node.js versions)
cp pkg-node/*.js pkg/

# Create package.json for npm publishing
cat > pkg/package.json << EOF
{
  "name": "raito-spv-verify-wasm",
  "version": "0.1.0",
  "description": "WebAssembly bindings for Raito SPV verification",
  "main": "raito_spv_verify_wasm.js",
  "module": "raito_spv_verify_wasm.js",
  "types": "raito_spv_verify_wasm.d.ts",
  "files": [
    "*.js",
    "*.wasm",
    "*.d.ts"
  ],
  "keywords": [
    "wasm",
    "bitcoin",
    "spv",
    "verification",
    "cairo",
    "starknet"
  ],
  "author": "Raito Team",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/your-org/raito.git"
  },
  "bugs": {
    "url": "https://github.com/your-org/raito/issues"
  },
  "homepage": "https://github.com/your-org/raito#readme"
}
EOF

# Clean up temporary directories
rm -rf pkg-web pkg-node

echo "Build completed successfully!"
echo "Output files are in the 'pkg' directory"
echo ""
echo "To test the build:"
echo "  cd pkg"
echo "  npm install"
echo "  node -e \"const wasm = require('./raito_spv_verify_wasm.js'); console.log('WASM module loaded successfully');\""
echo ""
echo "To publish to npm:"
echo "  cd pkg"
echo "  npm publish" 