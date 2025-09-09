//! CLI wrapper for the verify functionality

use bitcoin::Network;
use clap::Args;
use std::path::PathBuf;

use crate::fetch::load_compressed_proof_from_bzip2;
use crate::format::format_transaction;
use raito_spv_verify::{verify_proof, CompressedSpvProof, VerifierConfig};

/// CLI arguments for the `verify` subcommand
#[derive(Clone, Debug, Args)]
pub struct VerifyArgs {
    /// Path to read the proof from
    #[arg(long)]
    proof_path: PathBuf,
    /// Development mode
    #[arg(long, default_value = "false")]
    dev: bool,
}

/// Run the `verify` subcommand: read a proof from disk and verify it
pub async fn run(args: VerifyArgs) -> Result<(), anyhow::Error> {
    // Load the compressed proof from the bzip2 compressed file
    let proof: CompressedSpvProof = load_compressed_proof_from_bzip2(&args.proof_path)?;

    let config = VerifierConfig::default();

    // Extract variables needed for formatting
    let transaction = proof.transaction.clone();
    let block_header = proof.block_header.clone();
    let chain_state_block_height = proof.chain_state.block_height;
    let block_height = proof.block_header_proof.leaf_index as u32;

    // Verify the proof
    verify_proof(proof, &config, args.dev).await?;

    // Format and display the transaction with ASCII graphics
    let formatted_tx = format_transaction(
        &transaction,
        Network::Bitcoin,
        &block_header,
        block_height,
        chain_state_block_height,
    );
    println!("{}", formatted_tx);

    Ok(())
}
