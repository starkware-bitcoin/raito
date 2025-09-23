/**
 * Simple example showing how to fetch and verify a specific transaction
 */

import { RaitoSpvSdk, createRaitoSpvSdk } from '../dist/index.js';

async function simpleExample() {
  console.log('🚀 Raito SPV TypeScript SDK - Simple Example');
  console.log('============================================\n');

  // Create SDK instance
  console.log('Creating SDK instance...');
  const sdk = createRaitoSpvSdk();
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

  // Fetch recent proven height
  try {
    console.log('📡 Fetching recent proven height...');
    const recentHeight = await sdk.fetchRecentProvenHeight();
    console.log(`✅ Most recent proven block height: ${recentHeight}\n`);
  } catch (error) {
    console.error('❌ Failed to fetch recent proven height:', error.message);
  }

  // Fetch and verify the specific transaction
  const txid =
    '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';

  try {
    console.log('📡 Fetching proof for transaction:', txid);

    // Fetch the proof as a string
    const proof = await sdk.fetchProof(txid);

    console.log(`📄 Proof as string length: ${proof.length} characters`);
    console.log(`📄 First 100 characters: ${proof.substring(0, 100)}...`);

    console.log('\n📡 Now attempting verification...');
    const result = await sdk.verifyProof(proof);

    console.log('✅ Verification result:', result);
  } catch (error) {
    console.error('❌ Error:', error.message);
    console.error('Stack trace:', error.stack);
  }

  console.log('\n🎉 Example completed!');
}

// Run the example
simpleExample().catch(console.error);
