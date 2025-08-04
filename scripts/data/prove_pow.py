#!/usr/bin/env python3

import json
import re
import os
import argparse
import subprocess
import logging
import time
from pathlib import Path
from generate_data import generate_data
from format_args import format_args
from format_assumevalid_args import generate_assumevalid_args
from generate_program_input import generate_program_input
import traceback
import colorlog
from dataclasses import dataclass
from typing import Optional
import datetime
import logging_setup

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
    1. Generate program input using generate_program_input.py
    2. Run cairo_program_runner with bootloader
    3. Prove using adapted_stwo
    Returns a tuple: (steps_info, total_elapsed, max_mem)
    steps_info is a list of dicts with keys: step, stdout, stderr, returncode, elapsed, max_memory
    """
    batch_dir = Path(proof).parent

    program_input_file = batch_dir / "program-input.json"
    priv_json = batch_dir / "priv.json"
    pub_json = batch_dir / "pub.json"
    trace_file = batch_dir / "trace.json"
    memory_file = batch_dir / "memory.json"
    resources_file = batch_dir / "resources.json"

    total_elapsed = 0.0
    max_mem = 0
    steps_info = []

    try:
        generate_program_input(
            executable_path=executable,
            args_file=arguments,
            program_hash_function="blake",
            output_file=str(program_input_file),
        )
    except Exception as e:
        logger.error(f"{job_info} failed to generate program input: {e}")
        return steps_info

    cairo_runner_cmd = [
        "cairo_program_runner",
        "--program",
        "../../bootloaders/simple_bootloader_compiled.json",
        "--program_input",
        str(program_input_file),
        "--air_public_input",
        str(pub_json),
        "--air_private_input",
        str(priv_json),
        "--trace_file",
        str(trace_file.resolve()),
        "--memory_file",
        str(memory_file.resolve()),
        "--layout",
        "all_cairo_stwo",
        "--proof_mode",
        "--execution_resources_file",
        str(resources_file),
        "--disable_trace_padding",
        "--merge_extra_segments",
    ]
    logger.debug(
        f"{job_info} [CAIRO_RUNNER] command:\n{' '.join(map(str, cairo_runner_cmd))}"
    )
    stdout, stderr, returncode, elapsed, max_memory = run(cairo_runner_cmd)
    steps_info.append(
        StepInfo(
            step="CAIRO_RUNNER",
            stdout=stdout,
            stderr=stderr,
            returncode=returncode,
            elapsed=elapsed,
            max_memory=max_memory,
        )
    )

    save_prover_log(
        batch_dir, "CAIRO_RUNNER", stdout, stderr, returncode, elapsed, max_memory
    )

    if returncode != 0:
        logger.error(f"{job_info} [CAIRO_RUNNER] error: {stdout or stderr}")

        # Try to get more meaningful error message using scarb execute
        logger.info(
            f"{job_info} [CAIRO_RUNNER] failed, trying scarb execute for better error messages..."
        )
        scarb_cmd = [
            "cairo-execute",
            "--prebuilt",
            "--args-file",
            str(arguments),
            "--output-path",
            str(batch_dir / "output.txt"),
            "--layout",
            "all_cairo_stwo",
            executable,
        ]
        logger.debug(
            f"{job_info} [CAIRO_EXECUTE] command:\n{' '.join(map(str, scarb_cmd))}"
        )
        (
            scarb_stdout,
            scarb_stderr,
            scarb_returncode,
            scarb_elapsed,
            scarb_max_memory,
        ) = run(scarb_cmd)

        steps_info.append(
            StepInfo(
                step="CAIRO_EXECUTE",
                stdout=scarb_stdout,
                stderr=scarb_stderr,
                returncode=scarb_returncode,
                elapsed=scarb_elapsed,
                max_memory=scarb_max_memory,
            )
        )

        save_prover_log(
            batch_dir,
            "CAIRO_EXECUTE",
            scarb_stdout,
            scarb_stderr,
            scarb_returncode,
            scarb_elapsed,
            scarb_max_memory,
        )

        logger.error(
            f"{job_info} [CAIRO_EXECUTE] output:\n{scarb_stdout or scarb_stderr}"
        )
        return steps_info

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

    save_prover_log(batch_dir, "PROVE", stdout, stderr, returncode, elapsed, max_memory)

    if returncode == 0:
        temp_files = [
            program_input_file,
            # pub_json,
            trace_file,
            memory_file,
            # resources_file,
        ]

        if priv_json.exists():
            try:
                with open(priv_json, "r") as f:
                    priv_data = json.load(f)
                    for key in ["trace_path", "memory_path"]:
                        if key in priv_data:
                            temp_files.append(Path(priv_data[key]))
                temp_files.append(priv_json)
            except Exception as e:
                logger.warning(f"Failed to parse {priv_json} for cleanup: {e}")
                temp_files.append(priv_json)

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

        logger.debug(f"{job_info} generating data (fast: {fast_data_generation})...")

        args_start_time = time.time()

        # Batch data - store in the batch directory
        batch_file = batch_dir / "batch.json"
        batch_data = generate_data(
            mode=mode, initial_height=height, num_blocks=step, fast=fast_data_generation
        )
        batch_args = {
            "chain_state": batch_data["chain_state"],
            "blocks": batch_data["blocks"],
            "block_mmr": batch_data["mmr_roots"],
        }
        batch_file.write_text(json.dumps(batch_args, indent=2))

        logger.debug(f"{job_info} generating args...")

        # Arguments file - store in the batch directory
        arguments_file = batch_dir / "arguments.json"
        args = generate_assumevalid_args(batch_file, previous_proof_file)
        arguments_file.write_text(json.dumps(args))

        args_elapsed = time.time() - args_start_time

        # Final proof file - store in the batch directory
        proof_file = batch_dir / "proof.json"

        # run prover
        steps_info = run_prover(
            job_info,
            "../../target/proving/assumevalid.executable.json",
            str(proof_file),
            str(arguments_file),
        )

        total_elapsed = sum(step.elapsed for step in steps_info) + args_elapsed

        max_memory_candidates = [
            step.max_memory for step in steps_info if step.max_memory is not None
        ]
        max_memory = max(max_memory_candidates) if max_memory_candidates else None

        last_step = steps_info[-1]
        final_return_code = last_step.returncode
        if final_return_code != 0:
            error = last_step.stderr or last_step.stdout
            logger.error(f"{job_info} error:\n{error}")
            return None
        else:
            logger.debug(f"{job_info} [GENERATE_ARGS] time: {args_elapsed:.2f} s")
            for info in steps_info:
                mem_usage = (
                    f"{info.max_memory/1024:.1f} MB"
                    if info.max_memory is not None
                    else "N/A"
                )
                logger.debug(
                    f"{job_info} [{info.step}] time: {info.elapsed:.2f} s max memory: {mem_usage}"
                )
            logger.info(
                f"{job_info} done, total execution time: {total_elapsed:.2f} seconds"
                + (
                    f", max memory: {max_memory/1024:.1f} MB"
                    if max_memory is not None
                    else ""
                )
            )
            logger.debug(
                f"{job_info} expected time to complete proving the whole chain: {900000 / step * total_elapsed / 3600:.2f} hours"
            )

            return proof_file

    except Exception as e:
        logger.error(
            f"{job_info} error while processing {job_info}:\n{e}\nstacktrace:\n{traceback.format_exc()}"
        )
        return None


def prove_pow(start, blocks, step, fast_data_generation=True):
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
    latest_proof_file = None

    # Process jobs sequentially
    for height in height_range:
        proof_file = prove_batch(height, processing_step, fast_data_generation)
        if proof_file is not None:
            processed_count += 1
            latest_proof_file = proof_file
        else:
            logger.info(f"Job at height: {height} failed, stopping further processing")
            return None

    logger.info(f"All {processed_count} jobs have been processed successfully")
    return latest_proof_file


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
        "--slow",
        action="store_true",
        help="Use slow data generation mode (default is fast mode)",
    )

    args = parser.parse_args()

    # Setup logging using the extracted function
    logging_setup.setup(verbose=args.verbose)

    start = args.start
    if start is None:
        start = auto_detect_start()
        logger.info(f"Auto-detected start: {start}")

    # Convert slow_data_generation flag to fast_data_generation parameter
    fast_data_generation = not args.slow

    result = prove_pow(start, args.blocks, args.step, fast_data_generation)
    if result is None:
        exit(1)
    else:
        print(f"Proof file generated: {result}")
