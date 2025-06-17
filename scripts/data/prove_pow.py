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
from logging.handlers import TimedRotatingFileHandler
import traceback
import colorlog

logger = logging.getLogger(__name__)

TMP_DIR = Path(".tmp")
PROOF_DIR = Path(".proofs")


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
    """Run a subprocess and measure execution time and memory usage (Linux only, using resource module)"""
    import time
    import resource
    import platform

    if platform.system() != "Linux":
        raise RuntimeError(
            "This script only supports Linux for timing and memory measurement."
        )
    start_time = time.time()
    usage_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, check=True, timeout=timeout
        )
        elapsed = time.time() - start_time
        usage_after = resource.getrusage(resource.RUSAGE_CHILDREN)
        # ru_maxrss is in kilobytes on Linux
        max_memory = usage_after.ru_maxrss - usage_before.ru_maxrss
        return result.stdout, result.stderr, result.returncode, elapsed, max_memory
    except subprocess.TimeoutExpired as e:
        elapsed = time.time() - start_time
        usage_after = resource.getrusage(resource.RUSAGE_CHILDREN)
        max_memory = usage_after.ru_maxrss - usage_before.ru_maxrss
        return "", f"Process timed out after {timeout} seconds", -1, elapsed, max_memory


def prove_batch(height, step):

    mode = "light"
    job_info = f"Job(height='{height}', step={step})"

    logger.info(f"Proving {job_info}")

    try:
        # Load previous proof
        if height == 0:
            # Option::None
            chain_state_proof = [hex(1)]
        else:
            # load previous proof file
            previous_proof_file = PROOF_DIR / f"{mode}_{height}.proof.json"

            if previous_proof_file.exists():
                chain_state_proof = json.loads(previous_proof_file.read_text())
                # Option::Some(chain_state_proof)
                chain_state_proof = [hex(0)] + chain_state_proof
            else:
                raise Exception(
                    f"Previous proof file {str(previous_proof_file)} does not exist"
                )

        # Load batch data
        batch_file = TMP_DIR / f"{mode}_{height}_{step}.json"

        batch_data = generate_data(
            mode=mode, initial_height=height, num_blocks=step, fast=True
        )

        # prepare args
        batch_args = {
            "chain_state": batch_data["chain_state"],
            "blocks": batch_data["blocks"],
        }

        Path(batch_file).write_text(json.dumps(batch_args, indent=2))
        arguments_file = batch_file.as_posix().replace(".json", "-arguments.json")
        args = format_args(batch_file)

        # add chain state proof to arguments
        args = json.loads(args)
        args = args + chain_state_proof

        with open(arguments_file, "w") as af:
            af.write(json.dumps(args))

        proof_file = PROOF_DIR / f"{mode}_{height + step}.proof.json"

        command = [
            "cairo-prove",
            "prove",
            "../../target/proving/fold.executable.json",
            str(proof_file),
            "--arguments-file",
            str(arguments_file),
            "--proof-format",
            "cairo-serde",
        ]

        # command = [
        #     "scarb", "--profile", "proving", "execute",
        #     "--no-build",
        #     "--package", "assumevalid",
        #     "--executable-name", "fold",
        #     "--arguments-file", str(arguments_file),
        #     "--print-resource-usage"
        # ]

        logger.debug(f"With command:\n{' '.join(command)}")

        stdout, stderr, returncode, elapsed_time, max_memory = run(command)

        if (
            returncode != 0
            or "FAIL" in stdout
            or "error" in stdout
            or "panicked" in stdout
        ):
            error = stdout or stderr
            logger.error(f"{job_info} error: {error}")
            return False
        else:
            logger.info(
                f"{job_info} done, execution time: {elapsed_time:.2f} seconds"
                + (
                    f", max memory: {max_memory/1024:.1f} MB"
                    if max_memory is not None
                    else ""
                )
            )
            return True

    except Exception as e:
        logger.error(f"Error while processing: {job_info}:\n{e}")
        logger.error(f"Stacktrace:\n{traceback.format_exc()}")
        return False


def main(start, blocks, step):
    """Main processing function - single threaded"""
    logger.info(
        "Starting single-threaded client, initial height: %d, blocks: %d, step: %d",
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
        try:
            # Process the height (generate and process in one step)
            success = prove_batch(height, processing_step)
            if success:
                processed_count += 1
        except subprocess.TimeoutExpired:
            logger.warning(f"Timeout while processing height {height}")
            return
        except Exception as e:
            logger.error(f"Error while processing height {height}: {e}")
            return

    logger.info(f"All {processed_count} jobs have been processed successfully.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run single-threaded client script")
    parser.add_argument("--start", type=int, required=True, help="Start block height")
    parser.add_argument(
        "--blocks", type=int, default=1, help="Number of blocks to process"
    )
    parser.add_argument(
        "--step", type=int, default=1, help="Step size for block processing"
    )

    parser.add_argument("--verbose", action="store_true", help="Verbose logging")

    args = parser.parse_args()

    # Setup logging using the extracted function
    setup_logging(verbose=args.verbose)

    main(args.start, args.blocks, args.step)
