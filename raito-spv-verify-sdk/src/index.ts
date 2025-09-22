/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

// import { fetchBlockProof, verifyBlockHeader } from './block-proof';
import * as chainStateProof from './chain-state-proof';
// import * as compressedSpvProof from './compressed-spv-proof';
// import { verifyTransaction } from './transaction-proof';
import { createVerifierConfig, VerifierConfig } from './config';
import { importAndInit } from './wasm';

// Re-export functions for external usage
// export { verifyTransaction } from './transaction-proof';

// Type declarations for different environments
export class RaitoSpvSdk {
  private wasm: any;
  private raitoRpcUrl: string;
  private config: string;

  constructor(raitoRpcUrl: string = 'https://api.raito.wtf', config: VerifierConfig) {
    this.raitoRpcUrl = raitoRpcUrl;
    this.config = JSON.stringify(config);
  }

  /**
   * Initialize the SDK with WASM module
   */
  async init(): Promise<void> {
    this.wasm = importAndInit()
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

  async verifyRecentChainState(): Promise<boolean> {
    const proof = await chainStateProof.fetchProof(this.raitoRpcUrl);
    const chainState = await chainStateProof.verifyChainState(this.wasm, proof, this.config);
    return chainState === 'true';
  }
}

/**
 * Create a new RaitoSpvSdk instance
 */
export function createRaitoSpvSdk(raitoRpcUrl?: string, config?: Partial<VerifierConfig>): RaitoSpvSdk {
  return new RaitoSpvSdk(raitoRpcUrl, createVerifierConfig(config));
}
