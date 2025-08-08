#![doc = include_str!("../README.md")]

use clap::{command, Parser, Subcommand};
use tracing::{error, info, subscriber::set_global_default};
use tracing_subscriber::filter::EnvFilter;

mod fetch;
mod proof;
mod verify;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Logging level (off, error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Fetch a compressed proof
    Fetch(fetch::FetchArgs),
    Verify(verify::VerifyArgs),
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

    let res = match cli.command {
        Commands::Fetch(args) => fetch::run(args).await,
        Commands::Verify(args) => verify::run(args).await,
    };

    match res {
        Ok(_) => {
            info!("Raito client has exited successfully");
            std::process::exit(0);
        }
        Err(err) => {
            error!("Raito client has exited with error: {}", err);
            std::process::exit(1);
        }
    }
}
