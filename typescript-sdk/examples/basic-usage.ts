/**
 * Basic usage example for Raito SPV TypeScript SDK
 */

import { RaitoSpvSdk, createRaitoSpvSdk } from '../src/index';

async function basicVerificationExample() {
  console.log('=== Basic Verification Example ===');
  
  // Create SDK instance
  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://localhost:8332',
    bitcoinRpcUserPwd: 'rpcuser:rpcpassword', // Replace with your credentials
    raitoRpcUrl: 'https://api.raito.wtf'
  });

  // Initialize the SDK
  await sdk.init();
  console.log('SDK initialized successfully');

  // Example proof data (replace with actual proof)
  const exampleProof = {
    chain_state: {
      block_height: 800000,
      total_work: "1813388729421943762059264",
      best_block_hash: "0000000000000000000000000000000000000000000000000000000000000000",
      current_target: "123456789",
      epoch_start_time: 1234567890,
      prev_timestamps: [1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890]
    },
    chain_state_proof: {}, // This would be actual Cairo proof data
    block_header: {
      version: 1,
      previousblockhash: "0000000000000000000000000000000000000000000000000000000000000000",
      merkleroot: "0000000000000000000000000000000000000000000000000000000000000000",
      time: 1234567890,
      bits: "1d00ffff",
      nonce: 123456,
      hash: "0000000000000000000000000000000000000000000000000000000000000000"
    },
    block_header_proof: {
      peaks_hashes: [],
      siblings_hashes: [],
      leaf_index: 0,
      leaf_count: 1
    },
    transaction: {
      version: 1,
      locktime: 0,
      vin: [],
      vout: []
    },
    transaction_proof: []
  };

  try {
    // Verify the proof
    const isValid = await sdk.verifyProof(exampleProof);
    console.log('Proof verification result:', isValid);
  } catch (error) {
    console.error('Verification failed (expected with example data):', error.message);
  }
}

async function fetchAndVerifyExample() {
  console.log('\n=== Fetch and Verify Example ===');
  
  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://localhost:8332',
    bitcoinRpcUserPwd: 'rpcuser:rpcpassword'
  });

  await sdk.init();

  // Example transaction ID (replace with actual transaction)
  const txid = 'a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456';

  try {
    // Fetch and verify in one operation
    const result = await sdk.fetchAndVerifyProof({
      txid,
      bitcoinRpcUrl: 'http://localhost:8332'
    });

    console.log('Fetch and verify result:');
    console.log('- Verified:', result.verified);
    console.log('- Block height:', result.proof.chain_state.block_height);
    console.log('- Total work:', result.proof.chain_state.total_work);
  } catch (error) {
    console.error('Fetch and verify failed:', error.message);
  }
}

async function customConfigExample() {
  console.log('\n=== Custom Configuration Example ===');
  
  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://localhost:8332'
  });

  await sdk.init();

  // Custom verification configuration
  const customConfig = {
    min_work: "1000000000000000000000000", // Custom minimum work
    task_output_size: 8
  };

  const exampleProof = {
    // ... proof data (same as above)
  } as any;

  try {
    const isValid = await sdk.verifyProof(exampleProof, customConfig);
    console.log('Custom config verification result:', isValid);
  } catch (error) {
    console.error('Custom config verification failed:', error.message);
  }
}

async function developmentModeExample() {
  console.log('\n=== Development Mode Example ===');
  
  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://localhost:8332'
  });

  await sdk.init();

  try {
    // Use development mode (connects to local bridge)
    const result = await sdk.fetchAndVerifyProof({
      txid: 'your-transaction-id',
      bitcoinRpcUrl: 'http://localhost:8332',
      dev: true // Uses local bridge at http://127.0.0.1:5000
    });

    console.log('Development mode result:', result.verified);
  } catch (error) {
    console.error('Development mode failed:', error.message);
  }
}

async function errorHandlingExample() {
  console.log('\n=== Error Handling Example ===');
  
  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://localhost:8332'
  });

  await sdk.init();

  try {
    // This will fail with invalid transaction ID
    await sdk.fetchProof({
      txid: 'invalid-txid',
      bitcoinRpcUrl: 'http://localhost:8332'
    });
  } catch (error) {
    console.log('Caught expected error:', error.constructor.name);
    console.log('Error message:', error.message);
    
    if (error.name === 'FetchError') {
      console.log('This is a fetch error - network or RPC issue');
    } else if (error.name === 'VerificationError') {
      console.log('This is a verification error - proof validation issue');
    } else if (error.name === 'RaitoError') {
      console.log('This is a general Raito SDK error');
    }
  }
}

// Run all examples
async function main() {
  console.log('Raito SPV TypeScript SDK - Examples');
  console.log('===================================');
  
  try {
    await basicVerificationExample();
    await fetchAndVerifyExample();
    await customConfigExample();
    await developmentModeExample();
    await errorHandlingExample();
    
    console.log('\n=== All examples completed ===');
  } catch (error) {
    console.error('Example failed:', error);
  }
}

// Run if this file is executed directly
if (require.main === module) {
  main().catch(console.error);
}

export {
  basicVerificationExample,
  fetchAndVerifyExample,
  customConfigExample,
  developmentModeExample,
  errorHandlingExample
};
