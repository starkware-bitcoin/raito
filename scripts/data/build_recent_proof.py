#!/usr/bin/env python3

import json
import os
import argparse
import logging
import time
import datetime
from pathlib import Path
from typing import Optional, Dict, Any
import traceback

try:
    import colorlog
except ImportError:
    colorlog = None

try:
    from google.cloud import storage
except ImportError:
    storage = None

from generate_data import request_rpc
from prove_pow import auto_detect_start, prove_pow
import logging_setup

logger = logging.getLogger(__name__)

GCS_BUCKET_NAME = os.getenv("GCS_BUCKET_NAME", "raito-proofs")

BITCOIN_RPC = os.getenv("BITCOIN_RPC")
USERPWD = os.getenv("USERPWD")
DEFAULT_URL = "https://bitcoin-mainnet.public.blastapi.io"


def get_latest_block_height() -> int:
    """Get the latest block height from Bitcoin node API."""
    try:
        latest_hash = request_rpc("getbestblockhash", [])

        block_info = request_rpc("getblock", [latest_hash])
        height = block_info["height"]

        logger.info(f"Latest block height: {height}")
        return height
    except Exception as e:
        logger.error(f"Failed to get latest block height: {e}")
        raise


def convert_proof_to_json(proof_file: Path) -> Optional[Path]:
    """Convert proof from Cairo-serde format to JSON format.

    Args:
        proof_file: Path to the original proof file in Cairo-serde format

    Returns:
        Path to the converted JSON proof file, or None if conversion failed
    """
    json_proof_file = proof_file.parent / f"{proof_file.stem}_json{proof_file.suffix}"
    logger.info(f"Converting proof from Cairo-serde format to JSON format...")

    try:
        import subprocess

        result = subprocess.run(
            [
                "convert_proof_format",
                "--input",
                str(proof_file),
                "--output",
                str(json_proof_file),
                "--hash",
                "blake2s",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        logger.info(f"Successfully converted proof to JSON format: {json_proof_file}")
        return json_proof_file
    except subprocess.CalledProcessError as e:
        logger.error(f"Failed to convert proof format: {e}")
        logger.error(f"stdout: {e.stdout}")
        logger.error(f"stderr: {e.stderr}")
        return None
    except FileNotFoundError:
        logger.error(
            "convert_proof_format command not found. Please install it using: make install-convert-proof-format"
        )
        return None


def upload_to_gcs(
    proof_file: Path, chainstate_data: Dict[str, Any], mmr_roots: Dict[str, Any]
) -> bool:
    """Upload proof and chainstate data to Google Cloud Storage."""
    if storage is None:
        logger.error(
            "Google Cloud Storage not available. Please install google-cloud-storage package."
        )
        return False

    try:
        client = None

        service_account_path = os.getenv("GOOGLE_APPLICATION_CREDENTIALS")
        if service_account_path and os.path.exists(service_account_path):
            logger.debug(
                f"Using service account credentials from: {service_account_path}"
            )
            client = storage.Client.from_service_account_json(service_account_path)

        if client is None:
            logger.error("No valid GCS authentication found.")
            return False

        bucket = client.bucket(GCS_BUCKET_NAME)
        logger.debug(f"Using GCS bucket: {GCS_BUCKET_NAME}")

        timestamp = datetime.datetime.now().isoformat()

        upload_data = {
            "timestamp": timestamp,
            "chainstate": chainstate_data,
            "mmr_roots": mmr_roots,
        }

        with open(proof_file, "r") as f:
            proof_content = json.load(f)

        upload_data["proof"] = proof_content

        filename = f"proof_{chainstate_data['block_height']}_{chainstate_data['best_block_hash']}.json"

        blob = bucket.blob(filename)
        blob.upload_from_string(
            json.dumps(upload_data, indent=2), content_type="application/json"
        )

        logger.info(f"Successfully uploaded proof to GCS: {filename}")
        return True

    except Exception as e:
        logger.error(f"Failed to upload to GCS: {e}")
        logger.error(traceback.format_exc())
        return False


def build_recent_proof(
    start_height: Optional[int] = None,
    max_step: int = 1000,
    fast_data_generation: bool = True,
    max_height: Optional[int] = None,
) -> bool:
    """Main function to build a proof for the most recent Bitcoin block."""
    try:
        latest_height = get_latest_block_height()

        if start_height is None:
            start_height = auto_detect_start()
            logger.debug(f"Auto-detected start height: {start_height}")
        else:
            logger.debug(f"Using provided start height: {start_height}")

            if start_height < 0:
                logger.error("Start height cannot be negative")
                return False

        # Apply max_height constraint if specified
        if max_height is not None:
            if max_height < start_height:
                logger.error(
                    f"Max height ({max_height}) cannot be less than start height ({start_height})"
                )
                return False
            if max_height >= latest_height:
                logger.warning(
                    f"Max height ({max_height}) is greater than or equal to latest height ({latest_height}), using latest height"
                )
                max_height = None  # Use latest height instead

        # Determine the actual end height
        end_height = max_height if max_height is not None else latest_height

        if start_height >= end_height:
            logger.error(
                f"Start height ({start_height}) must be less than end height ({end_height})"
            )
            return False

        blocks_to_process = end_height - start_height

        if blocks_to_process <= 0:
            logger.info("No new blocks to process")
            return True

        step = min(max_step, blocks_to_process)

        # temporary
        # blocks_to_process = step

        logger.info(
            f"Processing {blocks_to_process} blocks from height {start_height} to {end_height}, step: {step}"
        )

        proof_file = prove_pow(
            start_height,
            blocks_to_process,
            step,
            fast_data_generation=fast_data_generation,
        )
        if proof_file is None:
            logger.error("Failed to generate proof")
            return False

        # Convert proof from Cairo-serde format to JSON format
        json_proof_file = convert_proof_to_json(proof_file)
        if json_proof_file is None:
            logger.error("Failed to convert proof to JSON format")
            return False

        from generate_data import generate_data

        data = generate_data(
            mode="light",
            initial_height=start_height + blocks_to_process - 1,
            num_blocks=1,
            fast=fast_data_generation,
        )
        chainstate_data = data["expected"]
        mmr_roots = data["mmr_roots"]

        upload_success = upload_to_gcs(json_proof_file, chainstate_data, mmr_roots)
        if not upload_success:
            logger.error("Failed to upload proof to GCS")
            return False

        # Clean up the temporary JSON proof file
        try:
            json_proof_file.unlink()
        except Exception as e:
            logger.warning(
                f"Failed to clean up temporary JSON proof file {json_proof_file}: {e}"
            )

        logger.info(
            f"Successfully built and uploaded proof for block {chainstate_data['block_height']}"
        )
        return True

    except Exception as e:
        logger.error(f"Error in build_recent_proof: {e}")
        logger.error(traceback.format_exc())
        return False


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Build validity proof for recent Bitcoin blocks"
    )
    parser.add_argument(
        "--start",
        type=int,
        help="Start block height (if not provided, will auto-detect from last proof)",
    )
    parser.add_argument(
        "--max-step",
        type=int,
        default=6000,
        help="Maximum number of blocks to process in each step (default: 6000)",
    )
    parser.add_argument(
        "--max-height",
        type=int,
        help="Maximum block height to process (if not provided, processes to latest block)",
    )
    parser.add_argument("--verbose", action="store_true", help="Verbose logging")
    parser.add_argument(
        "--slow",
        action="store_true",
        help="Use slow data generation mode (default is fast mode)",
    )

    args = parser.parse_args()

    logging_setup.setup(verbose=args.verbose)

    # Convert slow_data_generation flag to fast_data_generation parameter
    fast_data_generation = not args.slow

    success = build_recent_proof(
        args.start, args.max_step, fast_data_generation, args.max_height
    )

    if success:
        logger.info("Proof building completed successfully")
        exit(0)
    else:
        logger.error("Proof building failed")
        exit(1)
