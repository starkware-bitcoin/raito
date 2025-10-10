use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::{ProveClient, ProveConfig, AssumeValidParams, generate_and_save_args};

/// Parameters for proving batch
#[derive(Debug, Clone)]
pub struct ProveBatchParams {
    /// Path to the assumevalid arguments JSON file
    pub arguments_file: PathBuf,
    /// Output directory for proof and temporary files
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

/// Result of the proving batch process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveBatchResult {
    /// Path to the generated proof file
    pub proof_path: PathBuf,
    /// Total execution time
    pub total_elapsed: Duration,
    /// Maximum memory usage in KB (if available)
    pub max_memory: Option<u64>,
    /// Execution metrics for each step
    pub step_metrics: Vec<StepMetrics>,
}

/// Metrics for a single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepMetrics {
    /// Name of the step
    pub step_name: String,
    /// Execution time for this step
    pub elapsed: Duration,
    /// Maximum memory usage in KB (if available)
    pub max_memory: Option<u64>,
    /// Return code
    pub return_code: i32,
}

/// Generate program-input.json for bootloader execution
pub async fn generate_program_input(
    executable_path: &Path,
    args_file: &Path,
    output_file: &Path,
) -> Result<()> {
    // Convert to absolute paths
    let executable_path = executable_path.canonicalize()?;
    let args_file = args_file.canonicalize()?;

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
    tokio::fs::write(output_file, json).await?;

    info!("Generated program-input.json at {}", output_file.display());
    Ok(())
}

/// Run cairo_program_runner with bootloader
pub async fn run_cairo_runner(
    bootloader_path: &Path,
    program_input_path: &Path,
    output_dir: &Path,
) -> Result<StepMetrics> {
    let start_time = Instant::now();

    // Set up output file paths
    let priv_json = output_dir.join("priv.json");
    let pub_json = output_dir.join("pub.json");
    let trace_file = output_dir.join("trace.json");
    let memory_file = output_dir.join("memory.json");
    let resources_file = output_dir.join("resources.json");

    // Build the command
    let mut cmd = Command::new("cairo_program_runner");
    cmd.args([
        "--program",
        bootloader_path.to_str().unwrap(),
        "--program_input",
        program_input_path.to_str().unwrap(),
        "--air_public_input",
        pub_json.to_str().unwrap(),
        "--air_private_input",
        priv_json.to_str().unwrap(),
        "--trace_file",
        trace_file.to_str().unwrap(),
        "--memory_file",
        memory_file.to_str().unwrap(),
        "--layout",
        "all_cairo_stwo",
        "--proof_mode",
        "--execution_resources_file",
        resources_file.to_str().unwrap(),
        "--disable_trace_padding",
        "--merge_extra_segments",
    ]);

    debug!("Running cairo_program_runner: {:?}", cmd);

    // Execute the command
    let output = cmd.output()?;
    let elapsed = start_time.elapsed();

    // Parse memory usage from stderr (Linux /usr/bin/time format)
    let max_memory = parse_memory_usage(&String::from_utf8_lossy(&output.stderr));

    let metrics = StepMetrics {
        step_name: "CAIRO_RUNNER".to_string(),
        elapsed,
        max_memory,
        return_code: output.status.code().unwrap_or(-1),
    };

    if output.status.success() {
        info!(
            "Cairo runner completed successfully in {:.2}s",
            elapsed.as_secs_f64()
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(
            "Cairo runner failed with return code {}",
            metrics.return_code
        );
        error!("STDOUT: {}", stdout);
        error!("STDERR: {}", stderr);
        return Err(anyhow!("Cairo runner failed: {}", stderr));
    }

    Ok(metrics)
}

/// Run adapted_stwo prover to generate proof
pub async fn run_prover(
    priv_json_path: &Path,
    pub_json_path: &Path,
    prover_params_path: &Path,
    proof_output_path: &Path,
) -> Result<StepMetrics> {
    let start_time = Instant::now();

    // Build the command
    let mut cmd = Command::new("adapted_stwo");
    cmd.args([
        "--priv_json",
        priv_json_path.to_str().unwrap(),
        "--pub_json",
        pub_json_path.to_str().unwrap(),
        "--params_json",
        prover_params_path.to_str().unwrap(),
        "--proof_path",
        proof_output_path.to_str().unwrap(),
        "--proof-format",
        "cairo-serde",
        "--verify",
    ]);

    debug!("Running adapted_stwo: {:?}", cmd);

    // Execute the command
    let output = cmd.output()?;
    let elapsed = start_time.elapsed();

    // Parse memory usage from stderr (Linux /usr/bin/time format)
    let max_memory = parse_memory_usage(&String::from_utf8_lossy(&output.stderr));

    let metrics = StepMetrics {
        step_name: "PROVE".to_string(),
        elapsed,
        max_memory,
        return_code: output.status.code().unwrap_or(-1),
    };

    if output.status.success() {
        info!(
            "Prover completed successfully in {:.2}s",
            elapsed.as_secs_f64()
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Prover failed with return code {}", metrics.return_code);
        error!("STDOUT: {}", stdout);
        error!("STDERR: {}", stderr);
        return Err(anyhow!("Prover failed: {}", stderr));
    }

    Ok(metrics)
}

/// Parse memory usage from /usr/bin/time output
fn parse_memory_usage(stderr: &str) -> Option<u64> {
    for line in stderr.lines() {
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
pub async fn prove_batch(params: ProveBatchParams) -> Result<ProveBatchResult> {
    let start_time = Instant::now();
    let mut step_metrics = Vec::new();

    info!("Starting assumevalid proving process");
    info!("Arguments file: {}", params.arguments_file.display());
    info!("Output directory: {}", params.output_dir.display());

    // Create output directory
    tokio::fs::create_dir_all(&params.output_dir).await?;

    // Set up file paths
    let program_input_file = params.output_dir.join("program-input.json");
    let priv_json = params.output_dir.join("priv.json");
    let pub_json = params.output_dir.join("pub.json");
    let trace_file = params.output_dir.join("trace.json");
    let memory_file = params.output_dir.join("memory.json");
    let resources_file = params.output_dir.join("resources.json");
    let proof_file = params.output_dir.join("proof.json");

    // Step 1: Generate program input
    info!("Step 1: Generating program input");
    generate_program_input(
        &params.executable,
        &params.arguments_file,
        &program_input_file,
    )
    .await?;

    // Step 2: Run cairo_program_runner
    info!("Step 2: Running cairo_program_runner");
    let cairo_metrics =
        run_cairo_runner(&params.bootloader, &program_input_file, &params.output_dir).await?;
    step_metrics.push(cairo_metrics);

    // Step 3: Run prover
    info!("Step 3: Running adapted_stwo prover");
    let prover_metrics =
        run_prover(&priv_json, &pub_json, &params.prover_params, &proof_file).await?;
    step_metrics.push(prover_metrics);

    // Clean up temporary files if requested
    if !params.keep_temp_files {
        info!("Cleaning up temporary files");
        let temp_files = vec![program_input_file, trace_file, memory_file, resources_file];

        for temp_file in temp_files {
            if temp_file.exists() {
                if let Err(e) = tokio::fs::remove_file(&temp_file).await {
                    warn!(
                        "Failed to remove temporary file {}: {}",
                        temp_file.display(),
                        e
                    );
                }
            }
        }

        // Also clean up priv.json and pub.json
        for temp_file in [&priv_json, &pub_json] {
            if temp_file.exists() {
                if let Err(e) = tokio::fs::remove_file(temp_file).await {
                    warn!(
                        "Failed to remove temporary file {}: {}",
                        temp_file.display(),
                        e
                    );
                }
            }
        }
    }

    let total_elapsed = start_time.elapsed();
    let max_memory = step_metrics.iter().filter_map(|m| m.max_memory).max();

    let result = ProveBatchResult {
        proof_path: proof_file,
        total_elapsed,
        max_memory,
        step_metrics,
    };

    info!(
        "Proving completed successfully in {:.2}s",
        total_elapsed.as_secs_f64()
    );
    if let Some(mem) = max_memory {
        info!("Maximum memory usage: {:.1} MB", mem as f64 / 1024.0);
    }
    info!("Proof saved to: {}", result.proof_path.display());

    Ok(result)
}

/// Parameters for proving multiple batches iteratively
#[derive(Debug, Clone)]
pub struct ProveParams {
    /// Starting block height
    pub start_height: u32,
    /// Total number of blocks to process
    pub total_blocks: u32,
    /// Step size for each batch
    pub step_size: u32,
    /// Bridge node RPC URL
    pub bridge_url: String,
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

/// Main function to prove multiple batches iteratively
pub async fn prove(params: ProveParams) -> Result<()> {
    info!(
        "Starting iterative proving process: start_height={}, total_blocks={}, step_size={}",
        params.start_height,
        params.total_blocks,
        params.step_size
    );

    // Create output directory
    tokio::fs::create_dir_all(&params.output_dir).await?;

    let end_height = params.start_height + params.total_blocks;
    let mut current_height = params.start_height;

    // Process batches sequentially
    while current_height < end_height {
        let current_step = std::cmp::min(params.step_size, end_height - current_height);
        if current_step <= 0 {
            break;
        }

        // Process a single batch
        let job_info = format!("Job(height='{}', blocks={})", current_height, current_step);
        info!("{} proving...", job_info);

        // Create dedicated directory for this proof batch
        let batch_name = format!(
            "batch_{}_to_{}",
            current_height,
            current_height + current_step
        );
        let batch_dir = params.output_dir.join(&batch_name);
        tokio::fs::create_dir_all(&batch_dir).await?;

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
            "{} [GENERATE_ARGS] time: {:.2} s",
            job_info,
            args_elapsed.as_secs_f64()
        );

        // Prove the batch
        let prove_params = ProveBatchParams {
            arguments_file: args_file,
            output_dir: batch_dir.clone(),
            executable: params.executable.clone(),
            bootloader: params.bootloader.clone(),
            prover_params: params.prover_params.clone(),
            keep_temp_files: params.keep_temp_files,
        };

        let batch_result = prove_batch(prove_params).await;

        match batch_result {
            Ok(mut result) => {
                // Add the args generation time to the total elapsed
                result.total_elapsed += args_elapsed;

                info!(
                    "{} done, total execution time: {:.2} seconds",
                    job_info,
                    result.total_elapsed.as_secs_f64()
                );
                if let Some(mem) = result.max_memory {
                    info!("{} max memory: {:.1} MB", job_info, mem as f64 / 1024.0);
                }

                info!("Batch at height {} completed successfully", current_height);
                current_height += current_step;
            }
            Err(e) => {
                error!("Batch at height {} failed: {}", current_height, e);
                info!("Stopping further processing due to batch failure");
                return Err(e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_batch_params() {
        let params = ProveBatchParams {
            arguments_file: PathBuf::from("args.json"),
            output_dir: PathBuf::from("./proofs"),
            executable: PathBuf::from("target/proving/assumevalid.executable.json"),
            bootloader: PathBuf::from("bootloaders/simple_bootloader_compiled.json"),
            prover_params: PathBuf::from("packages/assumevalid/prover_params.json"),
            keep_temp_files: false,
        };
        assert_eq!(params.arguments_file, PathBuf::from("args.json"));
        assert_eq!(params.keep_temp_files, false);
    }

    #[test]
    fn test_parse_memory_usage() {
        let stderr = "Maximum resident set size (kbytes): 123456";
        assert_eq!(parse_memory_usage(stderr), Some(123456));

        let stderr_no_memory = "Some other output";
        assert_eq!(parse_memory_usage(stderr_no_memory), None);
    }
}
