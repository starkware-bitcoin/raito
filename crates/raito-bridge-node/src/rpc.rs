use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::app::AppClient;

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
        let app = Router::new()
            .route("/broadcast", post(generate_proof))
            .route("/head", get(get_head))
            .with_state(self.app_client.clone());

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

pub async fn generate_proof(State(app_client): State<AppClient>) -> Result<Json<()>, StatusCode> {
    Ok(Json(()))
}

pub async fn get_head(State(app_client): State<AppClient>) -> Result<Json<u32>, StatusCode> {
    let block_count = app_client
        .get_block_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(block_count))
}
