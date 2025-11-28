use anyhow::Result;
use clap::{Parser, Subcommand};
use raito_assumevalid::prove::{prove, ProveParams};
use std::path::PathBuf;
use tracing_subscriber::{self, EnvFilter};

/// Raito AssumeValid - Generate assumevalid arguments and prove Cairo programs
#[derive(Parser)]
#[command(name = "raito-assumevalid")]
#[command(about = "Generate assumevalid arguments and prove Cairo programs")]
#[command(version)]
struct Cli {
    /// Bridge node RPC URL
    #[arg(long, default_value = "https://staging.raito.wtf")]
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
        /// Use cloud storage to detect latest proof height instead of local directory scanning
        #[arg(long)]
        load_from_gcs: bool,

        #[arg(long)]
        save_to_gcs: bool,

        #[arg(long, default_value = "raito-proofs")]
        gcs_bucket: String,

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
        #[arg(
            long,
            default_value = "crates/raito-assumevalid/compiled/assumevalid-syscalls.executable.json"
        )]
        executable: PathBuf,

        /// Path to the prover parameters JSON file
        #[arg(long)]
        prover_params_file: Option<PathBuf>,

        /// Don't delete temporary files after completion
        #[arg(long, default_value = "false")]
        keep_temp_files: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging - validate and normalize the log level
    let base_level = match cli.log_level.as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => cli.log_level.as_str(),
        _ => "info",
    };

    // Build an EnvFilter with per-target overrides to silence noisy dependencies.
    // Always merge our suppressions even if RUST_LOG is set.
    let mut env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(base_level));

    for directive in [
        "gcp_auth::custom_service_account=off",
        "gcp_auth::authentication_manager=off",
        "reqwest::connect=off",
        "hyper_util::client::legacy::connect::http=off",
        "hyper::client::connect::dns=off",
        "rustls::client=off",
        // Reduce chattiness of h2/hyper/reqwest to info+ regardless of base debug level
        "h2=info",
        "hyper=info",
        "hyper_util=info",
        "reqwest=info",
    ] {
        if let Ok(dir) = directive.parse() {
            env_filter = env_filter.add_directive(dir);
        }
    }

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    match cli.command {
        Commands::Prove {
            executable,
            load_from_gcs,
            save_to_gcs,
            gcs_bucket,
            total_blocks,
            step_size,
            output_dir,
            prover_params_file,
            keep_temp_files,
        } => {
            let params = ProveParams {
                executable,
                load_from_gcs,
                save_to_gcs,
                gcs_bucket,
                bridge_url: cli.bridge_url,
                total_blocks,
                step_size,
                output_dir,
                prover_params_file,
                keep_temp_files,
            };

            prove(params).await?;
        }
    }

    Ok(())
}
