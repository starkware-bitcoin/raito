import { getRaitoSpvSdk } from '../dist/index.js';

async function simpleExample() {
  console.log('🚀 Raito SPV TypeScript SDK - Node Example');
  console.log('============================================\n');

  // Create SDK instance with default config and default RPC URL
  console.log('Creating SDK instance...');
  const sdk = getRaitoSpvSdk({
    verifierConfig: { min_work: '0' }
  });
  console.log('✅ SDK instance created\n');

  // Initialize SDK (loads WASM)
  console.log('Initializing SDK...');

  await sdk.init();
  console.log('✅ SDK initialized successfully\n');

  console.log('📡 Fetching recent proven height...');
  const recentHeight = await sdk.fetchRecentProvenHeight();
  console.log(`✅ Most recent proven block height: ${recentHeight}\n`);

  console.log('🔎 Verifying most recent chain state proof...');
  const result = await sdk.verifyRecentChainState();
  console.log('✅ Chain state proof verification completed\n');

  console.log('🔎 Verifying most recent proven block header...');
  await sdk.verifyBlockHeader(recentHeight);
  console.log(`✅ Block header at height ${recentHeight} verified\n`);

  const txid =
    '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';
  console.log(`🔎 Verifying transaction inclusion for txid: ${txid}...`);
  await sdk.verifyTransaction(txid);
  console.log('✅ Transaction verified and fetched\n');

  const txid2 =
    'f20adf4cb519484e2763c38d901bc971336f22639fbec73e127b822711669bde';
  console.log(`🔎 Verifying transaction inclusion for txid: ${txid2}...`);
  await sdk.verifyTransaction(txid2);
  console.log('✅ Transaction verified and fetched\n');

  console.log('🎉 Dev example completed!');
}

// Run the example
simpleExample().catch(console.error);
