import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify';

// DOM elements
const txidInput = document.getElementById('txid');
const blockHeightInput = document.getElementById('block-height');
const verifyChainstateBtn = document.getElementById('verify-chainstate-btn');
const verifyBlockBtn = document.getElementById('verify-block-btn');
const verifyTransactionBtn = document.getElementById('verify-transaction-btn');
const statusDiv = document.getElementById('status');
const resultsDiv = document.getElementById('results');

// SDK instance
let sdk = null;

// Initialize the application
async function init() {
  console.log('🚀 Initializing Raito SPV SDK Demo');

  // Detect environment
  const isBrowser = typeof window !== 'undefined';
  const isNode =
    typeof process !== 'undefined' && process.versions && process.versions.node;

  try {
    // Create SDK instance
    console.log('Creating SDK instance...');
    sdk = createRaitoSpvSdk();

    // Initialize SDK
    console.log('Initializing SDK...');
    statusDiv.textContent = 'Initializing SDK...';
    statusDiv.className = 'status loading';

    await sdk.init();

    statusDiv.textContent =
      'SDK ready! Try verifying chain state, block headers, or transactions.';
    statusDiv.className = 'status success';

    console.log('✅ SDK initialized successfully');
  } catch (error) {
    console.error('❌ Failed to initialize SDK:', error);
    statusDiv.textContent = `Failed to initialize SDK: ${error.message}`;
    statusDiv.className = 'status error';
  }
}

// Verify recent chain state
async function verifyChainState() {
  if (!sdk) {
    statusDiv.textContent = 'SDK not initialized';
    statusDiv.className = 'status error';
    return;
  }

  try {
    // Disable button and show loading
    verifyChainstateBtn.disabled = true;
    verifyChainstateBtn.textContent = 'Verifying...';
    statusDiv.textContent = 'Verifying recent chain state...';
    statusDiv.className = 'status loading';
    resultsDiv.innerHTML = '';

    console.log('🔎 Verifying recent chain state...');

    const startTime = Date.now();
    const result = await sdk.verifyRecentChainState();
    const totalTime = Date.now() - startTime;

    // Display results
    resultsDiv.innerHTML = `
      <div class="result-item">
        <h3>✅ Chain State Verification Result</h3>
        <p><strong>Status:</strong> Valid</p>
        <p><strong>Block Height:</strong> ${result.chainState.block_height}</p>
        <p><strong>Best Block Hash:</strong> <code>${result.chainState.best_block_hash}</code></p>
        <p><strong>MMR Root:</strong> <code>${result.mmrRoot.substring(0, 32)}...</code></p>
        <p><strong>Verification Time:</strong> ${totalTime}ms</p>
      </div>
    `;

    statusDiv.textContent = `Chain state verification completed in ${totalTime}ms`;
    statusDiv.className = 'status success';

    console.log('✅ Chain state verification completed:', result);
  } catch (error) {
    console.error('❌ Chain state verification failed:', error);
    statusDiv.textContent = `Chain state verification failed: ${error.message}`;
    statusDiv.className = 'status error';

    resultsDiv.innerHTML = `
      <div class="result-item error">
        <h3>❌ Error</h3>
        <p><strong>Message:</strong> ${error.message}</p>
      </div>
    `;
  } finally {
    // Re-enable button
    verifyChainstateBtn.disabled = false;
    verifyChainstateBtn.textContent = 'Verify Recent Chain State';
  }
}

// Verify block header
async function verifyBlock() {
  const blockHeight = parseInt(blockHeightInput.value.trim());

  if (!blockHeight || blockHeight < 0) {
    statusDiv.textContent = 'Please enter a valid block height';
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
    verifyBlockBtn.disabled = true;
    verifyBlockBtn.textContent = 'Verifying...';
    statusDiv.textContent = `Verifying block header at height ${blockHeight}...`;
    statusDiv.className = 'status loading';
    resultsDiv.innerHTML = '';

    console.log(`🔎 Verifying block header at height ${blockHeight}...`);

    const startTime = Date.now();
    const blockHeader = await sdk.verifyBlockHeader(blockHeight);
    const totalTime = Date.now() - startTime;

    // Display results
    resultsDiv.innerHTML = `
      <div class="result-item">
        <h3>✅ Block Header Verification Result</h3>
        <p><strong>Status:</strong> Valid</p>
        <p><strong>Block Height:</strong> ${blockHeight}</p>
        <p><strong>Previous Block Hash:</strong> <code>${blockHeader.prev_blockhash}</code></p>
        <p><strong>Merkle Root:</strong> <code>${blockHeader.merkle_root}</code></p>
        <p><strong>Timestamp:</strong> ${new Date(blockHeader.time * 1000).toISOString()}</p>
        <p><strong>Verification Time:</strong> ${totalTime}ms</p>
      </div>
    `;

    statusDiv.textContent = `Block header verification completed in ${totalTime}ms`;
    statusDiv.className = 'status success';

    console.log('✅ Block header verification completed:', blockHeader);
    console.log('Block header properties:', Object.keys(blockHeader));
  } catch (error) {
    console.error('❌ Block header verification failed:', error);
    statusDiv.textContent = `Block header verification failed: ${error.message}`;
    statusDiv.className = 'status error';

    resultsDiv.innerHTML = `
      <div class="result-item error">
        <h3>❌ Error</h3>
        <p><strong>Message:</strong> ${error.message}</p>
        <p><strong>Block Height:</strong> ${blockHeight}</p>
      </div>
    `;
  } finally {
    // Re-enable button
    verifyBlockBtn.disabled = false;
    verifyBlockBtn.textContent = 'Verify Block Header';
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
    verifyTransactionBtn.disabled = true;
    verifyTransactionBtn.textContent = 'Verifying...';
    statusDiv.textContent = `Verifying transaction ${txid.substring(0, 16)}...`;
    statusDiv.className = 'status loading';
    resultsDiv.innerHTML = '';

    console.log(`🔎 Verifying transaction ${txid.substring(0, 16)}...`);

    const startTime = Date.now();
    const transaction = await sdk.verifyTransaction(txid);
    const totalTime = Date.now() - startTime;

    // Display results
    resultsDiv.innerHTML = `
      <div class="result-item">
        <h3>✅ Transaction Verification Result</h3>
        <p><strong>Status:</strong> Valid</p>
        <p><strong>Transaction ID:</strong> <code>${txid}</code></p>
        <p><strong>Version:</strong> ${transaction.version}</p>
        <p><strong>Lock Time:</strong> ${transaction.lock_time}</p>
        <p><strong>Input Count:</strong> ${transaction.input.length}</p>
        <p><strong>Output Count:</strong> ${transaction.output.length}</p>
        <p><strong>Verification Time:</strong> ${totalTime}ms</p>
      </div>
    `;

    statusDiv.textContent = `Transaction verification completed in ${totalTime}ms`;
    statusDiv.className = 'status success';

    console.log('✅ Transaction verification completed:', transaction);
  } catch (error) {
    console.error('❌ Transaction verification failed:', error);
    statusDiv.textContent = `Transaction verification failed: ${error.message}`;
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
    verifyTransactionBtn.disabled = false;
    verifyTransactionBtn.textContent = 'Verify Transaction';
  }
}

// Event listeners
verifyChainstateBtn.addEventListener('click', verifyChainState);
verifyBlockBtn.addEventListener('click', verifyBlock);
verifyTransactionBtn.addEventListener('click', verifyTransaction);

blockHeightInput.addEventListener('keypress', e => {
  if (e.key === 'Enter') {
    verifyBlock();
  }
});

txidInput.addEventListener('keypress', e => {
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
