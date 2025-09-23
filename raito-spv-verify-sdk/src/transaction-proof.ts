/**
 * Fetch a transaction inclusion proof from the Raito bridge RPC
 *
 * @param raitoRpcUrl - URL of the Raito bridge RPC endpoint
 * @param txId - Transaction ID to fetch proof for
 * @returns Promise<string> - The transaction inclusion proof as JSON string
 */
export async function fetchTransactionProof(
  raitoRpcUrl: string,
  txId: string
): Promise<string> {
  const url = `${raitoRpcUrl}/transaction-proof/${txId}`;

  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        Accept: 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.text();
  } catch (error) {
    throw new Error(`Failed to fetch transaction proof: ${error}`);
  }
}

/**
 * Verify a transaction inclusion proof using WASM
 *
 * @param wasm - The initialized WASM module
 * @param transactionProofData - The transaction inclusion proof as JSON string
 * @returns boolean - True if the transaction is verified to be included in the block
 */
export function verifyTransactionProof(
  wasm: any,
  transactionProofData: string
): boolean {
  if (!wasm) {
    throw new Error('WASM module not initialized');
  }

  try {
    return wasm.verify_transaction(transactionProofData);
  } catch (error) {
    throw new Error(`Transaction verification failed: ${error}`);
  }
}
