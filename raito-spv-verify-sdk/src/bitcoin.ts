// -------- Rust-bitcoin-like Types --------
export type BlockHash = string; // 32-byte hex (big-endian display)
export type Txid = string;

export interface BlockHeader {
  version: number; // i32
  prev_blockhash: BlockHash; // big-endian hex
  merkle_root: string; // big-endian hex
  time: number; // u32
  bits: number; // u32
  nonce: number; // u32
  blockHash(): BlockHash; // method to calculate block hash
}

export interface OutPoint {
  txid: Txid;
  vout: number;
}
export interface TxIn {
  previous_output: OutPoint;
  script_sig: string; // hex
  sequence: number; // u32
  witness: string[]; // hex pushes
}
export interface TxOut {
  value: bigint; // u64 (sats)
  script_pubkey: string; // hex
}
export interface Transaction {
  version: number; // i32
  lock_time: number; // u32
  input: TxIn[];
  output: TxOut[];
}
