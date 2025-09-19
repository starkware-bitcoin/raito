/**
 * Compressed SPV Proof Module
 * Handles fetching and verification of compressed SPV proofs
 */

export interface VerifierConfig {
  min_work: string;
  bootloader_hash: string;
  task_program_hash: string;
  task_output_size: number;
}

/**
 * Fetch a complete compressed SPV proof for a transaction as a string
 */
export async function fetchProof(raitoRpcUrl: string, txid: string): Promise<string> {
  try {
    const url = `${raitoRpcUrl}/compressed_spv_proof/${txid}`;
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'text/plain',
      },
    });
    if (!response.ok) {
      throw new Error(`Failed to fetch proof: ${response.status} ${response.statusText}`);
    }
    return await response.text() as string;
  } catch (error) {
    throw new Error(`Failed to fetch proof: ${error}`);
  }
}

/**
 * Verify a compressed SPV proof
 */
export async function verifyProof(
  wasmModule: any,
  proof: string,
  config?: Partial<VerifierConfig>
): Promise<boolean> {
  if (!wasmModule) {
    throw new Error('WASM module not provided.');
  }

  try {
    const verifierConfig = JSON.stringify(createVerifierConfig(config));
    return await wasmModule.verify_proof_with_config(proof, verifierConfig);
  } catch (error) {
    throw new Error(`Proof verification failed: ${error}`);
  }
}

/**
 * Create verifier configuration with defaults
 */
function createVerifierConfig(config?: Partial<VerifierConfig>): VerifierConfig {
  return {
    min_work: config?.min_work || '1813388729421943762059264',
    bootloader_hash: config?.bootloader_hash || '0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3',
    task_program_hash: config?.task_program_hash || '0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca',
    task_output_size: config?.task_output_size || 8,
  };
}
