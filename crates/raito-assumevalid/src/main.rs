use anyhow::Result;
use clap::{Parser, Subcommand};
use raito_assumevalid::{
    generate_args::{AssumeValidParams, ProveClient, ProveConfig, generate_and_save_args},
    prove::{ProveBatchParams, prove_batch, ProveParams, prove, auto_detect_start_height},
};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber;

/// Raito AssumeValid - Generate assumevalid arguments and prove Cairo programs
#[derive(Parser)]
#[command(name = "raito-assumevalid")]
#[command(about = "Generate assumevalid arguments and prove Cairo programs")]
#[command(version)]
struct Cli {
    /// Bridge node RPC URL
    #[arg(long, default_value = "https://api.raito.wtf/")]
    bridge_url: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Prove multiple batches iteratively (similar to prove_pow in Python)
    Prove {
        /// Starting block height (if not set, will auto-detect from last proof)
        #[arg(long)]
        start_height: Option<u32>,

        /// Total number of blocks to process
        #[arg(long, default_value = "1")]
        total_blocks: u32,

        /// Step size for each batch
        #[arg(long, default_value = "1")]
        step_size: u32,

        /// Output directory for all proofs
        #[arg(long, default_value = ".proofs")]
        output_dir: PathBuf,

        /// Path to the Cairo executable JSON file
        #[arg(long, default_value = "target/proving/assumevalid.executable.json")]
        executable: PathBuf,

        /// Path to the bootloader JSON file
        #[arg(long, default_value = "bootloaders/simple_bootloader_compiled.json")]
        bootloader: PathBuf,

        /// Path to the prover parameters JSON file
        #[arg(long, default_value = "packages/assumevalid/prover_params.json")]
        prover_params: PathBuf,

        /// Don't delete temporary files after completion
        #[arg(long)]
        keep_temp_files: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = match cli.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    // Create client
    let config = ProveConfig {
        bridge_node_url: cli.bridge_url.clone(),
    };
    let client = ProveClient::new(config);

    match cli.command {
        Commands::Prove {
            start_height,
            total_blocks,
            step_size,
            output_dir,
            executable,
            bootloader,
            prover_params,
            keep_temp_files
        } => {
            // Auto-detect start height if not provided
            let start_height = if let Some(height) = start_height {
                height
            } else {
                let detected = auto_detect_start_height(&output_dir);
                info!("Auto-detected start height: {}", detected);
                detected
            };

            info!("Starting iterative proving: start_height={}, total_blocks={}, step_size={}", 
                  start_height, total_blocks, step_size);
            info!("Output directory: {}", output_dir.display());

            let params = ProveParams {
                start_height,
                total_blocks,
                step_size,
                bridge_url: cli.bridge_url.clone(),
                output_dir: output_dir.clone(),
                executable,
                bootloader,
                prover_params,
                keep_temp_files,
            };

            let final_proof_path = prove(params).await?;

            info!("Iterative proving completed successfully!");
            info!("Final proof saved to: {}", final_proof_path.display());
        }
    }

    Ok(())
}