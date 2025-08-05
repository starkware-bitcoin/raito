# Raito Bridge Node

A Bitcoin block indexer that builds Merkle Mountain Range (MMR) accumulator of the Bitcoin blocks, and generates data required for running the [`assumevalid`](../../packages/assumevalid/) program.

## Overview

The Raito Bridge Node serves as a data preprocessing layer for the Bitcoin ZK client, and as an API providing compressed SPV proofs. A compressed SPV proof is a self-sufficient transaction inclusion proof that does not require clients to store the Bitcoin headers locally nor keep connection to a Bitcoin RPC node.

## What it does

1. **Connects to Bitcoin Core** via RPC to fetch block headers
2. **Builds MMR accumulator** using Cairo-compatible Blake2 hashing
3. **Generates sparse roots** - MMR state representations compatible with the Cairo ZK client
4. **Organizes output** into sharded JSON files for efficient access by the proving pipeline

Raito bridge node does not handle reorgs, instead it operates with a configurable lag (by default — 1 block).

## Usage

### Command Line

```bash
# Basic usage with remote RPC node
cargo run --bin raito-bridge-node -- --bitcoin-rpc-url https://bitcoin-mainnet.public.blastapi.io

# With authentication
cargo run --bin raito-bridge-node -- --bitcoin-rpc-url http://localhost:8332 --bitcoin-rpc-userpwd user:password

# Custom data directory and shard size
cargo run --bin raito-bridge-node -- \
  --bitcoin-rpc-url http://localhost:8332 \
  --mmr-db-path ./custom/mmr.db \
  --mmr-roots-dir ./custom/roots \
  --mmr-shard-size 5000

# Production setup with remote node and custom RPC server host
cargo run --bin raito-bridge-node -- \
  --bitcoin-rpc-url https://bitcoin-node.example.com:8332 \
  --bitcoin-rpc-userpwd myuser:mypassword \
  --rpc-host 0.0.0.0:8080 \
  --mmr-shard-size 50000 \
  --log-level warn
```

### Environment Variables

You can use environment variables instead of command line arguments:

```bash
# Set environment variables
export BITCOIN_RPC="http://localhost:8332"
export USERPWD="user:password"

# Run with defaults (no arguments needed)
cargo run --bin raito-bridge-node
```

### Using .env File

Create a `.env` file in the project directory:

```env
BITCOIN_RPC=http://localhost:8332
USERPWD=user:password
```

Then simply run:

```bash
cargo run --bin raito-bridge-node
```

## Configuration

| Option | Default | Environment Variable | Description |
|--------|---------|---------------------|-------------|
| `--bitcoin-rpc-url` | - | `BITCOIN_RPC` | Bitcoin Core RPC URL (required) |
| `--bitcoin-rpc-userpwd` | - | `USERPWD` | RPC credentials in `user:password` format |
| `--rpc-host` | `127.0.0.1:5000` | - | Host and port for the bridge node's RPC server |
| `--mmr-db-path` | `./.mmr_data/mmr.db` | - | SQLite database path for MMR storage |
| `--mmr-roots-dir` | `./.mmr_data/roots` | - | Output directory for sparse roots JSON files |
| `--mmr-shard-size` | `10000` | - | Number of blocks per shard directory |
| `--log-level` | `info` | - | Logging verbosity |

> **Note**: When environment variables are set (either directly or via `.env` file), you can run the bridge node without any command line arguments. This is especially convenient for deployment and development setups.

## Output Format

Sparse roots are written as JSON files organized by block height:
```
.mmr_data/roots/
├── 10000/
│   ├── block_0.json
│   ├── block_1.json
│   └── ...
└── 20000/
    ├── block_10000.json
    └── ...
```

Each file contains the MMR sparse roots at that block height, compatible with Raito's Cairo implementation.

## RPC Server and API Endpoints

The Raito Bridge Node runs an HTTP RPC server that provides REST endpoints for querying MMR data and generating proofs. By default, the server binds to `127.0.0.1:5000`, but this can be configured using the `--rpc-host` option.

### Available Endpoints

#### GET /proof/:height

Generate an inclusion proof for a block at the specified height.

**Parameters:**
- `height` (path parameter): The block height to generate a proof for (0-indexed)

**Response:**
```json
{
  "roots": [
    "0x5fd720d341e64d17d3b8624b17979b0d0dad4fc17d891796a3a51a99d3f41599",
    "0x693aa1ab81c6362fe339fc4c7f6d8ddb1e515701e58c5bb2fb54a193c8287fdc"
  ],
  "siblings": [
    "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66"
  ]
}
```

**Response Fields:**
- `roots`: Array of MMR peak hashes at the time of proof generation (hex-encoded strings)
- `siblings`: Array of sibling hashes needed to reconstruct the path to the root (hex-encoded strings)

**Status Codes:**
- `200 OK`: Proof generated successfully
- `500 Internal Server Error`: Failed to generate proof (e.g., invalid height)

#### GET /head

Get the current head (latest block count) from the MMR.

**Response:**
```json
832500
```

**Response:** The current block count as a JSON number (total number of blocks indexed)

**Status Codes:**
- `200 OK`: Block count retrieved successfully
- `500 Internal Server Error`: Failed to retrieve block count

### Usage Examples

```bash
# Get the current block count
curl http://localhost:5000/head

# Generate a proof for block at height 100
curl http://localhost:5000/proof/100

# Using a custom RPC host
cargo run --bin raito-bridge-node -- \
  --bitcoin-rpc-url http://localhost:8332 \
  --rpc-host 0.0.0.0:8080

# Then query the custom endpoint
curl http://localhost:8080/head
```

### Integration

The RPC server is designed to be used by:
1. **ZK Clients**: To obtain inclusion proofs for Bitcoin blocks
2. **Monitoring Tools**: To track synchronization progress via the `/head` endpoint

## Requirements

- Access to a Bitcoin RPC node
- Sufficient disk space (numbers are for the first 900K blocks)
    * 300MB for the accumulator state DB
    * 3.6GB for the sparse roots files
