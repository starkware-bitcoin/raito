use base64::{engine::general_purpose, Engine as _};
use bitcoin::block::Header as BlockHeader;
use bitcoin::consensus::Decodable;
use bitcoin::BlockHash;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::http_client::{HeaderMap, HeaderValue, HttpClient};
use jsonrpsee::rpc_params;
use std::time::Duration;
use tracing::debug;

/// Bitcoin RPC client
pub struct BitcoinClient {
    client: HttpClient,
    block_count: u32,
}

impl BitcoinClient {
    /// Create a new Bitcoin RPC client
    pub fn new(url: String, userpwd: Option<String>) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(userpwd) = userpwd {
            let creds = general_purpose::STANDARD.encode(userpwd);
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Basic {creds}")).unwrap(),
            );
        };
        // Add retry logic
        let client = HttpClient::builder()
            .set_headers(headers)
            .request_timeout(Duration::from_secs(5))
            .build(url)
            .map_err(|e| anyhow::anyhow!("Failed to create Bitcoin RPC client: {}", e))?;
        Ok(Self {
            client,
            block_count: 0,
        })
    }

    async fn request<T: Decodable>(&self, method: &str, params: ArrayParams) -> anyhow::Result<T> {
        let res_hex: String = self.client.request(method, params).await?;
        let res_bytes = hex::decode(res_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode response: {}", e))?;
        bitcoin::consensus::deserialize(&res_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize response: {}", e))
    }

    pub async fn get_block_hash(&self, height: u32) -> anyhow::Result<BlockHash> {
        self.client
            .request("getblockhash", rpc_params![height])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block hash: {}", e))
    }

    pub async fn get_block_header(&self, hash: &BlockHash) -> anyhow::Result<BlockHeader> {
        self.request("getblockheader", rpc_params![hash.to_string(), false])
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
        let res: u64 = self
            .client
            .request("getblockcount", rpc_params![])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block count: {}", e))?;
        Ok(res as u32)
    }

    /// Wait for a block header at the given height
    pub async fn wait_block_header(
        &mut self,
        height: u32,
    ) -> anyhow::Result<(BlockHeader, BlockHash)> {
        while height >= self.block_count {
            self.block_count = self.get_block_count().await?;
            if height < self.block_count {
                debug!("New block count: {}", self.block_count);
                break;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
        self.get_block_header_by_height(height).await
    }
}
