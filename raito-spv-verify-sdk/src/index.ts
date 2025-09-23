/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

import * as chainStateProof from './chain-state-proof.js';
import * as blockProof from './block-proof.js';
import * as transactionProof from './transaction-proof.js';
import { ChainStateProofVerificationResult } from './chain-state-proof.js';
import { createVerifierConfig, VerifierConfig } from './config.js';
import { importAndInit } from './wasm.js';
import * as bitcoin from './bitcoin.js';
import { BitcoinCoreClient, BlockHeader } from './bitcoin.js';

// Re-export functions for external usage
// export { verifyTransaction } from './transaction-proof';

// Type declarations for different environments
export class RaitoSpvSdk {
  private wasm: any;
  private raitoRpcUrl: string;
  private bitcoin: BitcoinCoreClient;
  private config: string;
  private chainStateFact: ChainStateProofVerificationResult | undefined;
  private blockHeaderFacts: Map<number, BlockHeader> = new Map();

  constructor(raitoRpcUrl: string = 'https://api.raito.wtf', config: VerifierConfig, bitcoin: BitcoinCoreClient) {
    this.raitoRpcUrl = raitoRpcUrl;
    this.config = JSON.stringify(config);
    this.bitcoin = bitcoin;
  }

  /**
   * Initialize the SDK with WASM module
   */
  async init(): Promise<void> {
    this.wasm = await importAndInit()
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

  async verifyRecentChainState(): Promise<ChainStateProofVerificationResult> {
    const proof = await chainStateProof.fetchProof(this.raitoRpcUrl);
    this.chainStateFact = await chainStateProof.verifyChainState(this.wasm, proof, this.config);
    return this.chainStateFact;
  }

  async verifyBlockHeader(blockHeight: number): Promise<BlockHeader> {
    if (this.blockHeaderFacts.has(blockHeight)) {
      return this.blockHeaderFacts.get(blockHeight)!;
    }

    if (!this.chainStateFact) {
      await this.verifyRecentChainState();
    }

    const chainState = this.chainStateFact!.chainState;
    const chainHead = chainState.block_height;
    const proof = await blockProof.fetchBlockProof(this.raitoRpcUrl, blockHeight, chainHead);
    const { header } = await this.bitcoin.getBlockHeaderByHeight(blockHeight);

    await this.wasm.verify_block_header(JSON.stringify(header), proof);
    return header;    
  }

  async verifyTransaction(txid: string): Promise<bitcoin.Transaction> {
    // Fetch the transaction proof from the Raito bridge
    const transactionProofData = await transactionProof.fetchTransactionProof(this.raitoRpcUrl, txid);
    
    // Verify the transaction proof using WASM
    transactionProof.verifyTransactionProof(this.wasm, transactionProofData);
    
    // Parse the proof data to extract the transaction
    const proof = JSON.parse(transactionProofData);
    return proof.transaction;    
  }

}

/**
 * Create a new RaitoSpvSdk instance
 */
export function createRaitoSpvSdk(bitcoin: BitcoinCoreClient, raitoRpcUrl?: string, config?: Partial<VerifierConfig>): RaitoSpvSdk {
  return new RaitoSpvSdk(raitoRpcUrl, createVerifierConfig(config), bitcoin);
}
