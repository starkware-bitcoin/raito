#![doc = include_str!("../README.md")]

use bitcoin::{MerkleBlock, Txid};
use clap::{command, Parser};
use raito_spv_core::bitcoin::BitcoinClient;
use std::str::FromStr;
use tracing::subscriber::set_global_default;
use tracing_subscriber::filter::EnvFilter;

mod proof;
mod verifier;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Bridge node RPC URL
    #[arg(long, default_value = "127.0.0.1:5000")]
    bridge_rpc_url: String,
    /// Bitcoin RPC URL
    #[arg(long, env = "BITCOIN_RPC")]
    bitcoin_rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    #[arg(long, env = "USERPWD")]
    bitcoin_rpc_userpwd: Option<String>,
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

    let bitcoin_client = BitcoinClient::new(cli.bitcoin_rpc_url, cli.bitcoin_rpc_userpwd).unwrap();
    let txid =
        Txid::from_str("46954558cd3f07ffcdd4befe304cc6fe15b96633dff20ab3a989676061cccd10").unwrap();
    let MerkleBlock { header, txn } = bitcoin_client
        .get_transaction_inclusion_proof(&txid)
        .await
        .unwrap();

    let block_hash = header.block_hash();
    let transaction = bitcoin_client
        .get_transaction(&txid, &block_hash)
        .await
        .unwrap();

    let block_header_ex = bitcoin_client
        .get_block_header_ex(&block_hash)
        .await
        .unwrap();
    let block_height = block_header_ex.height;

    println!("Block height: {}", block_height);
    println!("Block hash: {}", block_hash);
    println!("Transaction: {:?}", transaction);
}
