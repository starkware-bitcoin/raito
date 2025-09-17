# Raito SPV Verify SDK

A comprehensive TypeScript SDK for fetching and verifying compressed SPV (Simplified Payment Verification) proofs. Built on WebAssembly for high performance in both web browsers and Node.js environments.


## Usage

### Basic Usage

```javascript
import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify';

async function verifyTransaction() {
  // Create SDK instance
  const sdk = createRaitoSpvSdk();
  
  // Initialize the SDK (loads WASM module)
  await sdk.init();
  
  // Fetch and verify a transaction
  const txid = '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';
  
  // Fetch the proof from Raito API
  const proof = await sdk.fetchProof(txid);

  // Verify the proof
  const isValid = await sdk.verifyProof(proof);

  console.log('Verification result:', isValid ? 'Valid' : 'Invalid');
}
```

## API Reference

### Functions

#### `createRaitoSpvSdk(raitoRpcUrl?)`

Creates a new RaitoSpvSdk instance.

- **`raitoRpcUrl`**: Optional custom Raito RPC endpoint URL (defaults to 'https://api.raito.wtf')
- **Returns**: RaitoSpvSdk instance

### RaitoSpvSdk Class

#### `init(): Promise<void>`

Initializes the SDK by loading the WebAssembly module. Must be called before using other methods.

- **Returns**: Promise that resolves when initialization is complete

#### `fetchProof(txid: string): Promise<string>`

Fetches a compressed SPV proof for a given transaction ID from the Raito API.

- **`txid`**: Bitcoin transaction ID (hex string)
- **Returns**: Promise that resolves to the proof data as a JSON string

#### `verifyProof(proof: string, config?: Partial<VerifierConfig>): Promise<boolean>`

Verifies a compressed SPV proof.

- **`proof`**: The compressed SPV proof data (JSON string)
- **`config`**: Optional partial verification configuration to override defaults
- **Returns**: Promise that resolves to `true` if verification succeeds, `false` otherwise

### Types

#### `VerifierConfig`

```typescript
interface VerifierConfig {
  min_work: string;
  bootloader_hash: string;
  task_program_hash: string;
  task_output_size: number;
}
```

#### `RaitoSpvSdk`

```typescript
class RaitoSpvSdk {
  constructor(raitoRpcUrl?: string);
  init(): Promise<void>;
  fetchProof(txid: string): Promise<string>;
  verifyProof(proof: string, config?: Partial<VerifierConfig>): Promise<boolean>;
}
```

## Building from Source

### Prerequisites

- Rust toolchain (latest stable)
- `wasm-pack` for building WASM
- Node.js 18+ and npm

### Build Steps

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build the complete SDK (includes WASM compilation and TypeScript bundling)
npm run build

```

## Examples

The SDK includes complete examples demonstrating different usage patterns:

### Node.js Example

```bash
# Run the Node.js example
node examples/node-example.js
```

### Web Browser Example

```bash
# Start the web example development server
cd examples/web-example
npm install
npm run dev
```
