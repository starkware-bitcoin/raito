//! Bitcoin RPC client for fetching block headers and chain information with retry logic.

use base64::{engine::general_purpose, Engine as _};
use bitcoin::block::Header as BlockHeader;
use bitcoin::consensus::Decodable;
use bitcoin::BlockHash;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::http_client::{HeaderMap, HeaderValue, HttpClient};
use jsonrpsee::rpc_params;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::debug;

/// Default HTTP request timeout
pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Default block count update interval in seconds
pub const BLOCK_COUNT_UPDATE_INTERVAL: Duration = Duration::from_secs(10);

/// Bitcoin RPC client
pub struct BitcoinClient {
    client: HttpClient,
    block_count: u32,
    backoff: backoff::ExponentialBackoff,
}

impl BitcoinClient {
    /// Create a new Bitcoin RPC client with default retry settings (exponential backoff)
    pub fn new(url: String, userpwd: Option<String>) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(userpwd) = userpwd {
            let creds = general_purpose::STANDARD.encode(userpwd);
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Basic {creds}")).unwrap(),
            );
        };

        let client = HttpClient::builder()
            .set_headers(headers)
            .request_timeout(HTTP_REQUEST_TIMEOUT)
            .build(url)
            .map_err(|e| anyhow::anyhow!("Failed to create Bitcoin RPC client: {}", e))?;

        Ok(Self {
            client,
            block_count: 0,
            backoff: backoff::ExponentialBackoff::default(),
        })
    }

    async fn request_decode<T: Decodable>(
        &self,
        method: &str,
        params: ArrayParams,
    ) -> anyhow::Result<T> {
        request_with_retry(self.backoff.clone(), || async {
            let res_hex: String = self
                .client
                .request(method, params.clone())
                .await
                .map_err(|e| anyhow::anyhow!("RPC request failed: {}", e))?;
            let res_bytes = hex::decode(res_hex)
                .map_err(|e| anyhow::anyhow!("Failed to decode response: {}", e))?;
            bitcoin::consensus::deserialize(&res_bytes)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize response: {}", e))
        })
        .await
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: ArrayParams,
    ) -> anyhow::Result<T> {
        request_with_retry(self.backoff.clone(), || async {
            self.client
                .request(method, params.clone())
                .await
                .map_err(|e| anyhow::anyhow!("RPC request failed: {}", e))
        })
        .await
    }

    /// Get block hash by height
    pub async fn get_block_hash(&self, height: u32) -> anyhow::Result<BlockHash> {
        self.request("getblockhash", rpc_params![height])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block hash: {}", e))
    }

    /// Get block header by hash
    pub async fn get_block_header(&self, hash: &BlockHash) -> anyhow::Result<BlockHeader> {
        self.request_decode("getblockheader", rpc_params![hash.to_string(), false])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block header: {}", e))
    }

    /// Get block header by height
    pub async fn get_block_header_by_height(
        &self,
        height: u32,
    ) -> anyhow::Result<(BlockHeader, BlockHash)> {
        let hash = self.get_block_hash(height).await?;
        let header = self.get_block_header(&hash).await?;
        Ok((header, hash))
    }

    /// Get current chain height
    pub async fn get_block_count(&self) -> anyhow::Result<u32> {
        self.request("getblockcount", rpc_params![])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block count: {}", e))
            .map(|res: u64| res as u32)
    }

    /// Wait for a block header at the given height.
    /// If the specified lag is non-zero, the function will wait till `lag` blocks are built on top of the expected block.
    pub async fn wait_block_header(
        &mut self,
        height: u32,
        lag: u32,
    ) -> anyhow::Result<(BlockHeader, BlockHash)> {
        while height >= self.block_count {
            self.block_count = self.get_block_count().await?.saturating_sub(lag);
            if height < self.block_count {
                debug!("New block count: {}", self.block_count);
                break;
            } else {
                tokio::time::sleep(BLOCK_COUNT_UPDATE_INTERVAL).await;
            }
        }
        self.get_block_header_by_height(height).await
    }
}

/// Execute a request with retry logic using exponential backoff
async fn request_with_retry<F, Fut, T>(
    backoff: backoff::ExponentialBackoff,
    operation: F,
) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    use backoff::{future::retry, Error};

    retry(backoff, || async {
        operation().await.map_err(Error::transient)
    })
    .await
}
