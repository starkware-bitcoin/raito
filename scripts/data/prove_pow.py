#!/usr/bin/env python3

import json
import re
import os
import argparse
import subprocess
import logging
from pathlib import Path
from generate_data import generate_data
from format_args import format_args
from format_assumevalid_args import generate_assumevalid_args
from logging.handlers import TimedRotatingFileHandler
import traceback
import colorlog
from dataclasses import dataclass
from typing import Optional
import datetime

logger = logging.getLogger(__name__)

TMP_DIR = Path(".tmp")
PROOF_DIR = Path(".proofs")


@dataclass
class StepInfo:
    step: str
    stdout: str
    stderr: str
    returncode: int
    elapsed: float
    max_memory: Optional[int]


def setup_logging(verbose=False, log_filename="proving.log"):
    """
    Set up logging configuration with both file and console handlers.

    Args:
        verbose (bool): If True, set DEBUG level; otherwise INFO level
        log_filename (str): Name of the log file
    """
    # File handler setup
    file_handler = TimedRotatingFileHandler(
        filename=log_filename,
        when="midnight",
        interval=1,
        backupCount=14,
        encoding="utf8",
    )
    file_handler.setLevel(logging.INFO)
    file_handler.setFormatter(
        logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
    )

    # Console handler with colors
    console_handler = colorlog.StreamHandler()
    console_handler.setLevel(logging.DEBUG)
    console_handler.setFormatter(
        colorlog.ColoredFormatter(
            "%(asctime)s - %(log_color)s%(levelname)s%(reset)s - %(message)s",
            log_colors={
                "DEBUG": "cyan",
                "INFO": "green",
                "WARNING": "yellow",
                "ERROR": "red",
                "CRITICAL": "red,bg_white",
            },
        )
    )

    # Root logger setup
    root_logger = logging.getLogger()
    root_logger.addHandler(console_handler)
    root_logger.addHandler(file_handler)

    # Set log level based on verbose flag
    if verbose:
        root_logger.setLevel(logging.DEBUG)
    else:
        root_logger.setLevel(logging.INFO)

    # Set specific log levels for external modules
    logging.getLogger("urllib3").setLevel(logging.WARNING)
    logging.getLogger("generate_data").setLevel(logging.WARNING)


def run(cmd, timeout=None):
    """Run a subprocess and measure execution time and memory usage (Linux only, using /usr/bin/time -v)"""
    import time
    import platform
    import re

    if platform.system() != "Linux":
        raise RuntimeError(
            "This script only supports Linux for timing and memory measurement."
        )
    # Prepend /usr/bin/time -v to the command
    time_cmd = ["/usr/bin/time", "-v"] + cmd
    start_time = time.time()
    try:
        result = subprocess.run(
            time_cmd, capture_output=True, text=True, check=False, timeout=timeout
        )
        elapsed = time.time() - start_time
        # /usr/bin/time -v outputs memory usage to stderr
        max_mem_match = re.search(
            r"Maximum resident set size \(kbytes\): (\d+)", result.stderr
        )
        max_memory = int(max_mem_match.group(1)) if max_mem_match else None
        # Remove the /
        # Split stderr into time output and actual stderr
        time_lines = []
        actual_stderr = []
        for line in result.stderr.splitlines():
            if (
                line.startswith("\t")
                or "Maximum resident set size" in line
                or "Command being timed" in line
                or "User time" in line
                or "System time" in line
                or "Percent of CPU" in line
                or "Elapsed (wall clock) time" in line
                or "Average" in line
                or "Exit status" in line
            ):
                time_lines.append(line)
            else:
                actual_stderr.append(line)
        cleaned_stderr = "\n".join(actual_stderr)
        return result.stdout, cleaned_stderr, result.returncode, elapsed, max_memory
    except subprocess.TimeoutExpired as e:
        elapsed = time.time() - start_time
        return "", f"Process timed out after {timeout} seconds", -1, elapsed, None


def save_prover_log(
    batch_dir, step_name, stdout, stderr, returncode, elapsed, max_memory
):
    log_file = batch_dir / f"{step_name.lower()}.log"

    with open(log_file, "w", encoding="utf-8") as f:
        f.write(f"=== {step_name} STEP LOG ===\n")
        f.write(f"Timestamp: {datetime.datetime.now().isoformat()}\n")
        f.write(f"Return Code: {returncode}\n")
        f.write(f"Execution Time: {elapsed:.2f} seconds\n")
        if max_memory is not None:
            f.write(f"Max Memory Usage: {max_memory/1024:.1f} MB\n")
        f.write("\n")

        if stdout:
            f.write("=== STDOUT ===\n")
            f.write(stdout)
            f.write("\n")

        if stderr:
            f.write("=== STDERR ===\n")
            f.write(stderr)
            f.write("\n")


def run_prover(job_info, executable, proof, arguments):
    """
    Run the prover pipeline:
    1. Generate a pie using cairo-execute
    2. Bootload using stwo-bootloader
    3. Prove using adapted_stwo
    Returns a tuple: (steps_info, total_elapsed, max_mem)
    steps_info is a list of dicts with keys: step, stdout, stderr, returncode, elapsed, max_memory
    """
    # Get the batch directory from the proof file path
    batch_dir = Path(proof).parent

    # Prepare intermediate file paths within the batch directory
    pie_file = batch_dir / "pie.cairo_pie.zip"
    priv_json = batch_dir / "priv.json"
    pub_json = batch_dir / "pub.json"

    total_elapsed = 0.0
    max_mem = 0
    steps_info = []

    # 1. Generate pie
    pie_cmd = [
        "cairo-execute",
        "--layout",
        "all_cairo_stwo",
        "--args-file",
        arguments,
        "--prebuilt",
        "--output-path",
        str(pie_file),
        executable,
    ]
    logger.debug(f"{job_info} [PIE] command:\n{' '.join(map(str, pie_cmd))}")
    stdout, stderr, returncode, elapsed, max_memory = run(pie_cmd)
    steps_info.append(
        StepInfo(
            step="PIE",
            stdout=stdout,
            stderr=stderr,
            returncode=returncode,
            elapsed=elapsed,
            max_memory=max_memory,
        )
    )
    # Save PIE step log
    save_prover_log(batch_dir, "PIE", stdout, stderr, returncode, elapsed, max_memory)
    if returncode != 0:
        return steps_info

    # 2. Bootload
    bootload_cmd = [
        "stwo-bootloader",
        "--pie",
        str(pie_file),
        "--output-path",
        str(batch_dir),
    ]
    logger.debug(f"{job_info} [BOOTLOAD] command:\n{' '.join(map(str, bootload_cmd))}")
    stdout, stderr, returncode, elapsed, max_memory = run(bootload_cmd)
    steps_info.append(
        StepInfo(
            step="BOOTLOAD",
            stdout=stdout,
            stderr=stderr,
            returncode=returncode,
            elapsed=elapsed,
            max_memory=max_memory,
        )
    )
    # Save BOOTLOAD step log
    save_prover_log(
        batch_dir, "BOOTLOAD", stdout, stderr, returncode, elapsed, max_memory
    )
    if returncode != 0:
        logger.error(f"{job_info} [BOOTLOAD] error: {stdout or stderr}")
        return steps_info

    # 3. Prove
    prove_cmd = [
        "adapted_stwo",
        "--priv_json",
        str(priv_json),
        "--pub_json",
        str(pub_json),
        "--params_json",
        "../../packages/assumevalid/prover_params.json",
        "--proof_path",
        str(proof),
        "--proof-format",
        "cairo-serde",
        "--verify",
    ]
    logger.debug(f"{job_info} [PROVE] command:\n{' '.join(map(str, prove_cmd))}")
    stdout, stderr, returncode, elapsed, max_memory = run(prove_cmd)

    steps_info.append(
        StepInfo(
            step="PROVE",
            stdout=stdout,
            stderr=stderr,
            returncode=returncode,
            elapsed=elapsed,
            max_memory=max_memory,
        )
    )
    # Save PROVE step log (stwo prover output)
    save_prover_log(batch_dir, "PROVE", stdout, stderr, returncode, elapsed, max_memory)

    if returncode == 0:
        temp_files = [pie_file, pub_json]

        # Parse priv.json to get trace and memory file paths
        if priv_json.exists():
            try:
                with open(priv_json, "r") as f:
                    priv_data = json.load(f)
                    if "trace_path" in priv_data:
                        temp_files.append(Path(priv_data["trace_path"]))
                    if "memory_path" in priv_data:
                        temp_files.append(Path(priv_data["memory_path"]))
                temp_files.append(
                    priv_json
                )  # Add priv.json itself after extracting paths
            except Exception as e:
                logger.warning(f"Failed to parse {priv_json} for cleanup: {e}")
                temp_files.append(priv_json)  # Still try to clean up priv.json

        for temp_file in temp_files:
            try:
                if temp_file.exists():
                    temp_file.unlink()
                    logger.debug(f"Cleaned up temporary file: {temp_file}")
            except Exception as e:
                logger.warning(f"Failed to clean up {temp_file}: {e}")

    return steps_info


def prove_batch(height, step, fast_data_generation=True):
    mode = "light"
    job_info = f"Job(height='{height}', blocks={step})"

    logger.debug(f"{job_info} proving...")

    try:
        # Create dedicated directory for this proof batch
        batch_name = f"{mode}_{height}_to_{height + step}"
        batch_dir = PROOF_DIR / batch_name
        batch_dir.mkdir(exist_ok=True)

        # Previous Proof - look for it in the previous batch directory
        previous_proof_file = None
        if height > 0:
            # Find the previous proof by looking for the directory that ends at current height
            for proof_dir in PROOF_DIR.glob(f"{mode}_*_to_{height}"):
                previous_proof_file = proof_dir / "proof.json"
                if previous_proof_file.exists():
                    break

        logger.debug(f"{job_info} generating data...")

        # Batch data - store in the batch directory
        batch_file = batch_dir / "batch.json"
        batch_data = generate_data(
            mode=mode, initial_height=height, num_blocks=step, fast=fast_data_generation
        )
        batch_args = {
            "chain_state": batch_data["chain_state"],
            "blocks": batch_data["blocks"],
        }
        batch_file.write_text(json.dumps(batch_args, indent=2))

        logger.debug(f"{job_info} generating args...")

        # Arguments file - store in the batch directory
        arguments_file = batch_dir / "arguments.json"
        args = generate_assumevalid_args(batch_file, previous_proof_file)
        arguments_file.write_text(json.dumps(args))

        # Final proof file - store in the batch directory
        proof_file = batch_dir / "proof.json"

        # run prover
        steps_info = run_prover(
            job_info,
            "../../target/proving/assumevalid.executable.json",
            str(proof_file),
            str(arguments_file),
        )

        total_elapsed = sum(step.elapsed for step in steps_info)

        max_memory_candidates = [
            step.max_memory for step in steps_info if step.max_memory is not None
        ]
        max_memory = max(max_memory_candidates) if max_memory_candidates else None

        last_step = steps_info[-1]
        final_return_code = last_step.returncode
        if final_return_code != 0:
            error = last_step.stderr or last_step.stdout
            logger.error(f"{job_info} error:\n{error}")
            return False
        else:
            for info in steps_info:
                mem_usage = (
                    f"{info.max_memory/1024:.1f} MB"
                    if info.max_memory is not None
                    else "N/A"
                )
                logger.debug(
                    f"{job_info}, [{info.step}] time: {info.elapsed:.2f} s max memory: {mem_usage}"
                )
            logger.info(
                f"{job_info} done, total execution time: {total_elapsed:.2f} seconds"
                + (
                    f", max memory: {max_memory/1024:.1f} MB"
                    if max_memory is not None
                    else ""
                )
            )

            return True

    except Exception as e:
        logger.error(
            f"{job_info} error while processing {job_info}:\n{e}\nstacktrace:\n{traceback.format_exc()}"
        )
        return False


def main(start, blocks, step, fast_data_generation=True):
    logger.info(
        "Initial height: %d, blocks: %d, step: %d, fast_data_generation: %s",
        start,
        blocks,
        step,
        fast_data_generation,
    )

    PROOF_DIR.mkdir(exist_ok=True)

    end = start + blocks

    # Generate height range
    height_range = range(start, end, step)
    processing_step = step

    processed_count = 0
    total_jobs = len(list(height_range))

    # Process jobs sequentially
    for height in height_range:
        success = prove_batch(height, processing_step, fast_data_generation)
        if success:
            processed_count += 1
        else:
            logger.info(f"Job at height: {height} failed, stopping further processing")
            return

    logger.info(f"All {processed_count} jobs have been processed successfully")


def auto_detect_start():
    """Auto-detect the starting height by finding the highest ending height from existing proof directories."""
    max_height = 0
    pattern = re.compile(r"light_\d+_to_(\d+)")

    if not PROOF_DIR.exists():
        return max_height

    for proof_dir in PROOF_DIR.iterdir():
        if proof_dir.is_dir():
            m = pattern.match(proof_dir.name)
            if m:
                # Check if the proof file actually exists
                proof_file = proof_dir / "proof.json"
                if proof_file.exists():
                    end_height = int(m.group(1))
                    if end_height > max_height:
                        max_height = end_height
    return max_height


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run single-threaded client script")
    parser.add_argument(
        "--start",
        type=int,
        required=False,
        help="Start block height (if not set, will auto-detect from last proof)",
    )
    parser.add_argument(
        "--blocks", type=int, default=1, help="Number of blocks to process"
    )
    parser.add_argument(
        "--step", type=int, default=10, help="Step size for block processing"
    )
    parser.add_argument("--verbose", action="store_true", help="Verbose logging")
    parser.add_argument(
        "--slow-data-generation",
        action="store_true",
        help="Use slow data generation mode (default is fast mode)",
    )

    args = parser.parse_args()

    # Setup logging using the extracted function
    setup_logging(verbose=args.verbose)

    start = args.start
    if start is None:
        start = auto_detect_start()
        logger.info(f"Auto-detected start: {start}")

    # Convert slow_data_generation flag to fast_data_generation parameter
    fast_data_generation = not args.slow_data_generation

    main(start, args.blocks, args.step, fast_data_generation)
