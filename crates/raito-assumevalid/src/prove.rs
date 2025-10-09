use anyhow::{anyhow, Result};
use cairo_air::utils::{deserialize_proof_from_file, serialize_proof_to_file, ProofFormat};
use regex::Regex;

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use tracing::{debug, error, info, warn};

use crate::gcs::{download_recent_proof_via_reqwest, upload_recent_proof, RecentProof};
use crate::generate_args::{generate_and_save_args, AssumeValidParams, ProveClient, ProveConfig};

/// Generate program-input.json for bootloader execution
pub async fn generate_program_input(
    executable_path: &Path,
    arguments_file: &Path,
    input_file: &Path,
) -> Result<()> {
    // Convert to absolute paths
    let executable_path = executable_path.canonicalize()?;
    let args_file = arguments_file.canonicalize()?;

    // Create the program input structure
    let program_input = json!({
        "single_page": true,
        "tasks": [
            {
                "type": "Cairo1Executable",
                "path": executable_path.to_string_lossy(),
                "program_hash_function": "blake",
                "user_args_file": args_file.to_string_lossy(),
            }
        ],
    });

    // Write to output file
    let json = serde_json::to_string_pretty(&program_input)?;
    tokio::fs::write(input_file, json).await?;

    debug!("Generated program-input.json at {}", input_file.display());
    Ok(())
}

/// Parse memory usage from /usr/bin/time output
fn parse_memory_usage(stderr: &str) -> Option<u64> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        // macOS format: "  12345  maximum resident set size" (bytes)
        if trimmed.ends_with("maximum resident set size") {
            if let Some(bytes_str) = trimmed.split_whitespace().next() {
                if let Ok(bytes) = bytes_str.parse::<u64>() {
                    // Convert bytes to KB for consistent output
                    return Some(bytes / 1024);
                }
            }
        }
        // Linux format: "Maximum resident set size (kbytes): 12345"
        if line.contains("Maximum resident set size (kbytes):") {
            if let Some(kb_str) = line.split(':').nth(1) {
                if let Ok(kb) = kb_str.trim().parse::<u64>() {
                    return Some(kb);
                }
            }
        }
    }
    None
}

/// Main function to prove batch - orchestrates the full pipeline
pub async fn run_and_prove(
    arguments_file: &Path,
    output_dir: &Path,
    executable: &Path,
    bootloader: &Path,
    prover_params: &Path,
    keep_temp_files: bool,
) -> Result<PathBuf> {
    info!("Starting assumevalid proving process");
    debug!("Arguments file: {}", arguments_file.display());
    debug!("Output directory: {}", output_dir.display());

    // Create output directory
    tokio::fs::create_dir_all(output_dir).await?;

    // Resolve output dir to absolute and set up file paths
    let out_dir = output_dir
        .canonicalize()
        .unwrap_or_else(|_| output_dir.to_path_buf());
    let program_input_file = out_dir.join("program-input.json");
    let proof_file = out_dir.join("proof.json");

    // Prepare program input for the bootloader
    debug!("Generating program-input.json");
    generate_program_input(executable, arguments_file, &program_input_file).await?;

    // Inline stwo_run_and_prove: build and run CLI
    let start_time = Instant::now();
    let program_abs = bootloader.canonicalize()?;
    let input_abs = program_input_file.canonicalize()?;
    let params_abs = prover_params.canonicalize()?;
    let proofs_dir_abs = out_dir.canonicalize()?;

    // Use -l on macOS, -v on Linux for /usr/bin/time
    let time_flag = if cfg!(target_os = "macos") {
        "-l"
    } else {
        "-v"
    };

    let mut cmd = Command::new("/usr/bin/time");
    cmd.args([
        time_flag,
        "stwo_run_and_prove",
        "--program",
        program_abs.to_str().unwrap(),
        "--program_input",
        input_abs.to_str().unwrap(),
        "--prover_params_json",
        params_abs.to_str().unwrap(),
        "--proofs_dir",
        proofs_dir_abs.to_str().unwrap(),
        "--proof-format",
        "cairo-serde",
        "--n_proof_attempts",
        "1",
        "--verify",
    ]);
    debug!("Running command: {:?}", cmd);

    let output = cmd.output()?;
    let elapsed = start_time.elapsed();
    let max_memory = parse_memory_usage(&String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        if let Some(mem) = max_memory {
            info!(
                "stwo_run_and_prove succeeded in {:.2}s, max RSS: {:.1} MB",
                elapsed.as_secs_f64(),
                mem as f64 / 1024.0
            );
        } else {
            info!(
                "stwo_run_and_prove succeeded in {:.2}s",
                elapsed.as_secs_f64()
            );
        }

        // Find and rename the generated proof file to proof.json
        if let Ok(entries) = fs::read_dir(&out_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Look for files that start with "proof_" and end with "_success" or similar patterns
                        if file_name.starts_with("proof_")
                            && (file_name.ends_with("_success") || file_name.contains("_success"))
                        {
                            if let Err(e) = fs::rename(&path, &proof_file) {
                                warn!(
                                    "Failed to rename proof file from {} to {}: {}",
                                    path.display(),
                                    proof_file.display(),
                                    e
                                );
                            } else {
                                debug!(
                                    "Renamed proof file from {} to {}",
                                    file_name,
                                    proof_file.file_name().unwrap().to_string_lossy()
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(
            "stwo_run_and_prove failed with return code {:?}",
            output.status.code()
        );
        error!("STDOUT: {}", stdout);
        error!("STDERR: {}", stderr);
        return Err(anyhow!("stwo_run_and_prove failed: {}", stderr));
    }

    // Clean up temporary files if requested
    if !keep_temp_files {
        debug!("Cleaning up temporary files");
        let temp_files = vec![program_input_file, arguments_file.to_path_buf()];

        for temp_file in temp_files {
            if temp_file.exists() {
                // Use std::fs::remove_file for synchronous context
                if let Err(e) = std::fs::remove_file(&temp_file) {
                    warn!(
                        "Failed to remove temporary file {}: {}",
                        temp_file.display(),
                        e
                    );
                }
            }
        }
    }

    info!("Proof saved to: {}", proof_file.display());

    Ok(proof_file)
}

/// Parameters for proving multiple batches iteratively
#[derive(Debug, Clone)]
pub struct ProveParams {
    pub load_from_gcs: bool,
    pub save_to_gcs: bool,
    /// URL to fetch the latest proof height from (used with --use-gcs)
    pub gcs_bucket: String,
    pub bridge_url: String,
    /// Total number of blocks to process
    pub total_blocks: u32,
    /// Step size for each batch
    pub step_size: u32,
    /// Output directory for all proofs
    pub output_dir: PathBuf,
    /// Path to the Cairo executable JSON file
    pub executable: PathBuf,
    /// Path to the bootloader JSON file
    pub bootloader: PathBuf,
    /// Path to the prover parameters JSON file
    pub prover_params: PathBuf,
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
                            if proof_file.exists() {
                                if end > max_height {
                                    max_height = end;
                                }
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
    // Create dedicated directory for this proof batch
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
        if current_step <= 0 {
            break;
        }

        // Process a single batch
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
        let _args_elapsed = args_start_time.elapsed();

        // Prove the batch using inlined parameters
        let batch_result = run_and_prove(
            &args_file,
            &batch_dir,
            &params.executable,
            &params.bootloader,
            &params.prover_params,
            params.keep_temp_files,
        )
        .await;

        match batch_result {
            Ok(proof_path) => {
                info!("{} done", job_info);

                // Store the last proof path and height for later upload
                last_proof_path = Some(proof_path);
                last_height = current_height + current_step;
                current_height += current_step;
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
