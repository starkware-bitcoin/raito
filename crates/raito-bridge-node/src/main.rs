#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use clap::{command, Parser};
use tokio::task::JoinHandle;
use tracing::{error, info, subscriber::set_global_default};
use tracing_subscriber::filter::EnvFilter;

use crate::{
    indexer::{Indexer, IndexerConfig},
    shutdown::Shutdown,
    sparse_roots::SparseRootsSinkConfig,
};

mod bitcoin;
mod indexer;
mod mmr;
mod shutdown;
mod sparse_roots;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Bitcoin RPC URL
    #[arg(long, env = "BITCOIN_RPC")]
    rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    #[arg(long, env = "USERPWD")]
    rpc_userpwd: Option<String>,
    #[arg(long, default_value = "./.mmr_data/mmr.db")]
    mmr_db_path: PathBuf,
    /// Output directory for sparse roots JSON files
    #[arg(long, default_value = "./.mmr_data/roots")]
    mmr_roots_dir: String,
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

    let shutdown = Shutdown::default();

    let indexer_config = IndexerConfig {
        rpc_url: cli.rpc_url,
        rpc_userpwd: cli.rpc_userpwd,
        sink_config: SparseRootsSinkConfig {
            output_dir: cli.mmr_roots_dir,
            shard_size: cli.mmr_shard_size,
        },
        mmr_db_path: cli.mmr_db_path,
    };
    let mut indexer = Indexer::new(indexer_config, shutdown.subscribe());

    let indexer_handle = tokio::spawn(async move { indexer.run().await });
    let shutdown_handle = tokio::spawn(async move { shutdown.run().await });

    match tokio::try_join!(flatten(indexer_handle), flatten(shutdown_handle)) {
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
