/**
 * Simple example showing how to fetch and verify a specific transaction
 */

const { RaitoSpvSdk, createRaitoSpvSdk } = require('../dist/index.js');

async function simpleExample() {
  console.log('🚀 Raito SPV TypeScript SDK - Simple Example');
  console.log('============================================\n');

  // Create SDK instance
  console.log('Creating SDK instance...');
  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://34.136.127.253:8332/',
    bitcoinRpcUserPwd: 'raito:r@it00ti@r',
    raitoRpcUrl: 'https://api.raito.wtf'
  });
  console.log('✅ SDK instance created\n');

  // Initialize SDK
  console.log('Initializing SDK...');
  try {
    console.log('📦 Loading WASM module...');
    await sdk.init();
    console.log('✅ SDK initialized successfully\n');
  } catch (error) {
    console.error('❌ Failed to initialize SDK:', error.message);
    return;
  }

  // Fetch and verify the specific transaction
  const txid = '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';
  
  try {
    console.log('📡 Fetching proof for transaction:', txid);
    
    // First, let's just fetch the proof without verifying to see the data structure
    const proof = await sdk.fetchProof({
      txid,
      bitcoinRpcUrl: 'http://34.136.127.253:8332/',
      bitcoinRpcUserPwd: 'raito:r@it00ti@r'
    });
    
    console.log('📊 Proof fetched successfully!');
    console.log('🔍 Chain state data:');
    console.log('  - block_height:', proof.chain_state.block_height, typeof proof.chain_state.block_height);
    console.log('  - total_work:', proof.chain_state.total_work, typeof proof.chain_state.total_work);
    console.log('  - best_block_hash:', proof.chain_state.best_block_hash, typeof proof.chain_state.best_block_hash);
    console.log('  - current_target:', proof.chain_state.current_target, typeof proof.chain_state.current_target);
    console.log('  - epoch_start_time:', proof.chain_state.epoch_start_time, typeof proof.chain_state.epoch_start_time);
    console.log('  - prev_timestamps:', proof.chain_state.prev_timestamps.slice(0, 3), '... (first 3)');
    
    console.log('\n🔍 Block header data:');
    console.log('  - version:', proof.block_header.version, typeof proof.block_header.version);
    console.log('  - time:', proof.block_header.time, typeof proof.block_header.time);
    console.log('  - bits:', proof.block_header.bits, typeof proof.block_header.bits);
    console.log('  - nonce:', proof.block_header.nonce, typeof proof.block_header.nonce);
    
    console.log('\n🔍 Transaction data:');
    console.log('  - version:', proof.transaction.version, typeof proof.transaction.version);
    console.log('  - locktime:', proof.transaction.locktime, typeof proof.transaction.locktime);
    
    // Fix the bits field by converting hex string to number
    console.log('\n🔧 Converting bits field from hex string to number...');
    const fixedProof = {
      ...proof,
      block_header: proof.block_header
    };
    
    console.log('  - bits (fixed):', fixedProof.block_header.bits, typeof fixedProof.block_header.bits);
    
    console.log('\n📡 Now attempting verification with fixed proof...');

    console.log("📋 fixed proof:");
    console.log(JSON.stringify(fixedProof.block_header, null, 2));
    console.log(JSON.stringify(fixedProof.chain_state, null, 2));
    console.log(JSON.stringify(fixedProof.transaction, null, 2));

    const result = await sdk.verifyProof(fixedProof);
    
    console.log('✅ Verification result:', result);
  } catch (error) {
    console.error('❌ Error:', error.message);
    console.error('Stack trace:', error.stack);
  }

  console.log('\n🎉 Example completed!');
}

// Run the example
simpleExample().catch(console.error);
