import { createVerifierConfig, VerifierConfig } from "./config";

/**
 * Snapshot of the consensus chain state used to validate block inclusion
 */
export interface ChainState {
  /** The height of the best block in the chain */
  block_height: number;
  /** The total accumulated work of the chain as a decimal string */
  total_work: string;
  /** The hash of the best block in the chain */
  best_block_hash: string;
  /** The current target difficulty as a compact decimal string */
  current_target: string;
  /** The start time (UNIX seconds) of the current difficulty epoch */
  epoch_start_time: number;
  /** The timestamps (UNIX seconds) of the previous 11 blocks */
  prev_timestamps: number[];
}

/**
 * Fetch the most recent proven block height from the Raito API
 */
export async function fetchRecentProvenHeight(raitoRpcUrl: string): Promise<number> {
    const url = `${raitoRpcUrl}/chainstate-proof/recent_proven_height`;
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
      },
    });
    if (!response.ok) {
      throw new Error(`Failed to fetch recent proven height: ${response.status} ${response.statusText}`);
    }
    const data = await response.json() as { block_height: number };
    return data.block_height;
}

/**
 * Fetch the latest chain state proof from the Raito bridge RPC
 * @param raitoRpcUrl - URL of the Raito bridge RPC endpoint
 * @returns Promise that resolves to the chain state proof as a string
 */
export async function fetchProof(raitoRpcUrl: string): Promise<string> {
    const url = `${raitoRpcUrl}/chainstate-proof/recent_proof`;
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'text/plain',
      },
    });
    
    if (!response.ok) {
      throw new Error(`Failed to fetch chain state proof: ${response.status} ${response.statusText}`);
    }
    
    return await response.text();
}

/**
 * Verify the Cairo recursive proof and consistency of the bootloader output with chain state
 * @param wasmModule - The initialized WASM module
 * @param chainStateProof - The chain state proof data
 * @param config - The verifier configuration
 * @returns Promise that resolves to the MMR hash on success
 */
export async function verifyChainState(
  wasm: any,
  proof: string,
  config: string
): Promise<String> {
  // Use regexp to extract the "chainstate" object from the proof string and parse it as JSON
  const match = proof.match(/"chainstate"\s*:\s*({.*?})(,|\s*})/s);
  let chainstate = null;
  if (match && match[1]) {
    try {
      chainstate = JSON.parse(match[1]);
    } catch (e) {
      throw new Error("Failed to parse chainstate from proof: " + e);
    }
  }


  console.log('chainstate:', chainstate);
  console.log('wasm:', wasm);
  try {
    const result = wasm.verify_chain_state(proof, config);
    return result;
  } catch (e) {
    throw new Error("Failed to verify chain state: " + e);
  }
}


