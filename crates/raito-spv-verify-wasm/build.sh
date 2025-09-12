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

# Browser via bundlers
wasm-pack build --release --target bundler --out-dir dist/bundler --out-name index

# Node.js (fs loads the .wasm)
wasm-pack build --release --target nodejs  --out-dir dist/node    --out-name index
