use crate::adapters::to_runner_args_hex;
use anyhow::{anyhow, Result};
use bitcoin::block::Header as BlockHeader;
use cairo_air::utils::{deserialize_proof_from_file, ProofFormat};
use raito_spv_mmr::sparse_roots::SparseRoots;
use raito_spv_verify::ChainState;
use std::path::PathBuf;
use tracing::debug;

use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

/// Configuration for the raito-assumevalid client
#[derive(Debug, Clone)]
pub struct ProveConfig {
    /// Bridge node RPC URL
    pub bridge_node_url: String,
}

impl Default for ProveConfig {
    fn default() -> Self {
        Self {
            bridge_node_url: "https://api.raito.wtf/".to_string(),
        }
    }
}

/// Client for interacting with raito-bridge-node
pub struct ProveClient {
    config: ProveConfig,
    client: reqwest::Client,
}

impl ProveClient {
    /// Create a new ProveClient with the given configuration
    pub fn new(config: ProveConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Fetch chain state for a given block height
    pub async fn get_chain_state(&self, block_height: u32) -> Result<ChainState> {
        let url = format!(
            "{}/chain-state/{}",
            self.config.bridge_node_url, block_height
        );
        let response = self.make_request(&url).await?;
        Ok(response.json().await?)
    }

    /// Fetch block headers for a given range
    pub async fn get_block_headers(&self, offset: u32, size: u32) -> Result<Vec<BlockHeader>> {
        let url = format!(
            "{}/headers?offset={}&size={}",
            self.config.bridge_node_url, offset, size
        );
        let response = self.make_request(&url).await?;
        Ok(response.json().await?)
    }

    /// Fetch MMR roots for a given chain height
    pub async fn get_mmr_roots(&self, chain_height: u32) -> Result<SparseRoots> {
        let url = format!(
            "{}/roots?chain_height={}",
            self.config.bridge_node_url, chain_height
        );
        let response = self.make_request(&url).await?;
        Ok(response.json().await?)
    }

    /// Get the current head (latest block height)
    pub async fn get_head(&self) -> Result<u32> {
        let url = format!("{}/head", self.config.bridge_node_url);
        let response = self.make_request(&url).await?;
        Ok(response.json().await?)
    }
    /// Make an HTTP request
    async fn make_request(&self, url: &str) -> Result<reqwest::Response> {
        debug!("Making request to {}", url);
        let response = self
            .client
            .get(url)
            .header("Accept-Encoding", "gzip")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        Ok(response)
    }
}

/// Parameters for generating assumevalid args
#[derive(Debug, Clone)]
pub struct AssumeValidParams {
    /// Starting block height
    pub start_height: u32,
    /// Number of blocks to include
    pub block_count: u32,
    /// Optional chain state proof path
    pub chain_state_proof_path: Option<PathBuf>,
}

/// Generate assumevalid args for the given parameters
pub async fn generate_assumevalid_args(
    client: &ProveClient,
    params: AssumeValidParams,
) -> Result<Vec<String>> {
    debug!(
        "Generating assumevalid args for height {} with {} blocks",
        params.start_height, params.block_count
    );

    // Fetch chain state for the starting height
    let chain_state = client.get_chain_state(params.start_height).await?;
    debug!("Fetched chain state for height {}", params.start_height);

    // Fetch block headers for the range: starting AFTER the current chain_state height
    let block_headers = client
        .get_block_headers(params.start_height + 1, params.block_count)
        .await?;
    debug!("Fetched {} block headers", block_headers.len());

    // Fetch MMR roots
    let block_mmr = client.get_mmr_roots(params.start_height).await?;
    debug!(
        "Fetched MMR roots for chain height {:?}",
        params.start_height
    );

    let chain_state_proof = if let Some(path) = &params.chain_state_proof_path {
        Some(deserialize_proof_from_file::<Blake2sMerkleHasher>(
            path,
            ProofFormat::CairoSerde,
        )?)
    } else {
        None
    };

    // Generate Cairo-compatible arguments
    let cairo_args = to_runner_args_hex(chain_state, &block_headers, &block_mmr, chain_state_proof);

    debug!("Generated {} Cairo arguments", cairo_args.len());

    Ok(cairo_args)
}

/// Generate and save assumevalid args to a file
pub async fn generate_and_save_args(
    client: &ProveClient,
    params: AssumeValidParams,
    file_path: &str,
) -> Result<()> {
    let cairo_args = generate_assumevalid_args(client, params).await?;
    save_cairo_args_to_file(&cairo_args, file_path).await?;
    Ok(())
}

/// Save Cairo arguments to a file
pub async fn save_cairo_args_to_file(cairo_args: &[String], file_path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(cairo_args)?;
    tokio::fs::write(file_path, json).await?;
    debug!(
        "Saved {} Cairo arguments to {}",
        cairo_args.len(),
        file_path
    );
    Ok(())
}
