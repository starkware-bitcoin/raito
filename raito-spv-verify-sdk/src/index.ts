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
import { BlockHeader, Transaction } from './bitcoin.js';
import { BlockProofVerificationResult } from './block-proof.js';

// Type declarations for different environments
export class RaitoSpvSdk {
  private wasm: any;
  private raitoRpcUrl: string;
  private config: string;
  private chainStateFact: ChainStateProofVerificationResult | undefined;
  private blockHeaderFacts: Map<number, BlockHeader> = new Map();

  private transactionFacts: Map<string, Transaction> = new Map();

  constructor(
    raitoRpcUrl: string = 'https://api.raito.wtf',
    config: VerifierConfig
  ) {
    console.log('Initializing RaitoSpvSdk...');
    console.log(`RPC URL: ${raitoRpcUrl}`);
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

  async verifyRecentChainState(): Promise<ChainStateProofVerificationResult> {
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

    const { mmrRoot: chainStateMmrRoot, chainState } =
      await this.verifyRecentChainState();

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
  raitoRpcUrl?: string,
  config?: Partial<VerifierConfig>
): RaitoSpvSdk {
  return new RaitoSpvSdk(raitoRpcUrl, createVerifierConfig(config));
}
