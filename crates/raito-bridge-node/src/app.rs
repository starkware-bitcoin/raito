//! Application server and client for managing MMR accumulator operations via async message passing.

use std::path::PathBuf;

use bitcoin::block::Header as BlockHeader;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, info};

use raito_spv_core::{
    block_mmr::{BlockInclusionProof, BlockMMR},
    sparse_roots::SparseRoots,
};

/// Request sent to the application server via the API channel
pub struct ApiRequest {
    /// The body of the API request containing the specific operation
    pub body: ApiRequestBody,
    /// Channel to send the response back to the caller
    pub tx_response: oneshot::Sender<ApiResponse>,
}

pub type ApiResponse = Result<ApiResponseBody, anyhow::Error>;

/// Possible request operations that can be sent to the application server
pub enum ApiRequestBody {
    /// Get the current block count from the MMR
    GetBlockCount(),
    /// Get MMR sparse roots for a given chain height (optional)
    /// The chain height is the number of blocks in the MMR minus one
    GetSparseRoots(Option<u32>),
    /// Add a new block header to the MMR
    AddBlock(BlockHeader),
    /// Generate an inclusion proof for a block at the given height and chain height (optional)
    GenerateBlockProof((u32, Option<u32>)),
}

/// Response body for API requests containing the result data
pub enum ApiResponseBody {
    /// Response containing the current block count
    GetBlockCount(u32),
    /// Response containing the sparse roots for a given block count
    GetSparseRoots(SparseRoots),
    /// Response containing sparse roots after adding a block
    AddBlock(SparseRoots),
    /// Response containing the inclusion proof for a block
    GenerateBlockProof(BlockInclusionProof),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Path to the database storing the MMR accumulator state
    pub mmr_db_path: PathBuf,
    /// Api requests channel capacity
    pub api_requests_capacity: usize,
}

/// The main application server that processes API requests and manages the MMR accumulator
pub struct AppServer {
    config: AppConfig,
    rx_requests: mpsc::Receiver<ApiRequest>,
    rx_shutdown: broadcast::Receiver<()>,
}

/// Client for communicating with the application server via async channels
#[derive(Clone)]
pub struct AppClient {
    tx_requests: mpsc::Sender<ApiRequest>,
}

impl AppServer {
    pub fn new(
        config: AppConfig,
        rx_requests: mpsc::Receiver<ApiRequest>,
        rx_shutdown: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            config,
            rx_requests,
            rx_shutdown,
        }
    }

    async fn run_inner(&mut self) -> Result<(), anyhow::Error> {
        info!("App server started");

        // We need to specify mmr_id to have deterministic keys in the database
        let mut mmr = BlockMMR::from_file(&self.config.mmr_db_path, "blocks").await?;

        loop {
            tokio::select! {
                Some(req) = self.rx_requests.recv() => {
                    match req.body {
                        ApiRequestBody::GetBlockCount() => {
                            let res = mmr.get_block_count().await.map(|block_count| ApiResponseBody::GetBlockCount(block_count));
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GetBlockCount request"))?;
                        }
                        ApiRequestBody::GetSparseRoots(chain_height) => {
                            let res = mmr.get_sparse_roots(chain_height).await.map(|sparse_roots| ApiResponseBody::GetSparseRoots(sparse_roots));
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GetSparseRoots request"))?;
                        }
                        ApiRequestBody::GenerateBlockProof((block_height, chain_height)) => {
                            let res = mmr.generate_proof(block_height, chain_height).await.map(|proof| ApiResponseBody::GenerateBlockProof(proof));
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GenerateBlockProof request"))?;
                        }
                        ApiRequestBody::AddBlock(block_header) => {
                            // This is a local-only method, so we treat errors differently here
                            mmr.add_block_header(block_header).await?;
                            let sparse_roots = mmr.get_sparse_roots(None).await?;
                            let res = Ok(ApiResponseBody::AddBlock(sparse_roots));
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to AddBlock request"))?;
                        }
                    }
                },
                _ = self.rx_shutdown.recv() => {
                    return Ok(())
                }
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), ()> {
        match self.run_inner().await {
            Err(err) => {
                error!("App server exited: {}", err);
                Err(())
            }
            Ok(()) => {
                info!("App server terminated");
                Ok(())
            }
        }
    }
}

impl AppClient {
    pub fn new(tx_requests: mpsc::Sender<ApiRequest>) -> Self {
        Self { tx_requests }
    }

    /// Helper method to send a request and handle the response
    async fn send_request<T>(
        &self,
        body: ApiRequestBody,
        extract_response: impl FnOnce(ApiResponseBody) -> Option<T>,
    ) -> Result<T, anyhow::Error> {
        let (tx_response, rx_response) = oneshot::channel();
        self.tx_requests
            .send(ApiRequest { body, tx_response })
            .await?;

        let res = rx_response
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send request"))?;

        match res {
            Ok(response_body) => extract_response(response_body)
                .ok_or_else(|| anyhow::anyhow!("Unexpected response type")),
            Err(err) => Err(err),
        }
    }

    pub async fn get_block_count(&self) -> Result<u32, anyhow::Error> {
        self.send_request(ApiRequestBody::GetBlockCount(), |response| match response {
            ApiResponseBody::GetBlockCount(block_count) => Some(block_count),
            _ => None,
        })
        .await
    }

    pub async fn get_sparse_roots(
        &self,
        block_count: Option<u32>,
    ) -> Result<SparseRoots, anyhow::Error> {
        self.send_request(
            ApiRequestBody::GetSparseRoots(block_count),
            |response| match response {
                ApiResponseBody::GetSparseRoots(sparse_roots) => Some(sparse_roots),
                _ => None,
            },
        )
        .await
    }

    pub async fn add_block(&self, block_header: BlockHeader) -> Result<SparseRoots, anyhow::Error> {
        self.send_request(
            ApiRequestBody::AddBlock(block_header),
            |response| match response {
                ApiResponseBody::AddBlock(sparse_roots) => Some(sparse_roots),
                _ => None,
            },
        )
        .await
    }

    pub async fn generate_block_proof(
        &self,
        block_height: u32,
        block_count: Option<u32>,
    ) -> Result<BlockInclusionProof, anyhow::Error> {
        self.send_request(
            ApiRequestBody::GenerateBlockProof((block_height, block_count)),
            |response| match response {
                ApiResponseBody::GenerateBlockProof(proof) => Some(proof),
                _ => None,
            },
        )
        .await
    }
}

/// Create app server and client
pub fn create_app(
    config: AppConfig,
    rx_shutdown: broadcast::Receiver<()>,
) -> (AppServer, AppClient) {
    let (tx_requests, rx_requests) = mpsc::channel(config.api_requests_capacity);
    let server = AppServer::new(config, rx_requests, rx_shutdown);
    let client = AppClient::new(tx_requests);
    (server, client)
}
