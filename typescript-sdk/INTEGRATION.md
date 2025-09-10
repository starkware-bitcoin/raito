# Integration Guide: Raito SPV TypeScript SDK

This guide shows how to integrate the Raito SPV TypeScript SDK with the WASM verification module.

## Prerequisites

1. **Built WASM Package**: The `raito-spv-verify-wasm` package must be built and available
2. **Node.js Environment**: Node.js 16+ or modern browser with WASM support
3. **RPC Endpoints**: Access to Bitcoin RPC and Raito bridge RPC endpoints

## Installation

### Option 1: Local Development

```bash
# In your project directory
npm install ../crates/raito-spv-verify-wasm/pkg
npm install ../typescript-sdk/dist
```

### Option 2: Published Packages (when available)

```bash
npm install @raito/spv-verify-wasm @raito/spv-sdk
```

## Basic Integration

### 1. Import and Initialize

```typescript
import { RaitoSpvSdk } from '@raito/spv-sdk';
import wasmModule from 'raito-spv-verify-wasm';

// Create SDK instance
const sdk = new RaitoSpvSdk({
  bitcoinRpcUrl: 'http://localhost:8332',
  bitcoinRpcUserPwd: 'rpcuser:rpcpassword',
  raitoRpcUrl: 'https://api.raito.wtf'
});

// Initialize with WASM module
await sdk.init();
```

### 2. Verify a Proof

```typescript
// Assuming you have a proof object
const proof = {
  chain_state: { /* ... */ },
  chain_state_proof: { /* ... */ },
  block_header: { /* ... */ },
  block_header_proof: { /* ... */ },
  transaction: { /* ... */ },
  transaction_proof: [/* ... */]
};

try {
  const isValid = await sdk.verifyProof(proof);
  console.log('Proof is valid:', isValid);
} catch (error) {
  console.error('Verification failed:', error.message);
}
```

### 3. Fetch and Verify

```typescript
try {
  const result = await sdk.fetchAndVerifyProof({
    txid: 'your-transaction-id',
    bitcoinRpcUrl: 'http://localhost:8332'
  });
  
  console.log('Verified:', result.verified);
  console.log('Block height:', result.proof.chain_state.block_height);
} catch (error) {
  console.error('Fetch and verify failed:', error.message);
}
```

## Advanced Usage

### Custom Verification Configuration

```typescript
const customConfig = {
  min_work: "2000000000000000000000000", // Custom minimum work
  bootloader_hash: "0x...", // Custom bootloader hash
  task_program_hash: "0x...", // Custom task program hash
  task_output_size: 16 // Custom output size
};

const isValid = await sdk.verifyProof(proof, customConfig);
```

### Development Mode

```typescript
// Use local Raito bridge for development
const result = await sdk.fetchAndVerifyProof({
  txid: 'your-txid',
  bitcoinRpcUrl: 'http://localhost:8332',
  dev: true // Uses http://127.0.0.1:5000
});
```

### Error Handling

```typescript
import { RaitoError, VerificationError, FetchError } from '@raito/spv-sdk';

try {
  const result = await sdk.fetchAndVerifyProof(options);
} catch (error) {
  if (error instanceof VerificationError) {
    console.error('Proof verification failed:', error.message);
  } else if (error instanceof FetchError) {
    console.error('Failed to fetch proof:', error.message);
  } else if (error instanceof RaitoError) {
    console.error('SDK error:', error.message);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Browser Integration

### HTML Setup

```html
<!DOCTYPE html>
<html>
<head>
    <title>Raito SPV SDK Example</title>
</head>
<body>
    <script src="node_modules/raito-spv-verify-wasm/raito_spv_verify_wasm.js"></script>
    <script src="node_modules/@raito/spv-sdk/dist/index.js"></script>
    <script>
        // Your application code here
    </script>
</body>
</html>
```

### ES6 Modules in Browser

```html
<script type="module">
import { RaitoSpvSdk } from './node_modules/@raito/spv-sdk/dist/index.js';
import wasmModule from './node_modules/raito-spv-verify-wasm/raito_spv_verify_wasm.js';

// Initialize SDK
const sdk = new RaitoSpvSdk({
  bitcoinRpcUrl: 'https://your-bitcoin-rpc.com'
});

await sdk.init();

// Use the SDK...
</script>
```

## React Integration

```typescript
import React, { useEffect, useState } from 'react';
import { RaitoSpvSdk } from '@raito/spv-sdk';

function VerificationComponent() {
  const [sdk, setSdk] = useState<RaitoSpvSdk | null>(null);
  const [verified, setVerified] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const initSdk = async () => {
      const sdkInstance = new RaitoSpvSdk({
        bitcoinRpcUrl: process.env.REACT_APP_BITCOIN_RPC_URL || 'http://localhost:8332'
      });
      await sdkInstance.init();
      setSdk(sdkInstance);
    };

    initSdk();
  }, []);

  const verifyTransaction = async (txid: string) => {
    if (!sdk) return;

    setLoading(true);
    try {
      const result = await sdk.fetchAndVerifyProof({ txid });
      setVerified(result.verified);
    } catch (error) {
      console.error('Verification failed:', error);
      setVerified(false);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <button 
        onClick={() => verifyTransaction('your-txid')}
        disabled={!sdk || loading}
      >
        {loading ? 'Verifying...' : 'Verify Transaction'}
      </button>
      {verified !== null && (
        <p>Verification result: {verified ? '✅ Valid' : '❌ Invalid'}</p>
      )}
    </div>
  );
}

export default VerificationComponent;
```

## Node.js Integration

```typescript
import { RaitoSpvSdk } from '@raito/spv-sdk';

async function main() {
  const sdk = new RaitoSpvSdk({
    bitcoinRpcUrl: process.env.BITCOIN_RPC_URL || 'http://localhost:8332',
    bitcoinRpcUserPwd: process.env.BITCOIN_RPC_USERPWD,
    raitoRpcUrl: process.env.RAITO_RPC_URL || 'https://api.raito.wtf'
  });

  await sdk.init();

  // Process command line arguments
  const txid = process.argv[2];
  if (!txid) {
    console.error('Usage: node verify.js <txid>');
    process.exit(1);
  }

  try {
    const result = await sdk.fetchAndVerifyProof({ txid });
    console.log(`Transaction ${txid} verification: ${result.verified ? 'VALID' : 'INVALID'}`);
  } catch (error) {
    console.error('Verification failed:', error.message);
    process.exit(1);
  }
}

main().catch(console.error);
```

## Testing

### Unit Tests

```typescript
import { RaitoSpvSdk } from '@raito/spv-sdk';

// Mock the WASM module
jest.mock('raito-spv-verify-wasm', () => ({
  init: jest.fn().mockResolvedValue(undefined),
  verify_proof_wasm: jest.fn().mockResolvedValue(true)
}));

describe('RaitoSpvSdk', () => {
  let sdk: RaitoSpvSdk;

  beforeEach(async () => {
    sdk = new RaitoSpvSdk({
      bitcoinRpcUrl: 'http://localhost:8332'
    });
    await sdk.init();
  });

  it('should verify proof successfully', async () => {
    const proof = { /* mock proof data */ };
    const result = await sdk.verifyProof(proof);
    expect(result).toBe(true);
  });
});
```

### Integration Tests

```typescript
// These tests require actual RPC endpoints
describe('Integration Tests', () => {
  const isIntegrationTest = process.env.INTEGRATION_TESTS === 'true';
  const test = isIntegrationTest ? it : it.skip;

  test('should fetch and verify real proof', async () => {
    const sdk = new RaitoSpvSdk({
      bitcoinRpcUrl: process.env.BITCOIN_RPC_URL!,
      raitoRpcUrl: process.env.RAITO_RPC_URL!
    });

    await sdk.init();

    const result = await sdk.fetchAndVerifyProof({
      txid: process.env.TEST_TXID!
    });

    expect(result.verified).toBeDefined();
  });
});
```

## Performance Considerations

1. **WASM Module Size**: The WASM module is ~13MB, consider lazy loading
2. **Memory Usage**: Proof verification can be memory intensive
3. **Network Calls**: Fetching proofs requires multiple RPC calls
4. **Caching**: Consider caching proofs and chain state data

## Troubleshooting

### Common Issues

1. **WASM Module Not Found**: Ensure the WASM package is properly installed
2. **RPC Connection Failed**: Check RPC endpoints and credentials
3. **Proof Verification Failed**: Verify proof data format and configuration
4. **Memory Issues**: Consider increasing Node.js memory limit with `--max-old-space-size`

### Debug Mode

```typescript
// Enable debug logging
process.env.DEBUG = 'raito:*';

const sdk = new RaitoSpvSdk({
  bitcoinRpcUrl: 'http://localhost:8332'
});
```

## Support

- GitHub Issues: https://github.com/raito-io/raito/issues
- Documentation: https://docs.raito.wtf
- Discord: https://discord.gg/raito
