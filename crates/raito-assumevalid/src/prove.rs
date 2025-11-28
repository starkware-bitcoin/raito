//! Proving module for raito-assumevalid
//!
//! This module provides functionality to run Cairo programs and generate STARK proofs
//! using the stwo-cairo prover library directly.

use anyhow::{anyhow, Result};
use cairo_air::utils::{deserialize_proof_from_file, serialize_proof_to_file, ProofFormat};
use cairo_program_runner_lib::cairo_run_program;
use cairo_program_runner_lib::utils::get_cairo_run_config;
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use memory_stats::memory_stats;
use regex::Regex;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_adapter::adapter::adapt;
use stwo_cairo_prover::prover::create_and_serialize_proof;
use tracing::{debug, error, info, warn};

/// Get current memory usage in MB
fn get_memory_mb() -> f64 {
    memory_stats()
        .map(|usage| usage.physical_mem as f64 / (1024.0 * 1024.0))
        .unwrap_or(0.0)
}

use crate::gcs::{download_recent_proof_via_reqwest, upload_recent_proof, RecentProof};
use crate::generate_args::{generate_and_save_args, AssumeValidParams, ProveClient, ProveConfig};
use crate::BOOTLOADER_STR;

/// Runs the Cairo program through the bootloader and generates a STARK proof.
///
/// This function:
/// 1. Loads the bootloader program from embedded resources
/// 2. Generates the program input JSON for the bootloader
/// 3. Runs the Cairo VM
/// 4. Adapts the VM output for the prover
/// 5. Generates and serializes the STARK proof
pub fn run_and_prove_with_library(
    executable: &Path,
    arguments_file: &Path,
    output_dir: &Path,
    prover_params: Option<&Path>,
    verify: bool,
) -> Result<PathBuf> {
    let start_time = Instant::now();
    let start_mem = get_memory_mb();

    // Create output directory
    fs::create_dir_all(output_dir)?;

    // Resolve paths to absolute with helpful error messages
    let executable_abs = executable.canonicalize().map_err(|e| {
        anyhow!(
            "Executable file not found at '{}': {}",
            executable.display(),
            e
        )
    })?;
    let args_file = arguments_file.canonicalize().map_err(|e| {
        anyhow!(
            "Arguments file not found at '{}': {}",
            arguments_file.display(),
            e
        )
    })?;

    // Generate program input JSON for the bootloader
    let program_input = json!({
        "single_page": true,
        "tasks": [
            {
                "type": "Cairo1Executable",
                "path": executable_abs.to_string_lossy(),
                "program_hash_function": "blake",
                "user_args_file": args_file.to_string_lossy(),
            }
        ],
    });
    let program_input_str = serde_json::to_string(&program_input)?;

    // Load bootloader program from embedded resource
    let bootloader_program = Program::from_bytes(BOOTLOADER_STR.as_bytes(), Some("main"))
        .map_err(|e| anyhow!("Failed to load bootloader program: {}", e))?;

    // Configure Cairo VM for proof mode
    let cairo_run_config = get_cairo_run_config(
        &None,                      // no dynamic layout params
        LayoutName::all_cairo_stwo, // layout
        true,                       // proof_mode
        true,                       // disable_trace_padding (redundant in stwo proof mode)
        true,                       // allow_missing_builtins (bootloader simulates them)
        false,                      // relocate_mem (adapter does relocation)
    )?;

    // Run the bootloader with the program input
    debug!("Running Cairo VM...");
    let vm_start = Instant::now();
    let runner = cairo_run_program(
        &bootloader_program,
        Some(program_input_str),
        cairo_run_config,
    )
    .map_err(|e| anyhow!("Cairo VM execution failed: {}", e))?;
    let vm_elapsed = vm_start.elapsed();
    let vm_mem = get_memory_mb();
    info!(
        "Cairo VM: {:.2}s, memory: {:.1} MB",
        vm_elapsed.as_secs_f64(),
        vm_mem
    );

    // Adapt the VM output for the prover
    debug!("Adapting VM output for prover...");
    let adapt_start = Instant::now();
    let prover_input = adapt(&runner).map_err(|e| anyhow!("Failed to adapt VM output: {}", e))?;
    let adapt_elapsed = adapt_start.elapsed();
    let adapt_mem = get_memory_mb();
    info!(
        "Adapt: {:.2}s, memory: {:.1} MB",
        adapt_elapsed.as_secs_f64(),
        adapt_mem
    );

    // Generate the proof
    let proof_file = output_dir.join("proof.json");
    debug!("Generating STARK proof...");
    let prove_start = Instant::now();
    create_and_serialize_proof(
        prover_input,
        verify,
        proof_file.clone(),
        ProofFormat::CairoSerde,
        prover_params.map(|p| p.to_path_buf()),
    )
    .map_err(|e| anyhow!("Proof generation failed: {}", e))?;
    let prove_elapsed = prove_start.elapsed();
    let prove_mem = get_memory_mb();
    info!(
        "Prove: {:.2}s, memory: {:.1} MB",
        prove_elapsed.as_secs_f64(),
        prove_mem
    );

    let total_elapsed = start_time.elapsed();
    let peak_mem = prove_mem.max(adapt_mem).max(vm_mem);
    info!(
        "Total: {:.2}s, peak memory: {:.1} MB (started at {:.1} MB)",
        total_elapsed.as_secs_f64(),
        peak_mem,
        start_mem
    );

    Ok(proof_file)
}

/// Parameters for proving multiple batches iteratively
#[derive(Debug, Clone)]
pub struct ProveParams {
    /// Path to the Cairo1 executable JSON file
    pub executable: PathBuf,
    pub load_from_gcs: bool,
    pub save_to_gcs: bool,
    /// GCS bucket name for loading/saving proofs
    pub gcs_bucket: String,
    pub bridge_url: String,
    /// Total number of blocks to process
    pub total_blocks: u32,
    /// Step size for each batch
    pub step_size: u32,
    /// Output directory for all proofs
    pub output_dir: PathBuf,
    /// Path to the prover parameters JSON file (optional)
    pub prover_params_file: Option<PathBuf>,
    /// Whether to keep temporary files after completion
    pub keep_temp_files: bool,
}

/// Find the previous proof file for a given start height
pub fn find_proof_file(start_height: u32, output_dir: &Path) -> Option<PathBuf> {
    if start_height == 0 {
        return None;
    }

    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    if dir_name.ends_with(&format!("_to_{}", start_height)) {
                        let proof_file = entry.path().join("proof.json");
                        if proof_file.exists() {
                            return Some(proof_file);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Auto-detect the starting height by finding the highest ending height from existing proof directories
pub fn auto_detect_start_height(proof_dir: &Path) -> u32 {
    let mut max_height = 0;
    let pattern = Regex::new(r"batch_(\d+)_to_(\d+)").unwrap();

    if !proof_dir.exists() {
        return max_height;
    }

    if let Ok(entries) = fs::read_dir(proof_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    if let Some(captures) = pattern.captures(dir_name) {
                        if let (Ok(_start), Ok(end)) =
                            (captures[1].parse::<u32>(), captures[2].parse::<u32>())
                        {
                            let proof_file = entry.path().join("proof.json");
                            if proof_file.exists() && end > max_height {
                                max_height = end;
                            }
                        }
                    }
                }
            }
        }
    }
    max_height
}

pub async fn create_batch_dir(
    start_height: u32,
    step_size: u32,
    output_dir: &Path,
) -> Result<PathBuf> {
    let batch_name = format!("batch_{}_to_{}", start_height, start_height + step_size);
    let batch_dir = output_dir.join(&batch_name);
    tokio::fs::create_dir_all(&batch_dir).await?;
    Ok(batch_dir)
}

/// Main function to prove multiple batches iteratively
pub async fn prove(params: ProveParams) -> Result<()> {
    if params.load_from_gcs {
        let recent_proof = download_recent_proof_via_reqwest(params.gcs_bucket.as_str()).await?;
        let batch_dir =
            create_batch_dir(0, recent_proof.chainstate.block_height, &params.output_dir).await?;

        // Save the recent proof to the output directory
        let proof_file = batch_dir.join("proof.json");
        serialize_proof_to_file::<Blake2sMerkleHasher>(
            &recent_proof.proof,
            &proof_file,
            ProofFormat::CairoSerde,
        )?;

        debug!(
            "Using recent proof up to height: {}",
            recent_proof.chainstate.block_height
        );
    }

    let start_height = auto_detect_start_height(&params.output_dir);

    info!(
        "Starting iterative proving process: start_height={}, total_blocks={}, step_size={}",
        start_height, params.total_blocks, params.step_size
    );
    debug!("Output directory: {}", params.output_dir.display());

    // Create output directory
    tokio::fs::create_dir_all(&params.output_dir).await?;

    let end_height = start_height + params.total_blocks;
    let mut current_height = start_height;
    let mut last_proof_path: Option<PathBuf> = None;
    let mut last_height = start_height;

    // Process batches sequentially
    while current_height < end_height {
        let current_step = std::cmp::min(params.step_size, end_height - current_height);
        if current_step == 0 {
            break;
        }

        let job_info = format!("Job(height='{}', blocks={})", current_height, current_step);
        info!("{} proving...", job_info);

        let batch_dir = create_batch_dir(current_height, current_step, &params.output_dir).await?;

        // Look for previous proof
        let chain_state_proof_path = find_proof_file(current_height, &params.output_dir);

        // Generate arguments for this batch
        debug!("{} generating args...", job_info);
        let args_start_time = Instant::now();

        let config = ProveConfig {
            bridge_node_url: params.bridge_url.clone(),
        };
        let client = ProveClient::new(config);

        let assumevalid_params = AssumeValidParams {
            start_height: current_height,
            block_count: current_step,
            chain_state_proof_path,
        };

        let args_file = batch_dir.join("arguments.json");
        generate_and_save_args(&client, assumevalid_params, &args_file.to_string_lossy()).await?;
        let args_elapsed = args_start_time.elapsed();
        debug!(
            "{} args generated in {:.2}s",
            job_info,
            args_elapsed.as_secs_f64()
        );

        // Prove the batch using the library directly
        let batch_result = run_and_prove_with_library(
            &params.executable,
            &args_file,
            &batch_dir,
            params.prover_params_file.as_deref(),
            true, // verify
        );

        match batch_result {
            Ok(proof_path) => {
                info!("{} done", job_info);

                // Store the last proof path and height for later upload
                last_proof_path = Some(proof_path);
                last_height = current_height + current_step;
                current_height += current_step;

                // Clean up temporary files if requested
                if !params.keep_temp_files {
                    if let Err(e) = std::fs::remove_file(&args_file) {
                        warn!("Failed to remove args file: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Batch at height {} failed: {}", current_height, e);
                info!("Stopping further processing due to batch failure");
                return Err(e);
            }
        }
    }

    // Upload only the last proof to GCS
    if params.save_to_gcs {
        if let Some(proof_path) = last_proof_path {
            info!("Uploading final proof to GCS for height {}", last_height);
            let client = ProveClient::new(ProveConfig {
                bridge_node_url: params.bridge_url.clone(),
            });

            let timestamp = format!("{}", chrono::Utc::now());
            let chainstate = client.get_chain_state(last_height).await?;
            let proof = deserialize_proof_from_file(&proof_path, ProofFormat::CairoSerde)?;

            let recent_proof = RecentProof {
                timestamp,
                chainstate,
                proof,
            };
            upload_recent_proof(&recent_proof, &params.gcs_bucket).await?;
            info!("Successfully uploaded final proof to GCS");
        }
    }

    Ok(())
}
