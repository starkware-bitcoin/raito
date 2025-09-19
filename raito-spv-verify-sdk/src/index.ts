/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

import { fetchBlockProof, getMmrHeight, BlockInclusionProof } from './block-proof';
import * as chainStateProof from './chain-state-proof';
import * as compressedSpvProof from './compressed-spv-proof';
import { VerifierConfig } from './config';
import { importAndInit } from './wasm';

// Re-export types for external usage
export { BlockInclusionProof } from './block-proof';


// Environment detection
const isNode = typeof window === 'undefined' && typeof process !== 'undefined' && process.versions && process.versions.node;
const isBrowser = typeof window !== 'undefined';

// Type declarations for different environments

export class RaitoSpvSdk {
  private wasm: any;
  private raitoRpcUrl: string;

  constructor(raitoRpcUrl: string = 'https://api.raito.wtf') {
    this.raitoRpcUrl = raitoRpcUrl;
  }

  /**
   * Initialize the SDK with WASM module
   */
  async init(): Promise<void> {
    this.wasm = importAndInit()
  }

  /**
   * Fetch a complete compressed SPV proof for a transaction as a string
   */
  async fetchProof(txid: string): Promise<string> {
    return compressedSpvProof.fetchProof(this.raitoRpcUrl, txid);
  }


   /**
   * Fetch the most recent proven block height
   */
  async fetchRecentProvenHeight(): Promise<number> {
    try {
      return await chainStateProof.fetchRecentProvenHeight(this.raitoRpcUrl);
    } catch (error) {
      throw new Error(`Failed to fetch recent proven height: ${error}`);
    }
  }

  /**
   * Verify a compressed SPV proof
   */
  async verifyProof(
    proof: string,
    config?: Partial<VerifierConfig>
  ): Promise<boolean> {
    if (!this.wasm) {
      throw new Error('SDK not initialized. Call init() first.');
    }
    return compressedSpvProof.verifyProof(this.wasm, proof, config);
  }


  /**
   * Get the current MMR height from the Raito bridge RPC
   */
  async getMmrHeight(): Promise<number> {
    return getMmrHeight(this.raitoRpcUrl);
  }
}

/**
 * Create a new RaitoSpvSdk instance
 */
export function createRaitoSpvSdk(raitoRpcUrl?: string): RaitoSpvSdk {
  return new RaitoSpvSdk(raitoRpcUrl);
}
