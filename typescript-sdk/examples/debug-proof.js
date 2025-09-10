/**
 * Debug example to see the full proof structure
 */

const { RaitoSpvSdk, createRaitoSpvSdk } = require('../dist/index.js');

async function debugProof() {
  console.log('🔍 Debugging proof structure...\n');

  const sdk = createRaitoSpvSdk({
    bitcoinRpcUrl: 'http://34.136.127.253:8332/',
    bitcoinRpcUserPwd: 'raito:r@it00ti@r',
    raitoRpcUrl: 'https://api.raito.wtf'
  });

  await sdk.init();

  const txid = '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc';
  
  try {
    const proof = await sdk.fetchProof({
      txid,
      bitcoinRpcUrl: 'http://34.136.127.253:8332/',
      bitcoinRpcUserPwd: 'raito:r@it00ti@r'
    });
    
    console.log('📊 Full proof structure:');
    console.log(JSON.stringify(proof, null, 2));
  } catch (error) {
    console.error('❌ Error:', error.message);
  }
}

debugProof().catch(console.error);
