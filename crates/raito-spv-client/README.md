# Compressed SPV proof client

A small CLI with two commands:
- Fetch a compressed proof for a Bitcoin transaction from network sources
- Verify that proof completely offline on a stateless machine (e.g., air‑gapped)

Goal: Produce a self‑sufficient proof that can be verified by a client with no prior state and no network access — suitable for air‑gapped environments or long‑term archival.

The resulting proof is written to disk in a compact binary format using [`serde-brief`](https://docs.rs/serde-brief).

## Installation

```bash
cargo install --locked --git https://github.com/starkware-bitcoin/raito raito-spv-client
# verify
raito-spv-client --help
```

## CLI

Global option:
- `--log-level <level>`: Logging level (`off`, `error`, `warn`, `info`, `debug`, `trace`). Default: `info`.

Subcommands:

### fetch
Fetch all components and write a compressed proof to a file.

Required:
- `--txid <TXID>`: Transaction id to prove.
- `--proof-path <PATH>`: Path to write the proof file.

Optional (can also be provided via env):
- `--raito-rpc-url <URL>`: Raito bridge RPC base URL. Default: `https://api.raito.wtf`. Env: `RAITO_BRIDGE_RPC`.
- `--bitcoin-rpc-url <URL>`: Bitcoin node RPC URL. Env: `BITCOIN_RPC`.
- `--bitcoin-rpc-userpwd <USER:PASSWORD>`: Basic auth credentials for Bitcoin RPC. Env: `USERPWD`.
- `--verify`: Verify the proof immediately after fetching.
- `--dev`: Development mode. Uses local bridge node and skips certain cross-checks.

Example:

```bash
cargo run -p raito-spv-client -- --log-level debug fetch \
  --txid <hex_txid> \
  --proof-path ./proofs/tx_proof.brief \
  --raito-rpc-url https://api.raito.wtf \
  --bitcoin-rpc-url http://127.0.0.1:8332 \
  --bitcoin-rpc-userpwd user:pass \
  --verify
```

### verify
Read a proof from disk and verify it.

- Designed to run completely offline; no network calls
- Stateless: verification uses only the data embedded in the proof
- Suitable for air‑gapped machines and long‑term archival

Required:
- `--proof-path <PATH>`: Path to the proof file.

Optional:
- `--dev`: Development mode. Skips certain cross-checks (e.g., strict MMR height equality).

```bash
cargo run -p raito-spv-client -- verify --proof-path ./proofs/tx_proof.brief
# or with dev mode enabled
cargo run -p raito-spv-client -- verify --proof-path ./proofs/tx_proof.brief --dev
```

Note: Implementation details of verification may evolve; the intended behavior is fully offline verification using the self‑contained proof.

## Output proof format

Proofs are written using `serde-brief` (binary, compact). The file contains a serialized `CompressedSpvProof`:

- `chain_state: ChainState`
  - Snapshot of chain height, total work, best block hash, current target, epoch start time, and previous timestamps.
- `chain_state_proof: CairoProof<Blake2sMerkleHasher>`
  - Recursive STARK proof attesting to the validity of `chain_state` and the block MMR root.
- `block_header: bitcoin::block::Header`
  - The header of the block containing the transaction.
- `block_header_proof: BlockInclusionProof`
  - Inclusion proof of `block_header` in the MMR (from `raito-spv-core`).
- `transaction: bitcoin::Transaction`
  - The full Bitcoin transaction being proven.
- `transaction_proof: Vec<u8>`
  - Bitcoin `PartialMerkleTree` (consensus-encoded) containing the Merkle path for the transaction within the block.

This format is not human‑readable. To deserialize programmatically, use `serde-brief` reader APIs.
