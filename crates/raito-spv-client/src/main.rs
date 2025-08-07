#![doc = include_str!("../README.md")]

use clap::{command, Parser};
use tracing::subscriber::set_global_default;
use tracing_subscriber::filter::EnvFilter;

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
}
