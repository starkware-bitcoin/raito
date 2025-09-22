/**
 * Verify that a transaction is included in a block header using a Merkle proof
 * 
 * @param wasmModule - The initialized WASM module
 * @param transaction - The transaction to verify as JSON string
 * @param blockHeader - The block header containing the transaction as JSON string
 * @param transactionProof - The transaction Merkle proof as hex string
 * @returns Promise<boolean> - True if the transaction is verified to be included in the block
 */
export async function verifyTransaction(
  wasm: any,
  transaction: string,
  blockHeader: string,
  transactionProof: string
): Promise<boolean> {
  if (!wasm) {
    throw new Error('WASM module not initialized');
  }

  try {
    const proofBytes = hexStringToUint8Array(transactionProof);
    
    return await wasm.verify_transaction(transaction, blockHeader, proofBytes);
  } catch (error) {
    throw new Error(`Transaction verification failed: ${error}`);
  }
}

/**
 * Convert a hex string to Uint8Array
 * @param hexString - The hex string to convert
 * @returns Uint8Array representation of the hex string
 */
function hexStringToUint8Array(hexString: string): Uint8Array {
  // Remove any whitespace and '0x' prefix if present
  const cleanHex = hexString.replace(/[\s]/g, '').replace(/^0x/, '');
  
  // Ensure the string has even length
  if (cleanHex.length % 2 !== 0) {
    throw new Error('Hex string must have even length');
  }
  
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
  }
  
  return bytes;
}
