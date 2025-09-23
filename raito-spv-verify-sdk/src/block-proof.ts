import { BlockHeader } from './bitcoin';

export interface BlockProofVerificationResult {
  mmrRoot: string;
  blockHeader: BlockHeader;
}

/**
 * Fetch a block header from the Raito bridge RPC
 *
 * @param raitoRpcUrl - The Raito RPC URL
 * @param blockHeight - Height of the block to fetch
 * @returns Promise<BlockHeader> - The block header as a JSON object
 */
export async function fetchBlockHeader(
  raitoRpcUrl: string,
  blockHeight: number
): Promise<BlockHeader> {
  const url = `${raitoRpcUrl}/block-header/${blockHeight}`;
  const response = await fetch(url, {
    method: 'GET',
    headers: {
      Accept: 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch block header: ${response.status} ${response.statusText}`
    );
  }

  return await response.json();
}

/**
 * Fetch the block MMR inclusion proof from the Raito bridge RPC
 *
 * @param raitoRpcUrl - The Raito RPC URL
 * @param blockHeight - Height of the block to prove
 * @param chainHeight - Current best height (chain head)
 * @returns Promise<String> - The block inclusion proof as a string
 */
export async function fetchBlockProof(
  raitoRpcUrl: string,
  blockHeight: number,
  chainHeight: number
): Promise<String> {
  if (blockHeight > chainHeight) {
    throw new Error(
      `Block height ${blockHeight} cannot be greater than chain height ${chainHeight}`
    );
  }

  let url = `${raitoRpcUrl}/block-inclusion-proof/${blockHeight}?chain_height=${chainHeight}`;

  console.log(`Fetching block proof for block height ${blockHeight}...`);
  const response = await fetch(url, {
    method: 'GET',
    headers: {
      Accept: 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch block proof: ${response.status} ${response.statusText}`
    );
  }

  return await response.text();
}

/**
 * Verify that a block header is included in the block MMR using an inclusion proof
 *
 * @param wasmModule - The initialized WASM module
 * @param blockHeader - The block header to verify as JSON string
 * @param blockHeaderProof - The block inclusion proof as JSON string
 * @returns Promise<string> - The computed block MMR root on success
 */
export async function verifyBlockHeader(
  wasm: any,
  blockHeader: string,
  blockHeaderProof: string
): Promise<string> {
  try {
    return await wasm.verify_block_header(blockHeader, blockHeaderProof);
  } catch (error) {
    throw new Error(`Block header verification failed: ${error}`);
  }
}
