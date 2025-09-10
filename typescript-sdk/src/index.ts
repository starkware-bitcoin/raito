/**
 * Raito SPV TypeScript SDK
 * Provides verification and fetching capabilities for SPV proofs
 */

import axios, { AxiosInstance } from 'axios';
import * as bitcoin from 'bitcoinjs-lib';
import {
  CompressedSpvProof,
  VerifierConfig,
  FetchProofOptions,
  ChainStateProof,
  TransactionInclusionProof,
  BlockInclusionProof,
  RaitoError,
  VerificationError,
  FetchError,
  BitcoinBlockHeader,
  BitcoinTransaction,
} from './types';

// Import WASM bindings (this will be available after building the WASM package)
declare const require: any;

export class RaitoSpvSdk {
  private raitoClient: AxiosInstance;
  private bitcoinClient: AxiosInstance;
  private wasmModule: any;

  constructor(
    private options: {
      raitoRpcUrl?: string;
      bitcoinRpcUrl: string;
      bitcoinRpcUserPwd?: string;
    }
  ) {
    this.raitoClient = axios.create({
      baseURL: options.raitoRpcUrl || 'https://api.raito.wtf',
      timeout: 30000,
      headers: {
        'Accept-Encoding': 'gzip',
      },
    });

    this.bitcoinClient = axios.create({
      baseURL: options.bitcoinRpcUrl,
      timeout: 30000,
      auth: options.bitcoinRpcUserPwd
        ? {
            username: options.bitcoinRpcUserPwd.split(':')[0],
            password: options.bitcoinRpcUserPwd.split(':')[1],
          }
        : undefined,
    });
  }

  /**
   * Initialize the SDK with WASM module
   */
  async init(): Promise<void> {
    try {
      // Load WASM module
      this.wasmModule = require('raito-spv-verify-wasm');
      await this.wasmModule.init();
    } catch (error) {
      throw new RaitoError(`Failed to initialize WASM module: ${error}`);
    }
  }

  /**
   * Verify a compressed SPV proof
   */
  async verifyProof(
    proof: CompressedSpvProof,
    config?: Partial<VerifierConfig>,
    dev: boolean = false
  ): Promise<boolean> {
    if (!this.wasmModule) {
      throw new VerificationError('SDK not initialized. Call init() first.');
    }

    try {
      const verifierConfig = this.createVerifierConfig(config);
      const proofJson = JSON.stringify(proof);
      
      const result = await this.wasmModule.verify_proof_wasm(proofJson);
      return result;
    } catch (error) {
      throw new VerificationError(`Proof verification failed: ${error}`);
    }
  }

  /**
   * Fetch a complete compressed SPV proof for a transaction
   */
  async fetchProof(options: FetchProofOptions): Promise<CompressedSpvProof> {
    try {
      // Fetch chain state proof
      const chainStateProof = await this.fetchChainStateProof(
        options.raitoRpcUrl || this.options.raitoRpcUrl || 'https://api.raito.wtf'
      );

      // Fetch transaction proof
      const transactionProof = await this.fetchTransactionProof(
        options.txid,
        options.bitcoinRpcUrl,
        options.bitcoinRpcUserPwd
      );

      // Fetch block MMR proof
      const blockProof = await this.fetchBlockProof(
        transactionProof.block_height,
        chainStateProof.chainstate.block_height,
        options.raitoRpcUrl || this.options.raitoRpcUrl || 'https://api.raito.wtf',
        options.dev || false
      );

      return {
        chain_state: chainStateProof.chainstate,
        chain_state_proof: chainStateProof.proof,
        block_header: transactionProof.block_header,
        block_header_proof: blockProof,
        transaction: transactionProof.transaction,
        transaction_proof: transactionProof.transaction_proof,
      } as CompressedSpvProof;
    } catch (error) {
      throw new FetchError(`Failed to fetch proof: ${error}`);
    }
  }

  /**
   * Fetch and verify a proof in one operation
   */
  async fetchAndVerifyProof(
    options: FetchProofOptions,
    config?: Partial<VerifierConfig>
  ): Promise<{ proof: CompressedSpvProof; verified: boolean }> {
    const proof = await this.fetchProof(options);
    const verified = await this.verifyProof(proof, config, options.dev);
    return { proof, verified };
  }

  /**
   * Fetch chain state proof from Raito bridge
   */
  private async fetchChainStateProof(raitoRpcUrl: string): Promise<ChainStateProof> {
    try {
      const response = await this.raitoClient.get('/chainstate-proof/recent_proof');
      return response.data;
    } catch (error) {
      throw new FetchError(`Failed to fetch chain state proof: ${error}`);
    }
  }

  /**
   * Fetch transaction inclusion proof from Bitcoin RPC
   * Implements the same logic as fetch_transaction_proof in fetch.rs
   */
  private async fetchTransactionProof(
    txid: string,
    bitcoinRpcUrl: string,
    bitcoinRpcUserPwd?: string
  ): Promise<TransactionInclusionProof> {
    try {
      // Create a temporary client for this specific request
      const client = axios.create({
        baseURL: bitcoinRpcUrl,
        timeout: 30000,
        auth: bitcoinRpcUserPwd
          ? {
              username: bitcoinRpcUserPwd.split(':')[0],
              password: bitcoinRpcUserPwd.split(':')[1],
            }
          : undefined,
      });

      // Step 1: Get transaction inclusion proof (MerkleBlock) - equivalent to get_transaction_inclusion_proof
      const merkleResponse = await client.post('', {
        jsonrpc: '2.0',
        id: 1,
        method: 'gettxoutproof',
        params: [[txid]],
      });

      const merkleProofHex = merkleResponse.data.result;
      if (!merkleProofHex) {
        throw new Error('Transaction inclusion proof not found');
      }

      // Parse the MerkleBlock - equivalent to get_transaction_inclusion_proof
      const merkleBlock = this.parseMerkleBlock(merkleProofHex);
      
      // Step 2: Get block hash from the header - equivalent to header.block_hash()
      const blockHash = merkleBlock.header.getId();

      // Step 3: Get full transaction - equivalent to get_transaction(&txid, &block_hash)
      const txResponse = await client.post('', {
        jsonrpc: '2.0',
        id: 2,
        method: 'getrawtransaction',
        params: [txid, false, blockHash],
      });

      const txHex = txResponse.data.result;
      if (!txHex) {
        throw new Error('Transaction not found');
      }

      const tx = bitcoin.Transaction.fromHex(txHex);
      const transaction = this.convertToBitcoinTransaction(tx);

      // Step 4: Get extended block header info - equivalent to get_block_header_ex(&block_hash)
      const blockResponse = await client.post('', {
        jsonrpc: '2.0',
        id: 3,
        method: 'getblockheader',
        params: [blockHash, true],
      });

      const blockHeaderEx = blockResponse.data.result;
      if (!blockHeaderEx) {
        throw new Error('Block header not found');
      }

      return {
        transaction: transaction,
        transaction_proof: this.hexToBytes(merkleProofHex), // Serialized MerkleBlock
        block_header: this.convertToBitcoinBlockHeader(merkleBlock.header), // Header from MerkleBlock
        block_height: blockHeaderEx.height,
      };
    } catch (error) {
      throw new FetchError(`Failed to fetch transaction proof: ${error}`);
    }
  }

  /**
   * Fetch block MMR inclusion proof from Raito bridge
   */
  private async fetchBlockProof(
    blockHeight: number,
    chainHeight: number,
    raitoRpcUrl: string,
    dev: boolean
  ): Promise<BlockInclusionProof> {
    try {
      let url: string;
      if (dev) {
        url = `http://127.0.0.1:5000/block-inclusion-proof/${blockHeight}?chain_height=${chainHeight}`;
      } else {
        const mmrHeight = await this.getMmrHeight(raitoRpcUrl);
        if (mmrHeight < chainHeight) {
          throw new Error(`MMR height ${mmrHeight} is less than chain height ${chainHeight}`);
        }
        url = `${raitoRpcUrl}/block-inclusion-proof/${blockHeight}?chain_height=${chainHeight}`;
      }

      if (blockHeight > chainHeight) {
        throw new Error(`Block height ${blockHeight} is greater than chain height ${chainHeight}`);
      }

      const response = await axios.get(url);
      return response.data;
    } catch (error) {
      throw new FetchError(`Failed to fetch block proof: ${error}`);
    }
  }

  /**
   * Get the current MMR height from Raito bridge
   */
  private async getMmrHeight(raitoRpcUrl: string): Promise<number> {
    try {
      const response = await axios.get(`${raitoRpcUrl}/head`);
      return response.data;
    } catch (error) {
      throw new FetchError(`Failed to get MMR height: ${error}`);
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

  /**
   * Convert hex string to byte array
   */
  private hexToBytes(hex: string): number[] {
    const bytes = [];
    for (let i = 0; i < hex.length; i += 2) {
      bytes.push(parseInt(hex.substr(i, 2), 16));
    }
    return bytes;
  }

  /**
   * Parse MerkleBlock from hex string following Bitcoin protocol
   * MerkleBlock format: header (80 bytes) + tx_count (varint) + hash_count (varint) + hashes + flag_bytes_count (varint) + flag_bytes
   */
  private parseMerkleBlock(hex: string): { header: bitcoin.Block; txn: any } {
    const bytes = this.hexToBytes(hex);
    let offset = 0;

    // Parse block header (80 bytes)
    const headerBytes = bytes.slice(offset, offset + 80);
    offset += 80;
    const header = bitcoin.Block.fromHex(headerBytes.map(b => b.toString(16).padStart(2, '0')).join(''));

    // Parse transaction count (varint)
    const txCount = this.readVarInt(bytes, offset);
    offset = txCount.offset;

    // Parse hash count (varint)
    const hashCount = this.readVarInt(bytes, offset);
    offset = hashCount.offset;

    // Parse hashes (32 bytes each)
    const hashes = [];
    for (let i = 0; i < hashCount.value; i++) {
      const hashBytes = bytes.slice(offset, offset + 32);
      hashes.push(Buffer.from(hashBytes));
      offset += 32;
    }

    // Parse flag bytes count (varint)
    const flagBytesCount = this.readVarInt(bytes, offset);
    offset = flagBytesCount.offset;

    // Parse flag bytes
    const flagBytes = bytes.slice(offset, offset + flagBytesCount.value);
    offset += flagBytesCount.value;

    // Create MerkleBlock structure
    const txn = {
      txCount: txCount.value,
      hashes: hashes,
      flagBytes: flagBytes
    };

    return {
      header: header,
      txn: txn
    };
  }

  /**
   * Read variable-length integer from byte array
   */
  private readVarInt(bytes: number[], offset: number): { value: number; offset: number } {
    const firstByte = bytes[offset];
    
    if (firstByte < 0xfd) {
      return { value: firstByte, offset: offset + 1 };
    } else if (firstByte === 0xfd) {
      const value = bytes[offset + 1] | (bytes[offset + 2] << 8);
      return { value: value, offset: offset + 3 };
    } else if (firstByte === 0xfe) {
      const value = bytes[offset + 1] | (bytes[offset + 2] << 8) | (bytes[offset + 3] << 16) | (bytes[offset + 4] << 24);
      return { value: value, offset: offset + 5 };
    } else {
      // 0xff - 8 bytes
      let value = 0;
      for (let i = 0; i < 8; i++) {
        value |= bytes[offset + 1 + i] << (i * 8);
      }
      return { value: value, offset: offset + 9 };
    }
  }

  /**
   * Convert bitcoinjs-lib Transaction to BitcoinTransaction interface
   */
  private convertToBitcoinTransaction(tx: bitcoin.Transaction): BitcoinTransaction {
    return {
      version: tx.version,
      is_segwit: tx.hasWitnesses(),
      input: tx.ins.map((input) => ({
        script_sig: input.script ? input.script.toString('hex') : '',
        sequence: input.sequence,
        previous_output: {
          txid: input.hash.toString('hex'),
          vout: input.index,
          data: {
            value: 0, // This would need to be fetched from UTXO set
            pk_script: '', // This would need to be fetched from UTXO set
            cached: false
          },
          block_height: 0, // This would need to be determined
          median_time_past: 0, // This would need to be calculated
          is_coinbase: false // This would need to be determined
        },
        witness: input.witness ? input.witness.map((w: any) => w.toString('hex')) : []
      })),
      output: tx.outs.map((output: any) => ({
        value: output.value,
        pk_script: output.script ? output.script.toString('hex') : ''
      })),
      lock_time: tx.locktime
    };
  }

  /**
   * Convert bitcoinjs-lib Block to BitcoinBlockHeader interface
   */
  private convertToBitcoinBlockHeader(block: bitcoin.Block): BitcoinBlockHeader {
    return {
      version: block.version,
      prev_blockhash: block.prevHash ? block.prevHash.toString('hex') : '',
      merkle_root: block.merkleRoot ? block.merkleRoot.toString('hex') : '',
      time: block.timestamp,
      bits: block.bits,
      nonce: block.nonce
    };
  }
}

/**
 * Create a new RaitoSpvSdk instance
 */
export function createRaitoSpvSdk(options: {
  raitoRpcUrl?: string;
  bitcoinRpcUrl: string;
  bitcoinRpcUserPwd?: string;
}): RaitoSpvSdk {
  return new RaitoSpvSdk(options);
}
