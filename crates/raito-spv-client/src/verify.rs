//! Verification routines for compressed SPV proofs, including transaction, block MMR,
//! Cairo recursive proof, and subchain work checks.

use bitcoin::Network;
use bitcoin::{block::Header as BlockHeader, consensus, MerkleBlock, Transaction};
use cairo_air::utils::{get_verification_output, VerificationOutput};
use cairo_air::{CairoProof, PreProcessedTraceVariant};
use raito_spv_core::block_mmr::{BlockInclusionProof, BlockMMR};
use std::path::PathBuf;
use stwo_prover::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};
use tracing::info;

use crate::format::format_transaction;
use crate::proof::{BootloaderOutput, ChainState, CompressedSpvProof, TaskResult};
use crate::work::verify_subchain_work;

/// CLI arguments for the `verify` subcommand
#[derive(Clone, Debug, clap::Args)]
pub struct VerifyArgs {
    /// Path to read the proof from
    #[arg(long)]
    proof_path: PathBuf,
    /// Development mode
    #[arg(long, default_value = "false")]
    dev: bool,
}

/// Configuration parameters controlling verification policies
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// Minimum cumulative work required on top of the target block (decimal string)
    pub min_work: String,
    /// Expected bootloader program hash used to generate the recursive proof (hex string)
    pub bootloader_hash: String,
    /// Expected payload program hash verified by the bootloader (hex string)
    pub task_program_hash: String,
    /// Expected size of the payload program output in felts
    pub task_output_size: u32,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            min_work: "1813388729421943762059264".to_string(), // 6 * 2^78, i.e. six block confirmations given the latest difficulty
            bootloader_hash: "0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3"
                .to_string(),
            task_program_hash: "0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca"
                .to_string(),
            task_output_size: 8,
        }
    }
}

/// Run the `verify` subcommand: read a proof from disk and verify it
pub async fn run(args: VerifyArgs) -> Result<(), anyhow::Error> {
    // Try to load the proof from the specified path
    let proof_file = std::fs::File::open(args.proof_path)?;
    let proof: CompressedSpvProof = serde_brief::from_reader(proof_file)?;

    let config = VerifierConfig::default();

    // Verify the proof
    verify_proof(proof, &config, args.dev).await?;

    Ok(())
}

/// Verify a compressed SPV proof end-to-end.
///
/// This checks transaction inclusion, block header inclusion in the block MMR,
/// Cairo recursive proof validity, and sufficient subchain work.
pub async fn verify_proof(
    proof: CompressedSpvProof,
    config: &VerifierConfig,
    dev: bool,
) -> Result<(), anyhow::Error> {
    let CompressedSpvProof {
        chain_state,
        chain_state_proof,
        block_header,
        block_header_proof,
        transaction,
        transaction_proof,
    } = proof;

    // Sanity checks
    if !dev && block_header_proof.leaf_count as u32 != chain_state.block_height + 1 {
        anyhow::bail!("Mismatched chain height and MMR size");
    }

    let block_height = block_header_proof.leaf_index as u32;

    info!("Verifying transaction inclusion proof ...");
    verify_transaction(&transaction, &block_header, transaction_proof)?;

    info!("Verifying block inclusion proof ...");
    let block_mmr_root_0 = verify_block_header(&block_header, block_header_proof).await?;

    info!("Verifying chain state proof ...");
    let block_mmr_hash_1 = verify_chain_state(&chain_state, chain_state_proof, &config)?;

    if !dev && block_mmr_root_0 != block_mmr_hash_1 {
        anyhow::bail!("Mismatched block MMR roots");
    }

    info!("Verifying subchain work ...");
    verify_subchain_work(block_height, &chain_state, &config)?;

    info!("Verification successful!");

    // Format and display the transaction with ASCII graphics
    let formatted_tx = format_transaction(
        &transaction,
        Network::Bitcoin,
        &block_header,
        block_height,
        chain_state.block_height,
    );
    println!("{}", formatted_tx);

    Ok(())
}

/// Verify that `transaction` is included in `block_header` using the provided Merkle proof.
pub fn verify_transaction(
    transaction: &Transaction,
    block_header: &BlockHeader,
    transaction_proof: Vec<u8>,
) -> anyhow::Result<()> {
    let merkle_block = MerkleBlock {
        header: block_header.clone(),
        txn: consensus::deserialize(&transaction_proof)?,
    };

    let mut matches = Vec::new();
    let mut indexes = Vec::new();
    merkle_block.extract_matches(&mut matches, &mut indexes)?;

    if matches.len() != 1 {
        anyhow::bail!("Expected 1 transaction match");
    }

    let txid = transaction.compute_txid();
    if txid != matches[0] {
        anyhow::bail!("Transaction ID mismatch");
    }

    Ok(())
}

/// Verify that `block_header` is included in the block MMR using the supplied inclusion proof.
///
/// Returns the computed block MMR root on success.
pub async fn verify_block_header(
    block_header: &BlockHeader,
    block_header_proof: BlockInclusionProof,
) -> anyhow::Result<String> {
    let BlockInclusionProof {
        peaks_hashes,
        siblings_hashes: _,
        leaf_index: _,
        leaf_count,
    } = block_header_proof.clone();
    let mmr = BlockMMR::from_peaks(peaks_hashes, leaf_count).await?;
    mmr.verify_proof(block_header, block_header_proof).await?;
    mmr.get_root_hash(None).await
}

/// Verify the Cairo recursive proof and consistency of the bootloader output with `chain_state`.
///
/// Returns the block MMR root extracted from the proof on success.
pub fn verify_chain_state(
    chain_state: &ChainState,
    chain_state_proof: CairoProof<Blake2sMerkleHasher>,
    config: &VerifierConfig,
) -> anyhow::Result<String> {
    info!("Extracting verification output...");

    // Extract verification output from the public memory
    let VerificationOutput {
        program_hash: bootloader_hash,
        output,
    } = get_verification_output(&chain_state_proof.claim.public_data.public_memory);

    // Decode the bootloader hash
    let bootloader_hash = format!("0x{}", hex::encode(&bootloader_hash.to_bytes_be()));

    // Decode bootloader output from the raw output felts
    let BootloaderOutput {
        n_tasks,
        task_output_size,
        task_program_hash,
        task_result,
    } = BootloaderOutput::decode(output)?;

    if n_tasks != 1 {
        anyhow::bail!(
            "Bootloader output: number of tasks must be 1, got {}",
            n_tasks
        );
    }
    if task_output_size != config.task_output_size {
        anyhow::bail!(
            "Bootloader output: task output size must be {}, got {}",
            config.task_output_size,
            task_output_size
        );
    }

    let TaskResult {
        chain_state_hash,
        block_mmr_hash,
        program_hash: prev_program_hash,
        bootloader_hash: prev_bootloader_hash,
    } = task_result.clone();

    // Check that chain state hashes match
    let expected_chain_state_hash = chain_state.blake2s_digest()?;
    if chain_state_hash != expected_chain_state_hash {
        anyhow::bail!(
            "Chain state hash doesn't match the expected hash: {} != {}",
            chain_state_hash,
            expected_chain_state_hash
        );
    }

    // Check that the program hash is the same as in the bootloader output and as expected
    if task_program_hash != config.task_program_hash {
        anyhow::bail!(
            "Bootloader output: task program hash doesn't match the expected hash: {} != {}",
            task_program_hash,
            config.task_program_hash
        );
    }
    if task_program_hash != prev_program_hash {
        anyhow::bail!(
            "Previous program hash doesn't match the task result: {} != {}",
            prev_program_hash,
            task_program_hash
        );
    }

    // Check that the previous bootloader hash is the same as in the Cairo claim and as expected
    if bootloader_hash != config.bootloader_hash {
        anyhow::bail!(
            "Bootloader hash doesn't match the expected hash: {} != {}",
            bootloader_hash,
            config.bootloader_hash
        );
    }
    if bootloader_hash != prev_bootloader_hash {
        anyhow::bail!(
            "Previous bootloader hash doesn't match the verification data: {} != {}",
            bootloader_hash,
            prev_bootloader_hash
        );
    }

    info!("Verifying Cairo proof...");
    cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
        chain_state_proof,
        PreProcessedTraceVariant::CanonicalWithoutPedersenAndPoseidon,
    )?;

    Ok(block_mmr_hash)
}
