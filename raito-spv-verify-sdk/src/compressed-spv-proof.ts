import { createVerifierConfig, VerifierConfig } from "./config";

/**
 * Fetch a complete compressed SPV proof for a transaction as a string
 */
export async function fetchProof(raitoRpcUrl: string, txid: string): Promise<string> {
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
}

/**
 * Verify a compressed SPV proof
 */
export async function verifyProof(
  wasmModule: any,
  proof: string,
  config?: Partial<VerifierConfig>
): Promise<boolean> {
    const verifierConfig = JSON.stringify(createVerifierConfig(config));
    return await wasmModule.verify_proof_with_config(proof, verifierConfig);
}
