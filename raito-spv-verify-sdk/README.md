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
  
  // Fetch recent proven height
  const recentHeight = await sdk.fetchRecentProvenHeight();
  console.log('Most recent proven block height:', recentHeight);
  
  // Fetch and verify a transaction
  const txid = '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';
  
  // Fetch the proof from Raito API
  const proof = await sdk.fetchProof(txid);

  // Verify the proof
  const isValid = await sdk.verifyProof(proof);

  console.log('Verification result:', isValid ? 'Valid' : 'Invalid');
}
```

### Block Proof Usage

```javascript
import { createRaitoSpvSdk, fetchBlockProof } from '@starkware-bitcoin/spv-verify';

async function fetchBlockProofExample() {
  // Create SDK instance
  const sdk = createRaitoSpvSdk();
  
  // Get recent proven height
  const recentHeight = await sdk.fetchRecentProvenHeight();
  
  // Fetch block proof for a specific block (e.g., 100 blocks before recent height)
  const blockHeight = recentHeight - 100;
  const blockProof = await sdk.fetchBlockProof(blockHeight, recentHeight);
  
  console.log('Block proof:', blockProof);
  console.log('Leaf index:', blockProof.leaf_index);
  console.log('Peaks hashes:', blockProof.peaks_hashes);
  
  // Or use the standalone function
  const blockProof2 = await fetchBlockProof(blockHeight, recentHeight);
  console.log('Same result:', blockProof.leaf_index === blockProof2.leaf_index);
}
```

### Chain State Proof Usage

```javascript
import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify';
import * as chainStateProof from '@starkware-bitcoin/spv-verify/chain-state-proof';

async function chainStateProofExample() {
  // Create SDK instance
  const sdk = createRaitoSpvSdk();
  await sdk.init();
  
  // Create default configuration
  const defaultConfig = await chainStateProof.createDefaultConfig(sdk.wasmModule);
  console.log('Default config:', defaultConfig);
  
  // Create custom configuration
  const customConfig = await chainStateProof.createCustomConfig(
    sdk.wasmModule,
    '1000000000000000000000000000000',
    '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    100
  );
  
  // Fetch chain state proof directly from RPC
  const chainStateProofString = await chainStateProof.fetchChainStateProof(sdk.raitoRpcUrl);
  const chainStateProofData = JSON.parse(chainStateProofString);
  const chainState = chainStateProofData.chainstate;
  const proofData = JSON.stringify(chainStateProofData.proof);
  
  // Verify chain state
  const mmrHash = await chainStateProof.verifyChainState(sdk.wasmModule, chainState, proofData, customConfig);
  console.log('Chain state verification MMR hash:', mmrHash);
  
  // Verify subchain work
  const blockHeight = chainState.block_height - 100;
  const workResult = await chainStateProof.verifySubchainWork(sdk.wasmModule, blockHeight, chainState, defaultConfig);
  console.log('Subchain work verification:', workResult);
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

#### `fetchRecentProvenHeight(): Promise<number>`

Fetches the most recent proven block height from the Raito API.

- **Returns**: Promise that resolves to the most recent proven block height as a number

#### `fetchProof(txid: string): Promise<string>`

Fetches a compressed SPV proof for a given transaction ID from the Raito API.

- **`txid`**: Bitcoin transaction ID (hex string)
- **Returns**: Promise that resolves to the proof data as a JSON string

#### `verifyProof(proof: string, config?: Partial<VerifierConfig>): Promise<boolean>`

Verifies a compressed SPV proof.

- **`proof`**: The compressed SPV proof data (JSON string)
- **`config`**: Optional partial verification configuration to override defaults
- **Returns**: Promise that resolves to `true` if verification succeeds, `false` otherwise

#### `fetchBlockProof(blockHeight: number, chainHeight: number, dev?: boolean): Promise<BlockInclusionProof>`

Fetches a block MMR inclusion proof from the Raito bridge RPC.

- **`blockHeight`**: Height of the block to prove
- **`chainHeight`**: Current best height (chain head)
- **`dev`**: Whether to use development mode (default: false)
- **Returns**: Promise that resolves to a BlockInclusionProof object

#### `getMmrHeight(): Promise<number>`

Gets the current MMR height from the Raito bridge RPC.

- **Returns**: Promise that resolves to the current MMR height as a number

### Chain State Proof Functions

The chain state proof verification functions are available as standalone functions that can be imported from the `chain-state-proof` module:

#### `fetchChainStateProof(raitoRpcUrl: string): Promise<string>`

Fetches the latest chain state proof from the Raito bridge RPC.

- **`raitoRpcUrl`**: URL of the Raito bridge RPC endpoint
- **Returns**: Promise that resolves to the chain state proof as a JSON string

#### `verifyChainState(wasmModule: any, chainState: ChainState, chainStateProof: string, config: VerifierConfig): Promise<string>`

Verifies the Cairo recursive proof and consistency of the bootloader output with chain state.

- **`wasmModule`**: The initialized WASM module
- **`chainState`**: The chain state data to verify
- **`chainStateProof`**: The chain state proof data (JSON string)
- **`config`**: The verifier configuration
- **Returns**: Promise that resolves to the MMR hash on success

#### `verifySubchainWork(wasmModule: any, blockHeight: number, chainState: ChainState, config: VerifierConfig): Promise<boolean>`

Verifies that there is enough work added on top of the target block.

- **`wasmModule`**: The initialized WASM module
- **`blockHeight`**: Height of the block to verify work for
- **`chainState`**: The chain state data
- **`config`**: The verifier configuration
- **Returns**: Promise that resolves to `true` if verification succeeds

#### `createDefaultConfig(wasmModule: any): Promise<VerifierConfig>`

Creates a default verifier configuration.

- **`wasmModule`**: The initialized WASM module
- **Returns**: Promise that resolves to the default VerifierConfig

#### `createCustomConfig(wasmModule: any, minWork: string, bootloaderHash: string, taskProgramHash: string, taskOutputSize: number): Promise<VerifierConfig>`

Creates a custom verifier configuration.

- **`wasmModule`**: The initialized WASM module
- **`minWork`**: Minimum work required for verification
- **`bootloaderHash`**: Hash of the bootloader program
- **`taskProgramHash`**: Hash of the task program
- **`taskOutputSize`**: Size of the task output
- **Returns**: Promise that resolves to the custom VerifierConfig

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

#### `BlockInclusionProof`

```typescript
interface BlockInclusionProof {
  peaks_hashes: string[];
  siblings_hashes: string[];
  leaf_index: number;
  leaf_count: number;
}
```

#### `ChainState`

```typescript
interface ChainState {
  /** The height of the best block in the chain */
  block_height: number;
  /** The total accumulated work of the chain as a decimal string */
  total_work: string;
  /** The hash of the best block in the chain */
  best_block_hash: string;
  /** The current target difficulty as a compact decimal string */
  current_target: string;
  /** The start time (UNIX seconds) of the current difficulty epoch */
  epoch_start_time: number;
  /** The timestamps (UNIX seconds) of the previous 11 blocks */
  prev_timestamps: number[];
}
```



#### `RaitoSpvSdk`

```typescript
class RaitoSpvSdk {
  constructor(raitoRpcUrl?: string);
  init(): Promise<void>;
  fetchRecentProvenHeight(): Promise<number>;
  fetchProof(txid: string): Promise<string>;
  verifyProof(proof: string, config?: Partial<VerifierConfig>): Promise<boolean>;
  fetchBlockProof(blockHeight: number, chainHeight: number, dev?: boolean): Promise<BlockInclusionProof>;
  getMmrHeight(): Promise<number>;
}
```

### Standalone Functions

#### `fetchBlockProof(blockHeight: number, chainHeight: number, raitoRpcUrl?: string, dev?: boolean): Promise<BlockInclusionProof>`

Standalone function to fetch block proof (convenience function).

- **`blockHeight`**: Height of the block to prove
- **`chainHeight`**: Current best height (chain head)
- **`raitoRpcUrl`**: URL of the Raito bridge RPC endpoint (default: 'https://api.raito.wtf')
- **`dev`**: Whether to use development mode (default: false)
- **Returns**: Promise that resolves to a BlockInclusionProof object

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

### Block Proof Example

```bash
# Run the block proof example
node examples/block-proof-example.js
```

### Web Browser Example

```bash
# Start the web example development server
cd examples/web-example
npm install
npm run dev
```
