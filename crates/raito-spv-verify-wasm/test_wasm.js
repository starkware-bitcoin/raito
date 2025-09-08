#!/usr/bin/env node

// Simple test script for raito-spv-verify-wasm
// This script tests the basic functionality without requiring a full build

const fs = require('fs');
const path = require('path');

console.log('Testing raito-spv-verify-wasm crate...');

// Check if the crate compiles
console.log('\n1. Checking if the crate compiles...');
try {
    const { execSync } = require('child_process');
    execSync('cargo check', { cwd: __dirname, stdio: 'pipe' });
    console.log('✅ Crate compiles successfully');
} catch (error) {
    console.log('❌ Crate compilation failed:', error.message);
    process.exit(1);
}

// Check if wasm-pack is available
console.log('\n2. Checking if wasm-pack is available...');
try {
    const { execSync } = require('child_process');
    execSync('wasm-pack --version', { stdio: 'pipe' });
    console.log('✅ wasm-pack is available');
} catch (error) {
    console.log('⚠️  wasm-pack is not available. Install it with: cargo install wasm-pack');
    console.log('   This is required for building the WASM output');
}

// Check file structure
console.log('\n3. Checking file structure...');
const requiredFiles = [
    'Cargo.toml',
    'src/lib.rs',
    'README.md',
    'build.sh',
    'examples/basic_usage.js',
    'examples/web_usage.html'
];

let allFilesExist = true;
for (const file of requiredFiles) {
    const filePath = path.join(__dirname, file);
    if (fs.existsSync(filePath)) {
        console.log(`✅ ${file}`);
    } else {
        console.log(`❌ ${file} - missing`);
        allFilesExist = false;
    }
}

if (!allFilesExist) {
    console.log('\n❌ Some required files are missing');
    process.exit(1);
}

// Check if build script is executable
console.log('\n4. Checking build script permissions...');
const buildScriptPath = path.join(__dirname, 'build.sh');
try {
    const stats = fs.statSync(buildScriptPath);
    if (stats.mode & 0o111) {
        console.log('✅ Build script is executable');
    } else {
        console.log('⚠️  Build script is not executable');
    }
} catch (error) {
    console.log('❌ Could not check build script permissions');
}

console.log('\n🎉 Basic tests completed successfully!');
console.log('\nNext steps:');
console.log('1. Install wasm-pack: cargo install wasm-pack');
console.log('2. Build the WASM output: ./build.sh');
console.log('3. Test the examples: npm test');
console.log('4. Serve the web example: npm run serve'); 