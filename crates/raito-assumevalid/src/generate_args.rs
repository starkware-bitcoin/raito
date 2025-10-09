use anyhow::{anyhow, Result};
use bitcoin::block::Header as BlockHeader;
use raito_cairo_args::adapters::assumevalid_args::to_runner_args_hex;
use raito_spv_mmr::sparse_roots::SparseRoots;
use raito_spv_verify::ChainState;
use std::time::Duration;
use serde::{Deserialize, Serialize};

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
        let url = format!("{}/chain-state/{}", self.config.bridge_node_url, block_height);
        let response = self.make_request(&url).await?;
        let chain_state: ChainState = response.json().await?;
        Ok(chain_state)
    }

    /// Fetch block headers for a given range
    pub async fn get_block_headers(&self, offset: u32, size: u32) -> Result<Vec<BlockHeader>> {
        let url = format!("{}/headers?offset={}&size={}", self.config.bridge_node_url, offset, size);
        let response = self.make_request(&url).await?;
        let headers: Vec<BlockHeader> = response.json().await?;
        Ok(headers)
    }

    /// Fetch MMR roots for a given chain height
    pub async fn get_roots(&self, chain_height: u32) -> Result<SparseRoots> {
        let url = format!("{}/roots?chain_height={}", self.config.bridge_node_url, chain_height);
        let response = self.make_request(&url).await?;
        let json_text = response.text().await?;
        let json_value: serde_json::Value = serde_json::from_str(&json_text)?;
        
        // Manually parse SparseRoots from JSON
        let block_height = json_value["block_height"]
            .as_u64()
            .ok_or_else(|| anyhow!("Missing or invalid block_height"))? as u32;
        
        let roots_array = json_value["roots"]
            .as_array()
            .ok_or_else(|| anyhow!("Missing or invalid roots array"))?;
        
        let mut roots = Vec::new();
        for root_obj in roots_array {
            if let Some(hi) = root_obj["hi"].as_str() {
                if let Some(lo) = root_obj["lo"].as_str() {
                    // Reconstruct the full hex string
                    let full_hex = format!("0x{}{}", hi, lo);
                    roots.push(full_hex);
                } else {
                    return Err(anyhow!("Invalid root format: missing 'lo'"));
                }
            } else {
                return Err(anyhow!("Invalid root format: missing 'hi'"));
            }
        }
        
        Ok(SparseRoots {
            block_height,
            roots,
        })
    }

    /// Get the current head (latest block height)
    pub async fn get_head(&self) -> Result<u32> {
        let url = format!("{}/head", self.config.bridge_node_url);
        let response = self.make_request(&url).await?;
        let head: u32 = response.json().await?;
        Ok(head)
    }

    /// Make an HTTP request
    async fn make_request(&self, url: &str) -> Result<reqwest::Response> {
        let response = self.client.get(url).send().await?;
        
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
    /// Optional chain state proof
    pub chain_state_proof: Option<Vec<u8>>,
}

/// Result of argument generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateArgsResult {
    /// Generated Cairo arguments
    pub cairo_args: Vec<String>,
    /// Output file path
    pub output_file: String,
    /// Generation time
    pub elapsed: Duration,
}

/// Generate assumevalid args for the given parameters
pub async fn generate_assumevalid_args(
    client: &ProveClient,
    params: AssumeValidParams,
) -> Result<Vec<String>> {
    tracing::info!("Generating assumevalid args for height {} with {} blocks", 
                   params.start_height, params.block_count);

    // Fetch chain state for the starting height
    let chain_state = client.get_chain_state(params.start_height).await?;
    tracing::info!("Fetched chain state for height {}", params.start_height);

    // Fetch block headers for the range
    let block_headers = client.get_block_headers(params.start_height, params.block_count).await?;
    tracing::info!("Fetched {} block headers", block_headers.len());

    // Fetch MMR roots
    let block_mmr = client.get_roots(params.start_height).await?;
    tracing::info!("Fetched MMR roots for chain height {:?}", params.start_height);

    // Generate Cairo-compatible arguments
    let cairo_args = to_runner_args_hex(
        chain_state,
        &block_headers,
        &block_mmr,
        params.chain_state_proof.as_deref(),
    )?;

    tracing::info!("Generated {} Cairo arguments", cairo_args.len());

    Ok(cairo_args)
}

/// Save Cairo arguments to a file
pub async fn save_cairo_args_to_file(cairo_args: &[String], file_path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(cairo_args)?;
    tokio::fs::write(file_path, json).await?;
    tracing::info!("Saved {} Cairo arguments to {}", cairo_args.len(), file_path);
    Ok(())
}

/// Generate and save assumevalid arguments
pub async fn generate_and_save_args(
    client: &ProveClient,
    params: AssumeValidParams,
    output_file: &str,
) -> Result<GenerateArgsResult> {
    let start_time = std::time::Instant::now();
    
    let cairo_args = generate_assumevalid_args(client, params).await?;
    save_cairo_args_to_file(&cairo_args, output_file).await?;
    
    let elapsed = start_time.elapsed();
    
    Ok(GenerateArgsResult {
        cairo_args,
        output_file: output_file.to_string(),
        elapsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_config_default() {
        let config = ProveConfig::default();
        assert_eq!(config.bridge_node_url, "https://api.raito.wtf/");
    }

    #[test]
    fn test_assume_valid_params() {
        let params = AssumeValidParams {
            start_height: 100,
            block_count: 10,
            chain_state_proof: None,
        };
        assert_eq!(params.start_height, 100);
        assert_eq!(params.block_count, 10);
    }
}
