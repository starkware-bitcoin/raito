//! HTTP RPC server providing REST endpoints for MMR proof generation and block count queries.

use accumulators::hasher::stark_blake::StarkBlakeHasher;
use raito_bitcoin_client::BitcoinClient;
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
use std::{path::PathBuf, str::FromStr, sync::Arc};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use bitcoin::{block::Header as BlockHeader, consensus, MerkleBlock};
use raito_spv_mmr::{
    block_mmr::{BlockInclusionProof, BlockMMR},
    sparse_roots::SparseRoots,
};
use raito_spv_verify::{ChainState, TransactionInclusionProof};

use crate::{chain_state::ChainStateStore, store::AppStore};

/// Query parameters for block inclusion proof generation and roots retrieval
#[derive(Debug, Deserialize)]
pub struct ChainHeightQuery {
    pub chain_height: Option<u32>,
}

/// Query parameters for block headers retrieval
#[derive(Debug, Deserialize)]
pub struct BlockHeadersQuery {
    pub offset: Option<u32>,
    pub size: Option<u32>,
}

/// Configuration for the RPC server
#[derive(Clone)]
pub struct RpcConfig {
    /// Host and port binding for the RPC server (e.g., "127.0.0.1:5000")
    pub rpc_host: String,
    /// MMR ID
    pub mmr_id: String,
    /// Path to the database storing the MMR accumulator
    pub mmr_db_path: PathBuf,
    /// Bitcoin RPC URL
    pub rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    pub rpc_userpwd: Option<String>,
}

/// HTTP RPC server that provides endpoints for MMR operations
pub struct RpcServer {
    config: RpcConfig,
    rx_shutdown: broadcast::Receiver<()>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    mmr: Arc<BlockMMR>,
    store: Arc<AppStore>,
    bitcoin_client: Arc<BitcoinClient>,
}

impl AppState {
    pub fn new(config: RpcConfig) -> Result<Self, anyhow::Error> {
        let mmr_id = Some(config.mmr_id.clone());
        let store = Arc::new(AppStore::multiple_concurrent_readers(
            &config.mmr_db_path,
            mmr_id.clone(),
        ));
        let hasher = StarkBlakeHasher::default();
        let mmr = BlockMMR::new(store.clone(), Arc::new(hasher), mmr_id);
        let bitcoin_client =
            BitcoinClient::new(config.rpc_url.clone(), config.rpc_userpwd.clone())?;
        Ok(Self {
            mmr: Arc::new(mmr),
            bitcoin_client: Arc::new(bitcoin_client),
            store: store.clone(),
        })
    }
}

impl RpcServer {
    pub fn new(config: RpcConfig, rx_shutdown: broadcast::Receiver<()>) -> Self {
        Self {
            config,
            rx_shutdown,
        }
    }

    async fn run_inner(&self) -> Result<(), std::io::Error> {
        info!("Starting RPC server on {}", self.config.rpc_host);

        let app_state = AppState::new(self.config.clone())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let app = Router::new()
            .route("/block-inclusion-proof/:block_height", get(generate_proof))
            .route("/head", get(get_head))
            .route("/roots", get(get_roots))
            .route("/headers", get(get_block_headers))
            .route("/transaction-proof/:tx_id", get(get_transaction_proof))
            .route("/block-header/:block_height", get(get_block_header))
            .route("/chain-state/:block_height", get(get_chain_state))
            .with_state(app_state)
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
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
/// * `block_height` - The block height to generate a proof for
/// * `chain_height` - The chain (MMR) height to generate a proof for (optional)
///
/// # Returns
/// * `Json<InclusionProof>` - The inclusion proof in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If proof generation fails
pub async fn generate_proof(
    State(state): State<AppState>,
    Path(block_height): Path<u32>,
    Query(query): Query<ChainHeightQuery>,
) -> Result<Json<BlockInclusionProof>, StatusCode> {
    let proof = state
        .mmr
        .generate_proof(block_height, query.chain_height)
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
    State(state): State<AppState>,
    Query(query): Query<ChainHeightQuery>,
) -> Result<Json<SparseRoots>, StatusCode> {
    let sparse_roots = state
        .mmr
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
pub async fn get_head(State(state): State<AppState>) -> Result<Json<u32>, StatusCode> {
    let block_count = state.mmr.get_block_count().await.map_err(|e| {
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
    State(state): State<AppState>,
    Path(block_height): Path<u32>,
) -> Result<Json<BlockHeader>, StatusCode> {
    let block_header = state
        .store
        .get_block_headers(block_height, 1)
        .await
        .map_err(|e| {
            error!(
                "Failed to get block header for height {}: {}",
                block_height, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .pop()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(block_header))
}

/// Get a range of block headers from the MMR
///
/// # Arguments
/// * `offset` - The starting block height to get the headers for
/// * `size` - The number of blocks to get the headers for
/// # Returns
/// * `Json<Vec<BlockHeader>>>` - The block headers in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If fetching the block headers fails
pub async fn get_block_headers(
    State(state): State<AppState>,
    Query(query): Query<BlockHeadersQuery>,
) -> Result<Json<Vec<BlockHeader>>, StatusCode> {
    let offset = query.offset.unwrap_or(0);
    let size = query.size.unwrap_or(10);
    let block_headers = state
        .store
        .get_block_headers(offset, size)
        .await
        .map_err(|e| {
            error!(
                "Failed to get {} block headers for offset {}: {}",
                size, offset, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(block_headers))
}

/// Get a transaction inclusion proof for a specific transaction
///
/// # Returns
/// * `Json<TransactionInclusionProof>` - The transaction inclusion proof in JSON format
/// * `StatusCode::BAD_REQUEST` - If the transaction ID is invalid
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If proof generation fails
pub async fn get_transaction_proof(
    State(state): State<AppState>,
    Path(tx_id): Path<String>,
) -> Result<Json<TransactionInclusionProof>, StatusCode> {
    let txid = bitcoin::Txid::from_str(&tx_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let MerkleBlock {
        header: block_header,
        txn,
    } = state
        .bitcoin_client
        .get_transaction_inclusion_proof(&txid)
        .await
        .map_err(|e| {
            error!(
                "Failed to fetch transaction proof for txid {}: {}",
                tx_id, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let block_hash = block_header.block_hash();
    let block_height = state
        .store
        .get_block_height(&block_hash)
        .await
        .map_err(|e| {
            error!(
                "Failed to get block height for block hash {}: {}",
                block_hash, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let transaction = state
        .bitcoin_client
        .get_transaction(&txid, &block_hash)
        .await
        .map_err(|e| {
            error!("Failed to get transaction for txid {}: {}", tx_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let transaction_proof = TransactionInclusionProof {
        transaction,
        transaction_proof: consensus::encode::serialize(&txn),
        block_header,
        block_height,
    };
    Ok(Json(transaction_proof.into()))
}

/// Get the chain state for a specific block height
///
/// # Returns
/// * `Json<ChainState>` - The chain state in JSON format
/// * `StatusCode::INTERNAL_SERVER_ERROR` - If fetching the chain state fails
pub async fn get_chain_state(
    State(state): State<AppState>,
    Path(block_height): Path<u32>,
) -> Result<Json<ChainState>, StatusCode> {
    let chain_state = state
        .store
        .get_chain_state(block_height)
        .await
        .map_err(|e| {
            error!(
                "Failed to get chain state for height {}: {}",
                block_height, e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(chain_state))
}
