/**
 * TypeScript type definitions for Raito SPV SDK
 */

// Bitcoin types - exact copies of corresponding types in fetch.rs
export interface BitcoinTransaction {
  version: number;
  is_segwit: boolean;
  input: BitcoinInput[];
  output: BitcoinOutput[];
  lock_time: number;
}

export interface BitcoinInput {
  previous_output: BitcoinOutPoint;
  script_sig: string;
  sequence: number;
  witness: string[];
}

export interface BitcoinOutPoint {
  txid: string;
  vout: number;
}

export interface BitcoinOutputData {
  value: number;
  pk_script: string;
  cached: boolean;
}

export interface BitcoinOutput {
  value: number;
  pk_script: string;
}

export interface BitcoinBlockHeader {
  version: number;
  prev_blockhash: string;
  merkle_root: string;
  time: number;
  bits: number;
  nonce: number;
}

// SPV Proof types
export interface ChainState {
  block_height: number;
  total_work: string;
  best_block_hash: string;
  current_target: string;
  epoch_start_time: number;
  prev_timestamps: number[];
}

export interface BlockInclusionProof {
  peaks_hashes: string[];
  siblings_hashes: string[];
  leaf_index: number;
  leaf_count: number;
}

export interface CompressedSpvProof {
  chain_state: ChainState;
  chain_state_proof: any; // Cairo proof - complex type
  block_header: BitcoinBlockHeader;
  block_header_proof: BlockInclusionProof;
  transaction: BitcoinTransaction;
  transaction_proof: number[]; // Uint8Array as number array
}

export interface VerifierConfig {
  min_work: string;
  bootloader_hash: string;
  task_program_hash: string;
  task_output_size: number;
}

// Fetch types
export interface FetchProofOptions {
  txid: string;
  raitoRpcUrl?: string;
  bitcoinRpcUrl: string;
  bitcoinRpcUserPwd?: string;
  dev?: boolean;
}

export interface ChainStateProof {
  chainstate: ChainState;
  proof: any; // Cairo proof
}

export interface TransactionInclusionProof {
  transaction: BitcoinTransaction;
  transaction_proof: number[];
  block_header: BitcoinBlockHeader;
  block_height: number;
}

// Error types
export class RaitoError extends Error {
  constructor(message: string, public code?: string) {
    super(message);
    this.name = 'RaitoError';
  }
}

export class VerificationError extends RaitoError {
  constructor(message: string) {
    super(message, 'VERIFICATION_ERROR');
    this.name = 'VerificationError';
  }
}

export class FetchError extends RaitoError {
  constructor(message: string) {
    super(message, 'FETCH_ERROR');
    this.name = 'FetchError';
  }
}
