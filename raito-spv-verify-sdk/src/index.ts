/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

import { fetchProof as fetchCompressedProof, verifyProof as verifyCompressedProof, VerifierConfig } from './compressed-spv-proof';

// Type definitions

export interface BlockInclusionProof {
  peaks_hashes: string[];
  siblings_hashes: string[];
  leaf_index: number;
  leaf_count: number;
}

// Environment detection
const isNode = typeof window === 'undefined' && typeof process !== 'undefined' && process.versions && process.versions.node;
const isBrowser = typeof window !== 'undefined';

// Type declarations for different environments

export class RaitoSpvSdk {
  private wasmModule: any;
  private raitoRpcUrl: string;

  constructor(raitoRpcUrl: string = 'https://api.raito.wtf') {
    this.raitoRpcUrl = raitoRpcUrl;
  }

  /**
   * Initialize the SDK with WASM module
   */
  async init(): Promise<void> {
    try {
      // Load WASM module based on environment
      if (isNode) {
        // Node.js environment - use dynamic import for ES modules
        this.wasmModule = await import('../dist/node/index.js');
      } else if (isBrowser) {
        // Browser environment - use web version for direct browser usage
        this.wasmModule = await import('../dist/web/index.js');
        const start = this.wasmModule.default ?? this.wasmModule.__wbg_init;
        if (typeof start !== 'function') {
          throw new Error('WASM initializer not found on module');
        }
        await start();  
      } else {
        throw new Error('Unsupported environment: neither Node.js nor browser detected');
      }
      await this.wasmModule.init();
    } catch (error) {
      throw new Error(`Failed to initialize WASM module: ${error}`);
    }
  }

  /**
   * Fetch a complete compressed SPV proof for a transaction as a string
   */
  async fetchProof(txid: string): Promise<string> {
    return fetchCompressedProof(this.raitoRpcUrl,txid);
  }

  /**
   * Fetch the most recent proven block height
   */
  async fetchRecentProvenHeight(): Promise<number> {
    try {
      const url = `${this.raitoRpcUrl}/chainstate-proof/recent_proven_height`;
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
    if (!this.wasmModule) {
      throw new Error('SDK not initialized. Call init() first.');
    }
    return verifyCompressedProof(this.wasmModule, proof, this.raitoRpcUrl, config);
  }


  /**
   * Get the current MMR height from the Raito bridge RPC
   */
  async getMmrHeight(): Promise<number> {
    try {
      const url = `${this.raitoRpcUrl}/head`;
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
   * @param blockHeight - Height of the block to prove
   * @param chainHeight - Current best height (chain head)
   * @param dev - Whether to use development mode (default: false)
   * @returns Promise<BlockInclusionProof> - The block inclusion proof
   */
  async fetchBlockProof(
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
      const mmrHeight = await this.getMmrHeight();
      if (mmrHeight < chainHeight) {
        throw new Error(
          `MMR height ${mmrHeight} is less than chain height ${chainHeight}`
        );
      }
      url = `${this.raitoRpcUrl}/block-inclusion-proof/${blockHeight}?chain_height=${chainHeight}`;
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
}

/**
 * Create a new RaitoSpvSdk instance
 */
export function createRaitoSpvSdk(raitoRpcUrl?: string): RaitoSpvSdk {
  return new RaitoSpvSdk(raitoRpcUrl);
}
