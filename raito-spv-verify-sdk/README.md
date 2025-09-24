# Raito SPV Verify SDK

A comprehensive TypeScript SDK for fetching and verifying compressed SPV (Simplified Payment Verification) proofs. Built on WebAssembly for high performance in both web browsers and Node.js environments.


## Usage

### Quick Start

```typescript
import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify';

async function main() {
  const sdk = createRaitoSpvSdk();
  await sdk.init();

  const recentHeight = await sdk.fetchRecentProvenHeight();
  console.log('Most recent proven block height:', recentHeight);

  const chainStateResult = await sdk.verifyRecentChainState();
  console.log('MMR root:', chainStateResult.mmrRoot);
  console.log('Chain state height:', chainStateResult.chainState.block_height);

  const blockHeader = await sdk.verifyBlockHeader(
    chainStateResult.chainState.block_height
  );
  console.log('Verified block header prev hash:', blockHeader.prev_blockhash);

  const txid = '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';
  const transaction = await sdk.verifyTransaction(txid);
  console.log('First output value (sats):', transaction.output[0]?.value);
}

main().catch(console.error);
```

### Custom RPC URL and Verifier Configuration

You can direct the SDK to another bridge endpoint or tweak the verifier
configuration that is passed to the WASM backend:

```typescript
const sdk = createRaitoSpvSdk('https://api.raito.wtf', {
  min_work: '1813388729421943762059264',
  bootloader_hash:
    '0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3',
  task_program_hash:
    '0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca',
  task_output_size: 8,
});
```

All fields are optional; omitted values fall back to the defaults shown above.

### Verifying Recent Chain State

- `verifyRecentChainState()` downloads the latest recursive proof from the Raito
  bridge RPC, verifies it with WASM, and caches the result.
- The returned object contains both the verified `mmrRoot` and the parsed
  `chainState` snapshot (including the most recent proven `block_height`).
- Subsequent calls reuse the cached result until a new instance of the SDK is
  created.

### Verifying Block Headers

- Call `verifyBlockHeader(blockHeight)` to fetch the inclusion proof and block
  header for that height. The SDK matches the MMR root with the previously
  verified chain state before returning the header.
- You can supply a `blockHeader` object as the second argument if you already
  have the header; the SDK will skip the fetch and only verify the proof.

### Verifying Transactions

- `verifyTransaction(txid)` retrieves the merkle proof from the bridge, verifies
  it with WASM, and ensures the enclosing block header is part of the verified
  MMR tree.
- The method returns the decoded transaction object, which is cached so repeated
  checks of the same `txid` are free.

## API Reference

### `createRaitoSpvSdk(raitoRpcUrl?, config?)`

- **`raitoRpcUrl`** (optional): custom bridge RPC endpoint. Defaults to
  `https://api.raito.wtf`.
- **`config`** (optional): partial verifier configuration. Missing fields fall
  back to the default verifier settings bundled with the SDK.
- **Returns**: a configured `RaitoSpvSdk` instance.

### `RaitoSpvSdk`

#### `init(): Promise<void>`

Loads and initialises the WASM module. Call once before using the other
methods.

#### `fetchRecentProvenHeight(): Promise<number>`

Fetches the most recent proven Bitcoin block height available from the bridge
API.

#### `verifyRecentChainState(): Promise<ChainStateProofVerificationResult>`

Downloads and verifies the latest recursive proof. Returns an object with the
verified MMR root and the parsed `chainState` snapshot. Results are cached per
SDK instance.

#### `verifyBlockHeader(blockHeight: number, blockHeader?: BlockHeader): Promise<BlockHeader>`

Verifies that the block header at `blockHeight` is included in the proven MMR.
Optionally accepts a pre-fetched header to avoid an extra RPC round trip.

#### `verifyTransaction(txid: string): Promise<Transaction>`

Verifies the merkle proof for `txid`, ensures the enclosing block header is in
the MMR, and returns the parsed transaction data.

### Types

#### `ChainStateProofVerificationResult`

```typescript
interface ChainStateProofVerificationResult {
  mmrRoot: string;
  chainState: ChainState;
}
```

#### `ChainState`

```typescript
interface ChainState {
  block_height: number;
  total_work: string;
  best_block_hash: string;
  current_target: string;
  epoch_start_time: number;
  prev_timestamps: number[];
}
```

#### `BlockHeader`

```typescript
interface BlockHeader {
  version: number;
  prev_blockhash: string;
  merkle_root: string;
  time: number;
  bits: number;
  nonce: number;
}
```

#### `Transaction`

```typescript
interface Transaction {
  version: number;
  lock_time: number;
  input: Array<{
    previous_output: { txid: string; vout: number };
    script_sig: string;
    sequence: number;
    witness: string[];
  }>;
  output: Array<{
    value: bigint;
    script_pubkey: string;
  }>;
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
