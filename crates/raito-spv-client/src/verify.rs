use std::path::PathBuf;

#[derive(Clone, Debug, clap::Args)]
pub struct VerifyArgs {
    /// Path to read the proof from
    #[arg(long)]
    proof_path: PathBuf,
}

pub async fn run(args: VerifyArgs) -> Result<(), anyhow::Error> {
    Ok(())
}
