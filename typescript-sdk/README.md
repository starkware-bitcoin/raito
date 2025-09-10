# Raito SPV TypeScript SDK

A comprehensive TypeScript SDK for Raito SPV (Simplified Payment Verification) proof verification and fetching. This SDK provides both client-side verification capabilities using WebAssembly and server-side proof fetching from Bitcoin and Raito nodes.

## Features

- 🔍 **SPV Proof Verification**: Verify compressed SPV proofs using WebAssembly
- 📡 **Proof Fetching**: Fetch complete SPV proofs from Bitcoin and Raito nodes
- 🛡️ **Type Safety**: Full TypeScript support with comprehensive type definitions
- ⚡ **Async/Await**: Modern async API design
- 🔧 **Configurable**: Customizable verification parameters and RPC endpoints
- 📦 **Lightweight**: Minimal dependencies, optimized for both Node.js and browsers

## Installation

```bash
npm install @raito/spv-sdk
```

## Quick Start

### Basic Verification

```typescript
import { RaitoSpvSdk } from '@raito/spv-sdk';

// Initialize the SDK
const sdk = new RaitoSpvSdk({
  bitcoinRpcUrl: 'http://localhost:8332',
  bitcoinRpcUserPwd: 'user:password', // optional
  raitoRpcUrl: 'https://api.raito.wtf' // optional, defaults to production
});

// Initialize WASM module
await sdk.init();

// Verify a proof
const proof = { /* your SPV proof data */ };
const isValid = await sdk.verifyProof(proof);
console.log('Proof is valid:', isValid);
```

### Fetching and Verifying Proofs

```typescript
// Fetch a complete proof for a transaction
const proof = await sdk.fetchProof({
  txid: 'your-transaction-id',
  bitcoinRpcUrl: 'http://localhost:8332',
  bitcoinRpcUserPwd: 'user:password'
});

// Verify the fetched proof
const isValid = await sdk.verifyProof(proof);
console.log('Fetched and verified proof:', isValid);
```

### One-Step Fetch and Verify

```typescript
// Fetch and verify in one operation
const result = await sdk.fetchAndVerifyProof({
  txid: 'your-transaction-id',
  bitcoinRpcUrl: 'http://localhost:8332'
});

console.log('Proof valid:', result.verified);
console.log('Proof data:', result.proof);
```

## API Reference

### RaitoSpvSdk

The main SDK class that provides all functionality.

#### Constructor

```typescript
new RaitoSpvSdk(options: {
  raitoRpcUrl?: string;        // Raito bridge RPC URL (default: https://api.raito.wtf)
  bitcoinRpcUrl: string;       // Bitcoin RPC URL (required)
  bitcoinRpcUserPwd?: string;  // Bitcoin RPC credentials in format "user:password"
})
```

#### Methods

##### `init(): Promise<void>`

Initialize the SDK with the WebAssembly module. Must be called before using verification methods.

##### `verifyProof(proof: CompressedSpvProof, config?: Partial<VerifierConfig>, dev?: boolean): Promise<boolean>`

Verify a compressed SPV proof.

- `proof`: The SPV proof to verify
- `config`: Optional verification configuration (uses defaults if not provided)
- `dev`: Development mode flag (default: false)
- Returns: `Promise<boolean>` - true if proof is valid

##### `fetchProof(options: FetchProofOptions): Promise<CompressedSpvProof>`

Fetch a complete SPV proof for a transaction.

- `options`: Fetch configuration including transaction ID and RPC endpoints
- Returns: `Promise<CompressedSpvProof>` - The complete proof

##### `fetchAndVerifyProof(options: FetchProofOptions, config?: Partial<VerifierConfig>): Promise<{proof: CompressedSpvProof, verified: boolean}>`

Fetch and verify a proof in one operation.

- `options`: Fetch configuration
- `config`: Optional verification configuration
- Returns: Promise with both proof data and verification result

### Types

#### CompressedSpvProof

```typescript
interface CompressedSpvProof {
  chain_state: ChainState;
  chain_state_proof: any; // Cairo proof
  block_header: BitcoinBlockHeader;
  block_header_proof: BlockInclusionProof;
  transaction: BitcoinTransaction;
  transaction_proof: number[];
}
```

#### VerifierConfig

```typescript
interface VerifierConfig {
  min_work: string;                    // Minimum cumulative work required
  bootloader_hash: string;             // Expected bootloader program hash
  task_program_hash: string;           // Expected payload program hash
  task_output_size: number;            // Expected payload program output size
}
```

#### FetchProofOptions

```typescript
interface FetchProofOptions {
  txid: string;                        // Transaction ID to fetch
  raitoRpcUrl?: string;                // Raito RPC URL (optional)
  bitcoinRpcUrl: string;               // Bitcoin RPC URL
  bitcoinRpcUserPwd?: string;          // Bitcoin RPC credentials
  dev?: boolean;                       // Development mode
}
```

## Configuration

### Default Verifier Configuration

The SDK uses sensible defaults for verification:

```typescript
const defaultConfig: VerifierConfig = {
  min_work: "1813388729421943762059264", // 6 * 2^78 (six block confirmations)
  bootloader_hash: "0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3",
  task_program_hash: "0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca",
  task_output_size: 8
};
```

### Custom Configuration

You can override any of these values:

```typescript
const customConfig = {
  min_work: "your-custom-min-work",
  bootloader_hash: "your-custom-bootloader-hash"
};

const isValid = await sdk.verifyProof(proof, customConfig);
```

## Error Handling

The SDK provides specific error types for different failure scenarios:

```typescript
import { RaitoError, VerificationError, FetchError } from '@raito/spv-sdk';

try {
  const proof = await sdk.fetchProof(options);
  const isValid = await sdk.verifyProof(proof);
} catch (error) {
  if (error instanceof VerificationError) {
    console.error('Proof verification failed:', error.message);
  } else if (error instanceof FetchError) {
    console.error('Failed to fetch proof:', error.message);
  } else if (error instanceof RaitoError) {
    console.error('Raito SDK error:', error.message);
  }
}
```

## Development Mode

For development and testing, you can use the development mode which connects to a local Raito bridge node:

```typescript
const proof = await sdk.fetchProof({
  txid: 'your-txid',
  bitcoinRpcUrl: 'http://localhost:8332',
  dev: true // Uses local bridge at http://127.0.0.1:5000
});
```

## Browser Usage

The SDK works in both Node.js and browser environments. For browser usage, make sure to include the WASM module:

```html
<script src="path/to/raito-spv-verify-wasm.js"></script>
<script src="path/to/raito-spv-sdk.js"></script>
```

## Examples

### Node.js Example

```typescript
import { RaitoSpvSdk } from '@raito/spv-sdk';

async function main() {
  const sdk = new RaitoSpvSdk({
    bitcoinRpcUrl: 'http://localhost:8332',
    bitcoinRpcUserPwd: 'rpcuser:rpcpassword'
  });

  await sdk.init();

  try {
    const result = await sdk.fetchAndVerifyProof({
      txid: 'a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456',
      bitcoinRpcUrl: 'http://localhost:8332'
    });

    console.log('Transaction verified:', result.verified);
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main();
```

### React Example

```typescript
import React, { useEffect, useState } from 'react';
import { RaitoSpvSdk } from '@raito/spv-sdk';

function VerificationComponent() {
  const [sdk, setSdk] = useState<RaitoSpvSdk | null>(null);
  const [verified, setVerified] = useState<boolean | null>(null);

  useEffect(() => {
    const initSdk = async () => {
      const sdkInstance = new RaitoSpvSdk({
        bitcoinRpcUrl: 'https://your-bitcoin-rpc.com'
      });
      await sdkInstance.init();
      setSdk(sdkInstance);
    };

    initSdk();
  }, []);

  const verifyTransaction = async (txid: string) => {
    if (!sdk) return;

    try {
      const result = await sdk.fetchAndVerifyProof({ txid });
      setVerified(result.verified);
    } catch (error) {
      console.error('Verification failed:', error);
    }
  };

  return (
    <div>
      <button onClick={() => verifyTransaction('your-txid')}>
        Verify Transaction
      </button>
      {verified !== null && (
        <p>Verification result: {verified ? 'Valid' : 'Invalid'}</p>
      )}
    </div>
  );
}
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/raito-io/raito.git
cd raito/typescript-sdk

# Install dependencies
npm install

# Build the SDK
npm run build

# Run tests
npm test
```

## License

MIT License - see LICENSE file for details.

## Support

For support and questions:
- GitHub Issues: https://github.com/raito-io/raito/issues
- Documentation: https://docs.raito.wtf
- Discord: https://discord.gg/raito
