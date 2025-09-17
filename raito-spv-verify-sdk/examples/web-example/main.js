import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify';

// DOM elements
const txidInput = document.getElementById('txid');
const verifyBtn = document.getElementById('verify-btn');
const statusDiv = document.getElementById('status');
const resultsDiv = document.getElementById('results');

// SDK instance
let sdk = null;

// Initialize the application
async function init() {
  console.log('🚀 Initializing Raito SPV SDK Demo');
  
  // Detect environment
  const isBrowser = typeof window !== 'undefined';
  const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;
  
  try {
    // Create SDK instance
    console.log('Creating SDK instance...');
    sdk = createRaitoSpvSdk();
    
    // Initialize SDK
    console.log('Initializing SDK...');
    statusDiv.textContent = 'Initializing SDK...';
    statusDiv.className = 'status loading';
    
    await sdk.init();
    
    statusDiv.textContent = 'SDK ready! Enter a transaction ID and click verify.';
    statusDiv.className = 'status success';
    
    console.log('✅ SDK initialized successfully');
    
  } catch (error) {
    console.error('❌ Failed to initialize SDK:', error);
    statusDiv.textContent = `Failed to initialize SDK: ${error.message}`;
    statusDiv.className = 'status error';
  }
}

// Verify transaction
async function verifyTransaction() {
  const txid = txidInput.value.trim();
  
  if (!txid) {
    statusDiv.textContent = 'Please enter a transaction ID';
    statusDiv.className = 'status error';
    return;
  }
  
  if (!sdk) {
    statusDiv.textContent = 'SDK not initialized';
    statusDiv.className = 'status error';
    return;
  }
  
  try {
    // Disable button and show loading
    verifyBtn.disabled = true;
    verifyBtn.textContent = 'Verifying...';
    statusDiv.textContent = 'Fetching proof...';
    statusDiv.className = 'status loading';
    resultsDiv.innerHTML = '';
    
    console.log('📡 Fetching proof for transaction:', txid);
    
    // Fetch the proof
    const startTime = Date.now();
    const proof = await sdk.fetchProof(txid);
    const fetchTime = Date.now() - startTime;
    
    statusDiv.textContent = 'Verifying proof...';
    
    // Verify the proof
    const verifyStartTime = Date.now();
    const result = await sdk.verifyProof(proof);
    const verifyTime = Date.now() - verifyStartTime;
    
    // Display results
    const totalTime = Date.now() - startTime;
    const proofData = JSON.parse(proof);
    const chainState = proofData.chain_state;
    
    resultsDiv.innerHTML = `
      <div class="result-item">
        <h3>✅ Verification Result</h3>
        <p><strong>Status:</strong> ${result ? 'Valid' : 'Invalid'}</p>
        <p><strong>Proof Block Height:</strong> ${chainState.block_height}</p>
        <p><strong>Proof Best Block Hash:</strong> <code>${chainState.best_block_hash}</code></p>
        <p><strong>Proof Length:</strong> ${proof.length} characters</p>
        <p><strong>Fetch Time:</strong> ${fetchTime}ms</p>
        <p><strong>Verify Time:</strong> ${verifyTime}ms</p>
        <p><strong>Total Time:</strong> ${totalTime}ms</p>
      </div>
    `;
    
    statusDiv.textContent = `Verification completed in ${totalTime}ms`;
    statusDiv.className = 'status success';
    
    console.log('✅ Verification completed:', result);
    
  } catch (error) {
    console.error('❌ Verification failed:', error);
    statusDiv.textContent = `Verification failed: ${error.message}`;
    statusDiv.className = 'status error';
    
    resultsDiv.innerHTML = `
      <div class="result-item error">
        <h3>❌ Error</h3>
        <p><strong>Message:</strong> ${error.message}</p>
        <p><strong>Transaction ID:</strong> <code>${txid}</code></p>
      </div>
    `;
  } finally {
    // Re-enable button
    verifyBtn.disabled = false;
    verifyBtn.textContent = 'Verify Transaction';
  }
}

// Event listeners
verifyBtn.addEventListener('click', verifyTransaction);
txidInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    verifyTransaction();
  }
});

// Initialize when DOM is loaded
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
