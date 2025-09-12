# Raito SPV Verify WASM

This crate provides WebAssembly bindings for the `raito-spv-verify` library, allowing you to verify compressed SPV proofs directly in web browsers and Node.js environments.

## Features

- **WASM-compatible**: Compiles to WebAssembly for cross-platform compatibility
- **Async support**: Non-blocking verification for better user experience
- **Error handling**: Comprehensive error reporting with detailed messages
- **Configuration management**: Easy creation of verification configurations
- **Type safety**: Full TypeScript/JavaScript type safety through WASM bindings

## Installation

### For Node.js

```bash
npm install @raito-stark/spv-verify-wasm
```

### For Web Browsers

```html
<script src="https://unpkg.com/@raito-stark/spv-verify-wasm@latest/dist/raito_spv_verify_wasm.js"></script>
```

## Usage

### Basic Verification

```javascript
import { verify_proof_async, create_default_config } from '@raito-stark/spv-verify-wasm';

// Create a default verification configuration
const config = create_default_config();

// Verify a proof asynchronously
try {
    const result = await verify_proof_async(proofData, config, false);
    
    if (result.success) {
        console.log('Proof verification successful!');
    } else {
        console.error('Verification failed:', result.error.message);
    }
} catch (error) {
    console.error('Verification error:', error);
}
```

### Custom Configuration

```javascript
import { create_custom_config } from '@raito-stark/spv-verify-wasm';

// Create a custom verification configuration
const config = create_custom_config(
    "1813388729421943762059264", // min_work
    "0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3", // bootloader_hash
    "0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca", // task_program_hash
    8 // task_output_size
);
```

### Synchronous Verification (Blocking)

```javascript
import { verify_proof_sync } from '@raito-stark/spv-verify-wasm';

// Note: This will block the main thread
const result = verify_proof_sync(proofData, config, false);

if (result.success) {
    console.log('Proof verification successful!');
} else {
    console.error('Verification failed:', result.error.message);
}
```

## API Reference

### Functions

#### `verify_proof_async(proof, config, dev)`

Asynchronously verifies a compressed SPV proof.

- **`proof`**: The compressed SPV proof data
- **`config`**: Verification configuration object
- **`dev`**: Development mode flag (boolean)
- **Returns**: Promise that resolves to a verification result

#### `verify_proof_sync(proof, config, dev)`

Synchronously verifies a compressed SPV proof (blocks main thread).

- **`proof`**: The compressed SPV proof data
- **`config`**: Verification configuration object
- **`dev`**: Development mode flag (boolean)
- **Returns**: Verification result object

#### `create_default_config()`

Creates a default verification configuration.

- **Returns**: Default configuration object

#### `create_custom_config(min_work, bootloader_hash, task_program_hash, task_output_size)`

Creates a custom verification configuration.

- **`min_work`**: Minimum cumulative work required (decimal string)
- **`bootloader_hash`**: Expected bootloader program hash (hex string)
- **`task_program_hash`**: Expected payload program hash (hex string)
- **`task_output_size`**: Expected payload program output size in felts
- **Returns**: Custom configuration object

### Types

#### `VerificationResult`

```typescript
interface VerificationResult {
    success: boolean;
    error?: VerificationError;
}
```

#### `VerificationError`

```typescript
interface VerificationError {
    message: string;
}
```

#### `VerifierConfig`

```typescript
interface VerifierConfig {
    min_work: string;
    bootloader_hash: string;
    task_program_hash: string;
    task_output_size: number;
}
```

## Building from Source

### Prerequisites

- Rust toolchain (latest stable)
- `wasm-pack` for building WASM
- Node.js and npm

### Build Steps

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build the WASM crate
cd crates/raito-spv-verify-wasm
wasm-pack build --target web  # For web browsers
wasm-pack build --target nodejs  # For Node.js
```

## Development Mode

When `dev` is set to `true`, the verification will skip certain checks that are useful during development and testing:

- Chain height validation
- Block MMR root consistency checks

**Warning**: Do not use development mode in production environments.

## Error Handling

The library provides comprehensive error handling with detailed error messages. Common error scenarios include:

- Invalid proof data format
- Mismatched chain heights
- Insufficient subchain work
- Cairo proof verification failures
- Transaction inclusion proof failures

## Performance Considerations

- **Async verification** is recommended for production use to avoid blocking the main thread
- **Sync verification** should only be used for small proofs or in worker threads
- Large proofs may take several seconds to verify
- Consider implementing progress indicators for better user experience

## Browser Compatibility

- Modern browsers with WebAssembly support
- Chrome 57+, Firefox 52+, Safari 11+, Edge 79+
- Node.js 12+ with `--experimental-wasm-threads` flag for better performance

## License

This project is licensed under the same license as the main Raito project. 