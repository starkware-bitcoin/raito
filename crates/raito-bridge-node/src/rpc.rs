//! HTTP RPC server providing REST endpoints for MMR proof generation and block count queries.

use raito_spv_verify::CompressedSpvProof;
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
use std::str::FromStr;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use bitcoin::block::Header as BlockHeader;
use raito_spv_client::fetch::fetch_compressed_proof;
use raito_spv_mmr::{block_mmr::BlockInclusionProof, sparse_roots::SparseRoots};
use raito_spv_verify::TransactionInclusionProof;

use crate::app::AppClient;

/// Query parameters for block inclusion proof generation and roots retrieval
#[derive(Debug, Deserialize)]
pub struct ChainHeightQuery {
    pub chain_height: Option<u32>,
}

/// Configuration for the RPC server
#[derive(Clone)]
pub struct RpcConfig {
    /// Host and port binding for the RPC server (e.g., "127.0.0.1:5000")
    pub rpc_host: String,
    /// Bitcoin RPC URL
    pub bitcoin_rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    pub bitcoin_rpc_userpwd: Option<String>,
    /// Raito RPC URL, where the /chainstate-proof/recent_proof endpoint is available
    pub raito_rpc_url: String,
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

        let inclusion = Router::new()
            .route("/block-inclusion-proof/:block_height", get(generate_proof))
            .route("/head", get(get_head))
            .route("/roots", get(get_roots))
            .route("/transaction-proof/:tx_id", get(get_transaction_proof))
            .route("/block-header/:block_height", get(get_block_header))
            .with_state(self.app_client.clone())
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        let compressed = Router::new()
            .route("/compressed_spv_proof/:tx_id", get(get_compressed_proof))
            .with_state(self.config.clone())
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        let app = Router::new().merge(inclusion).merge(compressed);

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
/// * `block_height` - The block height to generate a proof for
/// * `chain_height` - The chain (MMR) height to generate a proof for (optional)
///
/// # Returns
/// * `Json<InclusionProof>` - The inclusion proof in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If proof generation fails
pub async fn generate_proof(
    State(app_client): State<AppClient>,
    Path(block_height): Path<u32>,
    Query(query): Query<ChainHeightQuery>,
) -> Result<Json<BlockInclusionProof>, StatusCode> {
    let proof = app_client
        .generate_block_proof(block_height, query.chain_height)
        .await
        .map_err(|e| {
            error!(
                "Failed to generate block proof for height {}: {}",
                block_height, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(proof))
}

/// Get the roots of the MMR: latest or for a given block count (optional)
///
/// # Arguments
/// * `chain_height` - The chain (MMR) height to get the roots for (optional)
///
/// # Returns
/// * `Json<SparseRoots>` - The sparse roots in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If getting roots fails
pub async fn get_roots(
    State(app_client): State<AppClient>,
    Query(query): Query<ChainHeightQuery>,
) -> Result<Json<SparseRoots>, StatusCode> {
    let sparse_roots = app_client
        .get_sparse_roots(query.chain_height)
        .await
        .map_err(|e| {
            error!(
                "Failed to get sparse roots for chain height {:?}: {}",
                query.chain_height, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(sparse_roots))
}

/// Get the current head (latest processed block height) from the MMR
///
/// # Returns
/// * `Json<u32>` - The current block count in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If getting block count fails
pub async fn get_head(State(app_client): State<AppClient>) -> Result<Json<u32>, StatusCode> {
    let block_count = app_client.get_block_count().await.map_err(|e| {
        error!("Failed to get block count: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(block_count - 1))
}

/// Get a block header by block height
///
/// # Arguments
/// * `block_height` - The block height to get the header for
///
/// # Returns
/// * `Json<BlockHeader>` - The block header in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If fetching the block header fails
pub async fn get_block_header(
    State(app_client): State<AppClient>,
    Path(block_height): Path<u32>,
) -> Result<Json<BlockHeader>, StatusCode> {
    let block_header = app_client
        .get_block_header(block_height)
        .await
        .map_err(|e| {
            error!(
                "Failed to get block header for height {}: {}",
                block_height, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(block_header))
}

/// Get a compressed SPV proof for a transaction in a specific block
///
/// # Returns
/// * `Json<CompressedSpvProof>` - The compressed SPV proof in JSON format
/// * `StatusCode::BAD_REQUEST` - If the transaction ID is invalid
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If proof generation fails
pub async fn get_compressed_proof(
    State(config): State<RpcConfig>,
    Path(tx_id): Path<String>,
) -> Result<Json<CompressedSpvProof>, StatusCode> {
    let txid = bitcoin::Txid::from_str(&tx_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    // Call the fetch_compressed_proof function
    let compressed_proof = fetch_compressed_proof(
        txid,
        config.bitcoin_rpc_url,
        config.bitcoin_rpc_userpwd,
        config.raito_rpc_url,
        false,
    )
    .await
    .map_err(|e| {
        error!("Failed to fetch compressed proof for txid {}: {}", tx_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(compressed_proof))
}

/// Get a transaction inclusion proof for a specific transaction
///
/// # Returns
/// * `Json<TransactionInclusionProof>` - The transaction inclusion proof in JSON format
/// * `StatusCode::BAD_REQUEST` - If the transaction ID is invalid
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If proof generation fails
pub async fn get_transaction_proof(
    State(app_client): State<AppClient>,
    Path(tx_id): Path<String>,
) -> Result<Json<TransactionInclusionProof>, StatusCode> {
    let txid = bitcoin::Txid::from_str(&tx_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let transaction_proof = app_client.get_transaction_proof(txid).await.map_err(|e| {
        error!(
            "Failed to fetch transaction proof for txid {}: {}",
            tx_id, e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(transaction_proof))
}
