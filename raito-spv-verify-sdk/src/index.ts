/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

import * as chainStateProof from './chain-state-proof.js';
import * as blockProof from './block-proof.js';
import * as transactionProof from './transaction-proof.js';
import { createVerifierConfig, VerifierConfig } from './config.js';
import { importAndInit } from './wasm.js';
import { BlockHeader, Transaction } from './bitcoin.js';
import { BlockProofVerificationResult } from './block-proof.js';

export * as chainStateProof from './chain-state-proof.js';
export * as blockProof from './block-proof.js';
export * as transactionProof from './transaction-proof.js';
export type { VerifierConfig } from './config.js';

/**
 * Configuration object for RaitoSpvSdk
 */
export interface RaitoSpvSdkConfig {
  raitoRpcUrl: string;
  verifierConfig: Partial<VerifierConfig>;
}

// Type declarations for different environments
export class RaitoSpvSdk {
  private wasm: any;
  private raitoRpcUrl: string;
  private config: string;
  private chainStateFact:
    | chainStateProof.ChainStateProofVerificationResult
    | undefined;
  private blockHeaderFacts: Map<number, BlockHeader> = new Map();

  private transactionFacts: Map<string, Transaction> = new Map();

  constructor(
    raitoRpcUrl: string = 'https://api.raito.wtf',
    config: VerifierConfig
  ) {
    console.log('Initializing RaitoSpvSdk...');
    console.log(`RPC URL: ${raitoRpcUrl}`);
    console.log(`Config: ${JSON.stringify(config)}`);
    this.raitoRpcUrl = raitoRpcUrl;
    this.config = JSON.stringify(config);
    console.log('RaitoSpvSdk initialized successfully');
  }

  async init(): Promise<void> {
    console.log('Initializing WASM module...');
    this.wasm = await importAndInit();
    console.log('WASM module initialized successfully');
  }

  async fetchRecentProvenHeight(): Promise<number> {
    console.log('Fetching recent proven block height...');
    try {
      const height = await chainStateProof.fetchRecentProvenHeight(
        this.raitoRpcUrl
      );
      console.log(`Recent proven height: ${height}`);
      return height;
    } catch (error) {
      console.error('Failed to fetch recent proven height:', error);
      throw new Error(`Failed to fetch recent proven height: ${error}`);
    }
  }

  async verifyRecentChainState(): Promise<chainStateProof.ChainStateProofVerificationResult> {
    if (this.chainStateFact) {
      console.log('Using cached chain state verification result');
      return this.chainStateFact;
    }

    console.log('Verifying recent chain state...');
    const proof = await chainStateProof.fetchProof(this.raitoRpcUrl);
    this.chainStateFact = await chainStateProof.verifyChainState(
      this.wasm,
      proof,
      this.config
    );
    console.log(
      `Chain state verified - Block height: ${
        this.chainStateFact.chainState.block_height
      }, MMR Root: ${this.chainStateFact.mmrRoot.substring(0, 16)}...`
    );
    return this.chainStateFact;
  }

  async verifyBlockHeader(
    blockHeight: number,
    blockHeader: BlockHeader | undefined = undefined
  ): Promise<BlockHeader> {
    if (this.blockHeaderFacts.has(blockHeight)) {
      console.log(
        `Using cached block header verification for height ${blockHeight}`
      );
      return this.blockHeaderFacts.get(blockHeight)!;
    }

    console.log(`Verifying block header for height ${blockHeight}...`);

    let { mmrRoot: chainStateMmrRoot, chainState } = await (async () => {
      let result = await this.verifyRecentChainState();
      if (blockHeight > result.chainState.block_height) {
        console.log(
          `Chain state is not up to date, trying to fetch latest chain state...`
        );
        this.chainStateFact = undefined;
        result = await this.verifyRecentChainState();
        if (blockHeight > result.chainState.block_height) {
          throw new Error(
            `Block height ${blockHeight} cannot be greater than latest proven chain height ${result.chainState.block_height}`
          );
        }
      }
      return result;
    })();

    const proof = await blockProof.fetchBlockProof(
      this.raitoRpcUrl,
      blockHeight,
      chainState.block_height
    );

    if (!blockHeader) {
      console.log(
        `Fetching block header from Raito bridge RPC for height ${blockHeight}...`
      );
      blockHeader = await blockProof.fetchBlockHeader(
        this.raitoRpcUrl,
        blockHeight
      );
    }

    const blockMmrRoot = await this.wasm.verify_block_header(
      JSON.stringify(blockHeader),
      proof
    );

    if (chainStateMmrRoot !== blockMmrRoot) {
      console.error(
        'Mismatched block MMR roots between chain state and block verification'
      );
      throw new Error('Mismatched block MMR roots');
    }

    console.log(`Verifying subchain work for block ${blockHeight}...`);
    const hasEnoughWork = this.wasm.verify_subchain_work(
      blockHeight,
      JSON.stringify(chainState),
      this.config
    );
    if (!hasEnoughWork) {
      throw new Error(
        `Not enough work on top of block ${blockHeight} according to verifier config`
      );
    }

    this.blockHeaderFacts.set(blockHeight, blockHeader);
    console.log(
      `Block header verified for height ${blockHeight} - MMR Root: ${blockMmrRoot.substring(
        0,
        16
      )}...`
    );
    return blockHeader;
  }

  async verifyTransaction(txid: string): Promise<Transaction> {
    if (this.transactionFacts.has(txid)) {
      console.log(
        `Using cached transaction verification for ${txid.substring(0, 16)}...`
      );
      return this.transactionFacts.get(txid)!;
    }

    console.log(`Verifying transaction ${txid.substring(0, 16)}...`);

    console.log(`Fetching transaction proof for ${txid.substring(0, 16)}...`);
    const transactionProofData = await transactionProof.fetchTransactionProof(
      this.raitoRpcUrl,
      txid
    );
    transactionProof.verifyTransactionProof(this.wasm, transactionProofData);

    const proof = JSON.parse(transactionProofData);
    const { block_header, block_height, transaction } = proof;

    console.log(
      `Transaction found in block ${block_height}, verifying block header...`
    );
    await this.verifyBlockHeader(block_height, block_header);

    this.transactionFacts.set(txid, transaction);
    console.log(
      `Transaction verified successfully: ${txid.substring(0, 16)}...`
    );

    return transaction;
  }
}

export function createRaitoSpvSdk(
  config?: Partial<RaitoSpvSdkConfig>
): RaitoSpvSdk {
  const raitoRpcUrl = config?.raitoRpcUrl || 'https://api.raito.wtf';
  const verifierConfig = createVerifierConfig(config?.verifierConfig);
  return new RaitoSpvSdk(raitoRpcUrl, verifierConfig);
}

// Singleton instance
let sdk: RaitoSpvSdk | undefined;
/**
 * Gets the singleton instance of RaitoSpvSdk
 * If no instance exists, creates one with default parameters
 * @param config Optional config for initialization
 * @returns The singleton RaitoSpvSdk instance
 */
export function getRaitoSpvSdk(
  config?: Partial<RaitoSpvSdkConfig>
): RaitoSpvSdk {
  if (!sdk) {
    sdk = createRaitoSpvSdk(config);
  }
  return sdk;
}

/**
 * Resets the singleton instance (useful for testing or reinitialization)
 */
export function resetRaitoSpvSdk(): void {
  sdk = undefined;
}
