/**
 * Block Proof Module
 * Handles fetching and verification of block inclusion proofs
 */

export interface BlockInclusionProof {
  peaks_hashes: string[];
  siblings_hashes: string[];
  leaf_index: number;
  leaf_count: number;
}

/**
 * Get the current MMR height from the Raito bridge RPC
 */
export async function getMmrHeight(raitoRpcUrl: string): Promise<number> {
  try {
    const url = `${raitoRpcUrl}/head`;
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
      },
    });
    if (!response.ok) {
      throw new Error(`Failed to fetch MMR height: ${response.status} ${response.statusText}`);
    }
    return await response.json() as number;
  } catch (error) {
    throw new Error(`Failed to fetch MMR height: ${error}`);
  }
}

/**
 * Fetch the block MMR inclusion proof from the Raito bridge RPC
 * 
 * @param raitoRpcUrl - The Raito RPC URL
 * @param blockHeight - Height of the block to prove
 * @param chainHeight - Current best height (chain head)
 * @param dev - Whether to use development mode (default: false)
 * @returns Promise<BlockInclusionProof> - The block inclusion proof
 */
export async function fetchBlockProof(
  raitoRpcUrl: string,
  blockHeight: number,
  chainHeight: number,
  dev: boolean = false
): Promise<BlockInclusionProof> {
  if (blockHeight > chainHeight) {
    throw new Error(
      `Block height ${blockHeight} cannot be greater than chain height ${chainHeight}`
    );
  }

  let url: string;
  if (dev) {
    console.log('DEV MODE: using local bridge node and default chain height');
    url = `http://127.0.0.1:5000/block-inclusion-proof/${blockHeight}`;
  } else {
    const mmrHeight = await getMmrHeight(raitoRpcUrl);
    if (mmrHeight < chainHeight) {
      throw new Error(
        `MMR height ${mmrHeight} is less than chain height ${chainHeight}`
      );
    }
    url = `${raitoRpcUrl}/block-inclusion-proof/${blockHeight}?chain_height=${chainHeight}`;
  }

  try {
    console.log(`Fetching block proof for block height ${blockHeight}...`);
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
      },
    });
    
    if (!response.ok) {
      throw new Error(`Failed to fetch block proof: ${response.status} ${response.statusText}`);
    }
    
    return await response.json() as BlockInclusionProof;
  } catch (error) {
    throw new Error(`Failed to fetch block proof: ${error}`);
  }
}
