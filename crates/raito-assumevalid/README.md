# raito-assumevalid

A Rust crate for generating assumevalid arguments for Cairo programs. This crate provides both a library interface and a command-line tool for fetching chain state and block headers from a raito-bridge-node and generating Cairo-compatible arguments.

## Features

- **Library Interface**: Use `raito-assumevalid` as a dependency in your Rust projects
- **CLI Tool**: Command-line interface for generating and managing assumevalid arguments
- **Bridge Node Integration**: Fetches data from raito-bridge-node via HTTP API
- **Cairo Serialization**: Converts data to Cairo-compatible format using raito-cairo-args
- **Flexible Configuration**: Configurable bridge node URL

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
raito-assumevalid = { path = "../raito-assumevalid" }
```

## Library Usage

```rust
use raito_assumevalid::{ProveClient, ProveConfig, AssumeValidParams, generate_assumevalid_args, save_cairo_args_to_file};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let config = ProveConfig {
        bridge_node_url: "https://api.raito.wtf/".to_string(),
    };
    
    // Create client
    let client = ProveClient::new(config);
    
    // Define parameters
    let params = AssumeValidParams {
        start_height: 100,
        block_count: 10,
        chain_height: None, // Use latest
        chain_state_proof: None,
    };
    
    // Generate assumevalid args
    let cairo_args = generate_assumevalid_args(&client, params).await?;
    
    println!("Generated {} Cairo arguments", cairo_args.len());
    
    // Save to file
    save_cairo_args_to_file(&cairo_args, "args.json").await?;
    
    Ok(())
}
```

## CLI Usage

### Generate assumevalid arguments

```bash
# Generate args for blocks 100-109
raito-assumevalid generate --start-height 100 --block-count 10

# Specify output file
raito-assumevalid generate --start-height 100 --block-count 10 --output my_args.json

# Use custom bridge node
raito-assumevalid --bridge-url http://localhost:8080 generate --start-height 100 --block-count 10
```

### Query bridge node

```bash
# Get current head
raito-assumevalid head

# Get chain state for specific height
raito-assumevalid chain-state 100
```

## Configuration

### Environment Variables

- `RAITO_BRIDGE_URL`: Default bridge node URL

### Command Line Options

- `--bridge-url`: Bridge node RPC URL (default: https://api.raito.wtf/)
- `--log-level`: Log level (trace, debug, info, warn, error)

## API Reference

### Core Types

- `ProveConfig`: Configuration for the client
- `ProveClient`: HTTP client for bridge node communication
- `AssumeValidParams`: Parameters for argument generation

### Key Functions

- `generate_assumevalid_args()`: Generate assumevalid arguments (returns `Vec<String>`)
- `save_cairo_args_to_file()`: Save Cairo arguments to JSON file

## Dependencies

- `raito-bridge-node`: For fetching chain state and block headers
- `raito-cairo-args`: For Cairo-compatible serialization
- `raito-spv-verify`: For chain state types
- `raito-spv-mmr`: For MMR roots types

## License

This project is part of the Raito ecosystem. See the main project LICENSE for details.
