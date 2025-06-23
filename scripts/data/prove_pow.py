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


def setup_logging(verbose=False, log_filename="client.log"):
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
        logging.Formatter("%(asctime)s - %(name)-10.10s - %(levelname)s - %(message)s")
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


def run_prover(job_info, executable, proof, arguments):
    """
    Run the prover pipeline:
    1. Generate a pie using cairo-execute
    2. Bootload using stwo-bootloader
    3. Prove using adapted_stwo
    Returns a tuple: (steps_info, total_elapsed, max_mem)
    steps_info is a list of dicts with keys: step, stdout, stderr, returncode, elapsed, max_memory
    """
    # Prepare intermediate file paths
    pie_file = Path(proof).with_suffix(".cairo_pie.zip")
    bootloader_dir = Path(proof).parent / (Path(proof).stem + "_bootload")
    bootloader_dir.mkdir(exist_ok=True)
    priv_json = bootloader_dir / "priv.json"
    pub_json = bootloader_dir / "pub.json"

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
    if returncode != 0:
        return steps_info

    # 2. Bootload
    bootload_cmd = [
        "stwo-bootloader",
        "--pie",
        str(pie_file),
        "--output-path",
        str(bootloader_dir),
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
    return steps_info


def prove_batch(height, step):

    mode = "light"
    job_info = f"Job(height='{height}', blocks={step})"

    logger.info(f"{job_info} proving...")

    try:
        # Previous Proof
        previous_proof_file = (
            PROOF_DIR / f"{mode}_{height}.proof.json" if height > 0 else None
        )

        # Batch data
        batch_file = TMP_DIR / f"{mode}_{height}_{step}.json"
        batch_data = generate_data(
            mode=mode, initial_height=height, num_blocks=step, fast=True
        )
        batch_args = {
            "chain_state": batch_data["chain_state"],
            "blocks": batch_data["blocks"],
        }
        Path(batch_file).write_text(json.dumps(batch_args, indent=2))

        # Arguments file
        args = generate_assumevalid_args(batch_file, previous_proof_file)
        arguments_file = batch_file.as_posix().replace(".json", "-arguments.json")
        with open(arguments_file, "w") as af:
            af.write(json.dumps(args))

        proof_file = PROOF_DIR / f"{mode}_{height + step}.proof.json"

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


def main(start, blocks, step):

    logger.info(
        "Initial height: %d, blocks: %d, step: %d",
        start,
        blocks,
        step,
    )

    TMP_DIR.mkdir(exist_ok=True)
    PROOF_DIR.mkdir(exist_ok=True)

    end = start + blocks

    # Generate height range
    height_range = range(start, end, step)
    processing_step = step

    processed_count = 0
    total_jobs = len(list(height_range))

    # Process jobs sequentially
    for height in height_range:
        success = prove_batch(height, processing_step)
        if success:
            processed_count += 1
        else:
            logger.info(f"Job at height: {height} failed, stopping further processing")
            return

    logger.info(f"All {processed_count} jobs have been processed successfully")


def auto_detect_start():
    proof_files = list(PROOF_DIR.glob("light_*.proof.json"))
    max_height = 0
    pattern = re.compile(r"light_(\d+)\.proof\.json")
    for pf in proof_files:
        m = pattern.match(pf.name)
        if m:
            h = int(m.group(1))
            if h > max_height:
                max_height = h
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

    args = parser.parse_args()

    # Setup logging using the extracted function
    setup_logging(verbose=args.verbose)

    start = args.start
    if start is None:
        start = auto_detect_start()
        logger.info(f"Auto-detected start: {start}")

    main(start, args.blocks, args.step)
