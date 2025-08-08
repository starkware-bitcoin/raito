#!/usr/bin/env python3

import json
import os
import argparse
import logging
import time
import datetime
import gzip
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
from mmr import get_latest_block_height
import logging_setup

logger = logging.getLogger(__name__)

GCS_BUCKET_NAME = os.getenv("GCS_BUCKET_NAME", "raito-proofs")


def convert_proof_to_json(proof_file: Path) -> Optional[Path]:
    """Convert proof from Cairo-serde format to JSON format.

    Args:
        proof_file: Path to the original proof file in Cairo-serde format

    Returns:
        Path to the converted JSON proof file, or None if conversion failed
    """
    json_proof_file = proof_file.parent / f"{proof_file.stem}_json{proof_file.suffix}"
    logger.debug(f"Converting proof from Cairo-serde format to JSON format...")

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


def compress_proof_data(proof_file: Path) -> Optional[Path]:
    """Compress the proof data using gzip."""
    compressed_proof_file = proof_file.parent / f"{proof_file.stem}_json.gz"
    logger.debug(
        f"Compressing proof data from {proof_file} to {compressed_proof_file}..."
    )

    try:
        with open(proof_file, "rb") as f_in, gzip.open(
            compressed_proof_file, "wb"
        ) as f_out:
            f_out.writelines(f_in)
        logger.info(f"Successfully compressed proof data: {compressed_proof_file}")
        return compressed_proof_file
    except Exception as e:
        logger.error(f"Failed to compress proof data: {e}")
        logger.error(traceback.format_exc())
        return None


def upload_to_gcs(proof_file: Path, chainstate_data: Dict[str, Any]) -> bool:
    """Upload compressed proof and chainstate data to Google Cloud Storage."""
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

        # Read the compressed proof data
        with gzip.open(proof_file, "rt") as f:
            proof_content = json.load(f)

        upload_data = {
            "timestamp": timestamp,
            "chainstate": chainstate_data,
            "proof": proof_content,
        }

        # Create filename without extension
        filename = f"proof_{chainstate_data['block_height']}_{chainstate_data['best_block_hash']}"

        # Compress the entire upload data
        compressed_data = gzip.compress(
            json.dumps(upload_data, indent=2).encode("utf-8")
        )

        blob = bucket.blob(filename)
        blob.content_encoding = "gzip"
        blob.upload_from_string(compressed_data, content_type="application/json")

        # Copy the blob to recent_proof
        recent_proof_blob = bucket.blob("recent_proof")
        recent_proof_blob.content_encoding = "gzip"
        bucket.copy_blob(blob, bucket, "recent_proof")

        logger.debug(f"Successfully uploaded compressed proof to GCS: {filename}")
        logger.debug(f"Successfully copied compressed proof to recent_proof")
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
    """Main function to build a proof for the most recent Bitcoin block.

    Args:
        start_height: Starting block height (auto-detected if None)
        max_step: Maximum number of blocks to process in each step
        fast_data_generation: Whether to use fast data generation mode
        max_height: Maximum block height to process (uses latest if None)
    """
    proof_file = None
    proof_dir = None

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

        if start_height > end_height:
            logger.error(
                f"Start height ({start_height}) must be less than or equal to end height ({end_height})"
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

        # Store the proof directory for potential cleanup
        proof_dir = proof_file.parent

        # Convert proof from Cairo-serde format to JSON format
        json_proof_file = convert_proof_to_json(proof_file)
        if json_proof_file is None:
            logger.error("Failed to convert proof to JSON format")
            return False

        # Compress the JSON proof file
        compressed_proof_file = compress_proof_data(json_proof_file)
        if compressed_proof_file is None:
            logger.error("Failed to compress proof data")
            return False

        from generate_data import generate_data

        data = generate_data(
            mode="light",
            initial_height=start_height + blocks_to_process - 1,
            num_blocks=1,
            fast=fast_data_generation,
            mmr_roots=False,
        )
        chainstate_data = data["expected"]

        upload_success = upload_to_gcs(compressed_proof_file, chainstate_data)
        if not upload_success:
            logger.error("Failed to upload proof to GCS")
            return False

        # Clean up the temporary files
        try:
            json_proof_file.unlink()
            compressed_proof_file.unlink()
        except Exception as e:
            logger.warning(
                f"Failed to clean up temporary files {json_proof_file} or {compressed_proof_file}: {e}"
            )

        logger.info(
            f"Successfully built and uploaded proof for block {chainstate_data['block_height']}"
        )
        return True

    except Exception as e:
        logger.error(f"Error in build_recent_proof: {e}")
        logger.error(traceback.format_exc())

        # Clean up the proof directory if it exists
        if proof_dir is not None and proof_dir.exists():
            try:
                import shutil

                shutil.rmtree(proof_dir)
                logger.debug(f"Cleaned up proof directory: {proof_dir}")
            except Exception as cleanup_error:
                logger.error(
                    f"Failed to clean up proof directory {proof_dir}: {cleanup_error}"
                )

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

    logging_setup.setup(
        verbose=args.verbose, log_filename="/var/log/raito/build_recent_proof.log"
    )

    # Convert slow_data_generation flag to fast_data_generation parameter
    fast_data_generation = not args.slow

    success = build_recent_proof(
        args.start, args.max_step, fast_data_generation, args.max_height
    )

    if success:
        exit(0)
    else:
        exit(1)
