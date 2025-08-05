use std::path::PathBuf;

use bitcoin::block::Header as BlockHeader;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, info};

use crate::mmr::Accumulator;

pub struct ApiRequest {
    pub body: ApiRequestBody,
    pub tx_response: oneshot::Sender<ApiResponse>,
}

pub enum ApiRequestBody {
    GetBlockCount(),
    AddBlock(BlockHeader),
    GenerateBlockProof(u32),
}

pub enum ApiResponse {
    GetBlockCount(u32),
    AddBlock(Vec<String>),
    GenerateBlockProof(),
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
                            let block_count = mmr.get_block_count().await?;
                            let res = ApiResponse::GetBlockCount(block_count);
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GetBlockCount request"))?;
                        }
                        ApiRequestBody::AddBlock(block_header) => {
                            mmr.add_block_header(block_header).await?;
                            let sparse_roots = mmr.get_sparse_roots().await?;
                            let res = ApiResponse::AddBlock(sparse_roots);
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to AddBlock request"))?;
                        }
                        ApiRequestBody::GenerateBlockProof(_) => {
                            let res = ApiResponse::GenerateBlockProof();
                            req.tx_response.send(res).map_err(|_| anyhow::anyhow!("Failed to send response to GenerateBlockProof request"))?;
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
            ApiResponse::GetBlockCount(block_count) => Ok(block_count),
            _ => Err(anyhow::anyhow!(
                "Unexpected response to GetBlockCount request"
            )),
        }
    }

    pub async fn add_block(&self, block_header: BlockHeader) -> Result<Vec<String>, anyhow::Error> {
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
            ApiResponse::AddBlock(sparse_roots) => Ok(sparse_roots),
            _ => Err(anyhow::anyhow!("Unexpected response to AddBlock request")),
        }
    }

    pub async fn generate_block_proof(&self, block_height: u32) -> Result<(), anyhow::Error> {
        let (tx_response, rx_response) = oneshot::channel();
        self.tx_requests
            .send(ApiRequest {
                body: ApiRequestBody::GenerateBlockProof(block_height),
                tx_response,
            })
            .await?;
        rx_response
            .await
            .map_err(|_| anyhow::anyhow!("Failed to generate block proof"))?;
        Ok(())
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
