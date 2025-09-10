// test.js
const mod = require('./pkg/raito_spv_verify_wasm.js');

// If your crate exports functions, you can call them now.
// The nodejs target initializes synchronously on require.
console.log('Exports:', Object.keys(mod));
console.log('WASM module loaded successfully');