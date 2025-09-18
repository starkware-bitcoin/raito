/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

// Type definitions
export interface VerifierConfig {
  min_work: string;
  bootloader_hash: string;
  task_program_hash: string;
  task_output_size: number;
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
    // Fetch the compressed SPV proof for the given transaction ID as a string.
    // This makes a GET request to the Raito RPC endpoint and returns the proof as a plain string.
    try {
      const url = `${this.raitoRpcUrl}/compressed_spv_proof/${txid}`;
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

    try {
      const verifierConfig = JSON.stringify(this.createVerifierConfig(config));
      const result = await this.wasmModule.verify_proof_with_config(proof, verifierConfig);
      return result;
    } catch (error) {
      throw new Error(`Proof verification failed: ${error}`);
    }
  }

  /**
   * Create verifier configuration with defaults
   */
  private createVerifierConfig(config?: Partial<VerifierConfig>): VerifierConfig {
    return {
      min_work: config?.min_work || '1813388729421943762059264',
      bootloader_hash: config?.bootloader_hash || '0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3',
      task_program_hash: config?.task_program_hash || '0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca',
      task_output_size: config?.task_output_size || 8,
    };
  }
}

/**
 * Create a new RaitoSpvSdk instance
 */
export function createRaitoSpvSdk(raitoRpcUrl?: string): RaitoSpvSdk {
  return new RaitoSpvSdk(raitoRpcUrl);
}
