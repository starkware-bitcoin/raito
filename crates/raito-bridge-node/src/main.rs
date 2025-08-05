#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use clap::{command, Parser};
use tokio::task::JoinHandle;
use tracing::{error, info, subscriber::set_global_default};
use tracing_subscriber::filter::EnvFilter;

use crate::{
    app::{create_app, AppConfig},
    indexer::{Indexer, IndexerConfig},
    rpc::{RpcConfig, RpcServer},
    shutdown::Shutdown,
    sparse_roots::SparseRootsSinkConfig,
};

mod app;
mod bitcoin;
mod indexer;
mod mmr;
mod rpc;
mod shutdown;
mod sparse_roots;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// RPC server host
    #[arg(long, default_value = "127.0.0.1:5000")]
    rpc_host: String,
    /// Bitcoin RPC URL
    #[arg(long, env = "BITCOIN_RPC")]
    bitcoin_rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    #[arg(long, env = "USERPWD")]
    bitcoin_rpc_userpwd: Option<String>,
    /// Path to the database storing the MMR accumulator state
    #[arg(long, default_value = "./.mmr_data/mmr.db")]
    mmr_db_path: PathBuf,
    /// Output directory for sparse roots JSON files
    #[arg(long, default_value = "./.mmr_data/roots")]
    mmr_roots_dir: PathBuf,
    /// Number of blocks per sparse roots shard directory
    #[arg(long, default_value = "10000")]
    mmr_shard_size: u32,
    /// Logging level (off, error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn init_tracing(log_level: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let subscriber_builder =
        tracing_subscriber::fmt::Subscriber::builder().with_env_filter(env_filter);

    let subscriber = subscriber_builder.with_writer(std::io::stderr).finish();
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    init_tracing(&cli.log_level);

    info!("Raito bridge node is launching...");

    // Instantiating components and wiring them together
    let shutdown = Shutdown::default();

    let app_config = AppConfig {
        mmr_db_path: cli.mmr_db_path,
        api_requests_capacity: 1000,
    };
    let (mut app_server, app_client) = create_app(app_config, shutdown.subscribe());

    let indexer_config = IndexerConfig {
        rpc_url: cli.bitcoin_rpc_url,
        rpc_userpwd: cli.bitcoin_rpc_userpwd,
        sink_config: SparseRootsSinkConfig {
            output_dir: cli.mmr_roots_dir,
            shard_size: cli.mmr_shard_size,
        },
    };
    let mut indexer = Indexer::new(indexer_config, app_client.clone(), shutdown.subscribe());

    let rpc_config = RpcConfig {
        rpc_host: cli.rpc_host,
    };
    let rpc_server = RpcServer::new(rpc_config, app_client.clone(), shutdown.subscribe());

    // Launching threads for each component
    let app_handle = tokio::spawn(async move { app_server.run().await });
    let indexer_handle = tokio::spawn(async move { indexer.run().await });
    let rpc_handle = tokio::spawn(async move { rpc_server.run().await });
    let shutdown_handle = tokio::spawn(async move { shutdown.run().await });

    // If at least one component exits with an error, the node will exit with an error
    match tokio::try_join!(
        flatten(app_handle),
        flatten(indexer_handle),
        flatten(rpc_handle),
        flatten(shutdown_handle)
    ) {
        Ok(_) => {
            info!("Raito bridge node has shut down");
            std::process::exit(0);
        }
        Err(_) => {
            error!("Raito bridge node has exited with error");
            std::process::exit(1);
        }
    }
}

async fn flatten<T>(handle: JoinHandle<Result<T, ()>>) -> Result<T, ()> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(()),
    }
}
