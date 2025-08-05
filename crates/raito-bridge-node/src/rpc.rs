use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use tower_http::trace::TraceLayer;

use crate::{app::AppClient, mmr::InclusionProof};

pub struct RpcConfig {
    pub rpc_host: String,
}

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
            .route("/proof/:height", get(generate_proof))
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

pub async fn generate_proof(
    State(app_client): State<AppClient>,
    Path(height): Path<u32>,
) -> Result<Json<InclusionProof>, StatusCode> {
    let proof = app_client
        .generate_block_proof(height)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(proof))
}

pub async fn get_head(State(app_client): State<AppClient>) -> Result<Json<u32>, StatusCode> {
    let block_count = app_client
        .get_block_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(block_count))
}
