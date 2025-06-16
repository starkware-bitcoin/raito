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

    # Console handler setup
    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.DEBUG)
    console_handler.setFormatter(
        logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
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
    """Run a subprocess"""
    try:
        result = subprocess.run(
            cmd, 
            capture_output=True,
            text=True,
            timeout=timeout
        )
        return result.stdout, result.stderr, result.returncode
    
    except subprocess.TimeoutExpired as e:
        # Process was killed due to timeout
        return "", f"Process timed out after {timeout} seconds", -1

def prove_batch(height, step):

    mode = "light"
    job_info = f"Job(height='{height}', step={step}, up_to_height={height + step})"    

    try:
        # Load previous proof
        if height == 0:
            chain_state_proof = [0]
        else:
            # load previous proof file
            previous_proof_file = PROOF_DIR / f"{mode}_{height}.proof.json"        
            
            if previous_proof_file.exists():
                chain_state_proof = json.load(previous_proof_file)
                chain_state_proof = [1] + chain_state_proof
            else:
                raise Exception(f"Previous proof file {previous_proof_file} does not exist")

        # Load batch data
        batch_file = TMP_DIR / f"{mode}_{height}_{step}.json"
        
        batch_data = generate_data(
            mode=mode, initial_height=height, num_blocks=step, fast=True
        )
        
        # store expected in a var and remove from batch_data
        # expected_chain_state = batch_data.pop("expected")

        batch_args = {
            "chain_state": batch_data["chain_state"],
            "blocks": batch_data["blocks"],
            "chain_state_proof": chain_state_proof,
        }

        # serialize 
        Path(batch_file).write_text(json.dumps(batch_args, indent=2))
        
        # Process the batch
        arguments_file = batch_file.as_posix().replace(".json", "-arguments.json")
        
        with open(arguments_file, "w") as af:
            af.write(str(format_args(batch_file, False)))
        
        proof_file = PROOF_DIR / f"{mode}_{height + step}.proof.json"

        # Use cairo-prove to generate proof for the fold function
        stdout, stderr, returncode = run(
            [
                "cairo-prove",
                "prove",
                "target/proving/fold.executable.json",  # Assuming fold executable exists
                proof_file,
                "--arguments-file",
                str(arguments_file),
                "--proof-format",
                "cairo-serde",
            ]
        )
        
        if returncode != 0 or "FAIL" in stdout or "error" in stdout or "panicked" in stdout:
            error = stdout or stderr
            # if returncode == -9:
            #     match = re.search(r"gas_spent=(\d+)", stdout)
            #     gas_info = (
            #         f", gas spent: {int(match.group(1))}"
            #         if match
            #         else ", no gas info found"
            #     )
            #     error = f"Return code -9, killed by OOM?{gas_info}"
            #     message = error
            # else:
            #     error_match = re.search(r"error='([^']*)'", error)
            #     if error_match:
            #         message = error_match.group(1)
            #     else:
            #         error_match = re.search(r"error: (.*)", error, re.DOTALL)
            #         if error_match:
            #             message = error_match.group(1)
            #         else:
            #             message = error
            
            # message = re.sub(r"\s+", " ", message)

            # TODO: handle errors
            logger.error(f"{job_info} error: {error}")
            # logger.debug(f"Full error while processing: {job_info}:\n{error}")
            return False
        else:
            logger.info(f"{job_info} done")

            # match = re.search(r"gas_spent=(\d+)", stdout)
            # gas_info = f"gas spent: {int(match.group(1))}" if match else "no gas info found"
            # logger.info(f"{job_info} done, {gas_info}")
            # if not match:
            #     logger.warning(f"{job_info}: no gas info found")
            return True
            
    except Exception as e:
        logger.error(f"Error while processing: {job_info}:\n{e}")
        return False


def main(start, blocks, step):
    """Main processing function - single threaded"""
    logger.info(
        "Starting single-threaded client, initial height: %d, blocks: %d, step: %d",
        start, blocks, step,
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
        logger.debug(f"Proving job {processed_count + 1}/{total_jobs}: height={height}, steps={processing_step}")

        try:
            # Process the height (generate and process in one step)
            success = prove_batch(height, processing_step)
            if success:
                processed_count += 1
        except subprocess.TimeoutExpired:
            logger.warning(f"Timeout while processing height {height}")
        except Exception as e:
            logger.error(f"Error while processing height {height}: {e}")

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