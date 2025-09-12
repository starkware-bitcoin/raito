#!/bin/bash

# Build script for the complete Raito SPV TypeScript SDK
# This script builds both the WASM module and the TypeScript SDK

set -e

echo "🚀 Building Raito SPV TypeScript SDK..."
echo "======================================"

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found. Please run this script from the typescript-sdk directory."
    exit 1
fi

# Build the WASM module first
echo ""
echo "📦 Building WASM module..."
cd ../crates/raito-spv-verify-wasm
if [ ! -f "build.sh" ]; then
    echo "❌ Error: build.sh not found in raito-spv-verify-wasm directory."
    exit 1
fi

chmod +x build.sh
./build.sh

# Go back to typescript-sdk directory
cd ../../typescript-sdk

# Install dependencies if needed
echo ""
echo "📦 Installing dependencies..."
npm install

# Build the TypeScript SDK
echo ""
echo "🔨 Building TypeScript SDK..."
npm run build

echo ""
echo "✅ Build completed successfully!"
echo ""
echo "📁 Output files:"
echo "  - WASM module: ../crates/raito-spv-verify-wasm/pkg/"
echo "  - TypeScript SDK: ./dist/"
echo ""
echo "🧪 To test:"
echo "  - Node.js: node examples/node-example.js"
echo "  - Browser: open examples/browser-simple.html in a web browser"
echo ""
echo "📦 To publish:"
echo "  - npm publish (from this directory)"
