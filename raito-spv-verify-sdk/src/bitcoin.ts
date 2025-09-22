import BitcoinJsonRpc from 'bitcoin-json-rpc';
import * as bitcoinjs from 'bitcoinjs-lib';

// -------- Rust-bitcoin-like Types --------
export type BlockHash = string; // 32-byte hex (big-endian display)
export type Txid = string;

export interface BlockHeader {
  version: number;             // i32
  prev_blockhash: BlockHash;   // big-endian hex
  merkle_root: string;         // big-endian hex
  time: number;                // u32
  bits: number;                // u32
  nonce: number;               // u32
}

export interface OutPoint { txid: Txid; vout: number; }
export interface TxIn {
  previous_output: OutPoint;
  script_sig: string;    // hex
  sequence: number;      // u32
  witness: string[];     // hex pushes
}
export interface TxOut {
  value: bigint;         // u64 (sats)
  script_pubkey: string; // hex
}
export interface Transaction {
  version: number;       // i32
  lock_time: number;     // u32
  input: TxIn[];
  output: TxOut[];
}

export interface GetBlockHeaderResult {
  hash: string;
  confirmations: number;
  height: number;
  version: number;
  versionHex: string;
  merkleroot: string;
  time: number;
  mediantime: number;
  nonce: number;
  bits: string;          // hex string in Core JSON
  difficulty: number;
  chainwork: string;
  nTx: number;
  previousblockhash?: string;
  nextblockhash?: string;
}

export interface MerkleBlock {
  header: BlockHeader;
  tx_count: number;   // u32 (total transactions in block)
  hashes: string[];   // big-endian 32-byte hex (txids + internals)
  flags: Uint8Array;  // raw bits
}

// -------- Constants & helpers --------
export const HTTP_REQUEST_TIMEOUT_MS = 5_000;

const NET_ERR = new Set([
  'ECONNRESET','ECONNREFUSED','EPIPE','ETIMEDOUT',
  'EHOSTUNREACH','ENETUNREACH','ENOTFOUND','EAI_AGAIN',
]);

function isRetryableError(err: any): boolean {
  const msg = String(err?.message ?? '').toLowerCase();
  const code = err?.code;
  if (NET_ERR.has(code)) return true;
  if (msg.includes('timeout')) return true;
  if (/\b(5\d\d|408)\b/.test(msg)) return true; // HTTP-ish surfaced in message
  if (code === -28 || msg.includes('loading block index')) return true; // Core warming up
  return false;
}
function sleep(ms: number) { return new Promise(res => setTimeout(res, ms)); }
async function withRetry<T>(op: () => Promise<T>, max = 8) {
  let attempt = 0, delay = 250;
  for (;;) {
    try { return await op(); }
    catch (e) {
      attempt++;
      if (attempt >= max || !isRetryableError(e)) throw e;
      const jitter = Math.floor(Math.random() * (delay / 2));
      await sleep(Math.min(delay + jitter, 5_000));
      delay = Math.min(delay * 2, 5_000);
    }
  }
}
function withTimeout<T>(p: Promise<T>, ms = HTTP_REQUEST_TIMEOUT_MS) {
  return new Promise<T>((resolve, reject) => {
    const t = setTimeout(() => reject(new Error(`RPC timeout after ${ms}ms`)), ms);
    p.then(v => { clearTimeout(t); resolve(v); }, e => { clearTimeout(t); reject(e); });
  });
}

// -------- minimal consensus decoders (header + merkleblock) --------
function hexToBuf(hex: string): Buffer {
  if (hex.startsWith('0x')) hex = hex.slice(2);
  return Buffer.from(hex, 'hex');
}
function hashLEtoBEHex(buf: Buffer): string {
  // consensus encodes hashes little-endian; display big-endian like rust-bitcoin
  return Buffer.from(buf).reverse().toString('hex');
}
function readI32LE(b: Buffer, o: number) { return b.readInt32LE(o); }
function readU32LE(b: Buffer, o: number) { return b.readUInt32LE(o); }

// CompactSize varint
function readCompactSize(b: Buffer, o: number): { value: number; size: number } {
  const first = b.readUInt8(o);
  if (first < 0xfd) return { value: first, size: 1 };
  if (first === 0xfd) return { value: b.readUInt16LE(o + 1), size: 3 };
  if (first === 0xfe) return { value: b.readUInt32LE(o + 1), size: 5 };
  // 0xff: JS number risk, but Core won’t emit >2^32 here for hashes/flags counts
  const low = b.readUInt32LE(o + 1);
  const high = b.readUInt32LE(o + 5);
  return { value: high * 2 ** 32 + low, size: 9 };
}
function readVarBytes(b: Buffer, o: number): { bytes: Buffer; size: number } {
  const { value: len, size: vi } = readCompactSize(b, o);
  const start = o + vi, end = start + len;
  return { bytes: b.subarray(start, end), size: vi + len };
}

function decodeBlockHeaderFromHex(hex: string): BlockHeader {
  const b = hexToBuf(hex);
  if (b.length !== 80) throw new Error(`Invalid block header length: ${b.length}`);
  return {
    version: readI32LE(b, 0),
    prev_blockhash: hashLEtoBEHex(b.subarray(4, 36)),
    merkle_root:    hashLEtoBEHex(b.subarray(36, 68)),
    time:  readU32LE(b, 68),
    bits:  readU32LE(b, 72),
    nonce: readU32LE(b, 76),
  };
}

function decodeMerkleBlockFromHex(hex: string): MerkleBlock {
  const b = hexToBuf(hex);
  if (b.length < 80) throw new Error('MerkleBlock too short');
  const header = decodeBlockHeaderFromHex(b.subarray(0, 80).toString('hex'));
  let off = 80;

  const tx_count = readU32LE(b, off); off += 4;

  const { value: nHashes, size: viH } = readCompactSize(b, off); off += viH;
  const hashes: string[] = [];
  for (let i = 0; i < nHashes; i++) {
    hashes.push(hashLEtoBEHex(b.subarray(off, off + 32)));
    off += 32;
  }

  const { bytes: flags, size: viF } = readVarBytes(b, off); off += viF;

  return { header, tx_count, hashes, flags: new Uint8Array(flags) };
}

// -------- Transaction decoder (bitcoinjs-lib) --------
function decodeTransactionFromHex(hex: string): Transaction {
  const tx = bitcoinjs.Transaction.fromHex(hex);
  const input: TxIn[] = tx.ins.map((inn) => {
    const txid = hashLEtoBEHex(Buffer.from(inn.hash));
    const vout = inn.index;
    const script_sig = Buffer.from(inn.script ?? Buffer.alloc(0)).toString('hex');
    const sequence = inn.sequence >>> 0;
    const witness = (inn.witness ?? []).map(w => Buffer.from(w).toString('hex'));
    return { previous_output: { txid, vout }, script_sig, sequence, witness };
  });
  const output: TxOut[] = tx.outs.map((out) => ({
    value: BigInt(out.value),
    script_pubkey: Buffer.from(out.script).toString('hex'),
  }));
  return { version: tx.version, lock_time: tx.locktime >>> 0, input, output };
}

// -------- Core-only client --------
export class BitcoinCoreClient {
  private rpc: any;
  private timeoutMs: number;

  constructor(
    url: string,                    // e.g. 'http://user:pass@127.0.0.1:8332'
    timeoutMs = HTTP_REQUEST_TIMEOUT_MS
  ) {
    this.rpc = new (BitcoinJsonRpc as any)(url);
    this.timeoutMs = timeoutMs;
  }

  private async raw<T = any>(method: string, params: any[] = []): Promise<T> {
    const call = async () => {
      const c = this.rpc as any;
      if (typeof c.cmd === 'function') return c.cmd(method, ...params);
      if (typeof c[method] === 'function') return c[method](...params);
      if (typeof c.call === 'function') return c.call(method, params);
      throw new Error('bitcoin-json-rpc client lacks cmd()/call()/method');
    };
    return withRetry(() => withTimeout(call(), this.timeoutMs));
  }

  // ---- API (matches your Rust methods, minus wait_block_header) ----

  /** getblockhash(height) */
  async getBlockHash(height: number): Promise<BlockHash> {
    return this.raw<string>('getblockhash', [height]);
  }

  /** getblockheader(hash, false) -> raw 80B header decoded to rust-like struct */
  async getBlockHeader(hash: BlockHash): Promise<BlockHeader> {
    const hex = await this.raw<string>('getblockheader', [hash, false]);
    return decodeBlockHeaderFromHex(hex);
  }

  /** getblockheader(hash, true) -> extended JSON */
  async getBlockHeaderEx(hash: BlockHash): Promise<GetBlockHeaderResult> {
    return this.raw<GetBlockHeaderResult>('getblockheader', [hash, true]);
  }

  /** by height: (header, hash) */
  async getBlockHeaderByHeight(height: number): Promise<{ header: BlockHeader; hash: BlockHash }> {
    const hash = await this.getBlockHash(height);
    const header = await this.getBlockHeader(hash);
    return { header, hash };
  }

  /**
   * getrawtransaction(txid, false [, blockhash]) -> decode to rust-like Transaction.
   * Provide blockhash to avoid needing Core -txindex.
   */
  async getTransaction(txid: Txid, blockHash?: BlockHash): Promise<Transaction> {
    const params = blockHash ? [txid, false, blockHash] : [txid, false];
    const hex = await this.raw<string>('getrawtransaction', params);
    return decodeTransactionFromHex(hex);
  }

  /** gettxoutproof([txid], [blockhash]) -> MerkleBlock (parsed from CMerkleBlock wire format) */
  async getTransactionInclusionProof(txid: Txid, blockHash?: BlockHash): Promise<MerkleBlock> {
    const params = blockHash ? [[txid], blockHash] : [[txid]];
    const hex = await this.raw<string>('gettxoutproof', params);
    return decodeMerkleBlockFromHex(hex);
  }

  /** getblockcount() */
  async getBlockCount(): Promise<number> {
    const n = await this.raw<number>('getblockcount', []);
    return Math.max(0, Math.min(0xffffffff, n | 0));
  }
}
