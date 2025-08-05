use std::path::PathBuf;

use bitcoin::block::Header as BlockHeader;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, info};

use crate::{
    mmr::{Accumulator, InclusionProof},
    sparse_roots::SparseRoots,
};

pub struct ApiRequest {
    pub body: ApiRequestBody,
    pub tx_response: oneshot::Sender<ApiResponse>,
}

pub type ApiResponse = Result<ApiResponseBody, anyhow::Error>;

pub enum ApiRequestBody {
    GetBlockCount(),
    AddBlock(BlockHeader),
    GenerateBlockProof(u32),
}

pub enum ApiResponseBody {
    GetBlockCount(u32),
    AddBlock(SparseRoots),
    GenerateBlockProof(InclusionProof),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Path to the database storing the MMR accumulator state
    pub mmr_db_path: PathBuf,
    /// Api requests channel capacity
    pub api_requests_capacity: usize,
}

pub struct AppServer {
    config: AppConfig,
    rx_requests: mpsc::Receiver<ApiRequest>,
    rx_shutdown: broadcast::Receiver<()>,
}

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
        let mut mmr = Accumulator::from_file(&self.config.mmr_db_path, "blocks").await?;

        loop {
            tokio::select! {
                Some(req) = self.rx_requests.recv() => {
                    match req.body {
                        ApiRequestBody::GetBlockCount() => {
                            let res = mmr.get_block_count().await.map(|block_count| ApiResponseBody::GetBlockCount(block_count));
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GetBlockCount request"))?;
                        }
                        ApiRequestBody::GenerateBlockProof(block_height) => {
                            let res = mmr.generate_proof(block_height).await.map(|proof| ApiResponseBody::GenerateBlockProof(proof));
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GenerateBlockProof request"))?;
                        }
                        ApiRequestBody::AddBlock(block_header) => {
                            // This is a local-only method, so we treat errors differently here
                            mmr.add_block_header(block_header).await?;
                            let sparse_roots = mmr.get_sparse_roots().await?;
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

    pub async fn get_block_count(&self) -> Result<u32, anyhow::Error> {
        let (tx_response, rx_response) = oneshot::channel();
        self.tx_requests
            .send(ApiRequest {
                body: ApiRequestBody::GetBlockCount(),
                tx_response,
            })
            .await?;
        let res = rx_response
            .await
            .map_err(|_| anyhow::anyhow!("Failed to get block count"))?;
        match res {
            Ok(ApiResponseBody::GetBlockCount(block_count)) => Ok(block_count),
            Err(err) => Err(err),
            _ => Err(anyhow::anyhow!(
                "Unexpected response to GetBlockCount request"
            )),
        }
    }

    pub async fn add_block(&self, block_header: BlockHeader) -> Result<SparseRoots, anyhow::Error> {
        let (tx_response, rx_response) = oneshot::channel();
        self.tx_requests
            .send(ApiRequest {
                body: ApiRequestBody::AddBlock(block_header),
                tx_response,
            })
            .await?;
        let res = rx_response
            .await
            .map_err(|_| anyhow::anyhow!("Failed to add block"))?;
        match res {
            Ok(ApiResponseBody::AddBlock(sparse_roots)) => Ok(sparse_roots),
            Err(err) => Err(err),
            _ => Err(anyhow::anyhow!("Unexpected response to AddBlock request")),
        }
    }

    pub async fn generate_block_proof(
        &self,
        block_height: u32,
    ) -> Result<InclusionProof, anyhow::Error> {
        let (tx_response, rx_response) = oneshot::channel();
        self.tx_requests
            .send(ApiRequest {
                body: ApiRequestBody::GenerateBlockProof(block_height),
                tx_response,
            })
            .await?;
        let res = rx_response
            .await
            .map_err(|_| anyhow::anyhow!("Failed to generate block proof"))?;
        match res {
            Ok(ApiResponseBody::GenerateBlockProof(proof)) => Ok(proof),
            Err(err) => Err(err),
            _ => Err(anyhow::anyhow!(
                "Unexpected response to GenerateBlockProof request"
            )),
        }
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
