//! HTTP RPC server providing REST endpoints for MMR proof generation and block count queries.

use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

use raito_spv_core::block_mmr::BlockInclusionProof;

use crate::app::AppClient;

/// Query parameters for block inclusion proof generation
#[derive(Debug, Deserialize)]
pub struct BlockProofQuery {
    pub block_count: Option<u32>,
}

/// Configuration for the RPC server
pub struct RpcConfig {
    /// Host and port binding for the RPC server (e.g., "127.0.0.1:5000")
    pub rpc_host: String,
}

/// HTTP RPC server that provides endpoints for MMR operations
pub struct RpcServer {
    config: RpcConfig,
    app_client: AppClient,
    rx_shutdown: broadcast::Receiver<()>,
}

impl RpcServer {
    pub fn new(
        config: RpcConfig,
        app_client: AppClient,
        rx_shutdown: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            config,
            app_client,
            rx_shutdown,
        }
    }

    async fn run_inner(&self) -> Result<(), std::io::Error> {
        info!("Starting RPC server on {}", self.config.rpc_host);

        let app = Router::new()
            .route("/block-inclusion-proof/:height", get(generate_proof))
            .route("/head", get(get_head))
            .with_state(self.app_client.clone())
            .layer(TraceLayer::new_for_http());

        let listener = TcpListener::bind(&self.config.rpc_host).await?;
        let mut rx_shutdown = self.rx_shutdown.resubscribe();

        axum::serve(listener, app)
            .with_graceful_shutdown(async move { rx_shutdown.recv().await.unwrap_or_default() })
            .await
    }

    pub async fn run(&self) -> Result<(), ()> {
        match self.run_inner().await {
            Err(err) => {
                error!("RPC server exited: {}", err);
                Err(())
            }
            Ok(()) => {
                info!("RPC server terminated");
                Ok(())
            }
        }
    }
}

/// Generate an inclusion proof for a block at the specified height
///
/// # Arguments
/// * `height` - The block height to generate a proof for
///
/// # Returns
/// * `Json<InclusionProof>` - The inclusion proof in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If proof generation fails
pub async fn generate_proof(
    State(app_client): State<AppClient>,
    Path(height): Path<u32>,
    Query(query): Query<BlockProofQuery>,
) -> Result<Json<BlockInclusionProof>, StatusCode> {
    let proof = app_client
        .generate_block_proof(height, query.block_count)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(proof))
}

/// Get the current head (latest block count) from the MMR
///
/// # Returns
/// * `Json<u32>` - The current block count in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If getting block count fails
pub async fn get_head(State(app_client): State<AppClient>) -> Result<Json<u32>, StatusCode> {
    let block_count = app_client
        .get_block_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(block_count))
}
