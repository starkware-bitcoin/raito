use std::path::PathBuf;

/// CLI arguments for the `verify` subcommand
#[derive(Clone, Debug, clap::Args)]
pub struct VerifyArgs {
    /// Path to read the proof from
    #[arg(long)]
    proof_path: PathBuf,
}

/// Run the `verify` subcommand: read a proof from disk and verify it
///
/// Currently a stub. Returns `Ok(())` as a placeholder.
pub async fn run(args: VerifyArgs) -> Result<(), anyhow::Error> {
    Ok(())
}
