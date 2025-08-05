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
cargo run --bin raito-bridge-node -- --rpc-url https://bitcoin-mainnet.public.blastapi.io

# With authentication
cargo run --bin raito-bridge-node -- --rpc-url http://localhost:8332 --rpc-userpwd user:password

# Custom data directory and shard size
cargo run --bin raito-bridge-node -- \
  --rpc-url http://localhost:8332 \
  --mmr-db-path ./custom/mmr.db \
  --mmr-roots-dir ./custom/roots \
  --mmr-shard-size 5000

# Production setup with remote node
cargo run --bin raito-bridge-node -- \
  --rpc-url https://bitcoin-node.example.com:8332 \
  --rpc-userpwd myuser:mypassword \
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
| `--rpc-url` | - | `BITCOIN_RPC` | Bitcoin Core RPC URL (required) |
| `--rpc-userpwd` | - | `USERPWD` | RPC credentials in `user:password` format |
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

## Requirements

- Access to a Bitcoin RPC node
- Sufficient disk space (numbers are for the first 900K blocks)
    * 300MB for the accumulator state DB
    * 3.6GB for the sparse roots files

