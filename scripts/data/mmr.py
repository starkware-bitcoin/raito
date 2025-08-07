#!/usr/bin/env python3

import json
import os
import logging
from pathlib import Path
from typing import Dict, Any

logger = logging.getLogger(__name__)

# MMR configuration
MMR_ROOTS_DIR = f"{os.path.dirname(os.path.realpath(__file__))}/.mmr_data/roots"
MMR_SHARD_SIZE = 10000


def get_latest_block_height() -> int:
    """Get the latest block height available in the .mmr_data/roots directory."""
    try:
        mmr_roots_dir = Path(MMR_ROOTS_DIR)

        if not mmr_roots_dir.exists():
            logger.error(f"MMR roots directory does not exist: {mmr_roots_dir}")
            raise FileNotFoundError(f"MMR roots directory not found: {mmr_roots_dir}")

        # Find all shard directories (they are named with numbers)
        shard_dirs = []
        for item in mmr_roots_dir.iterdir():
            if item.is_dir() and item.name.isdigit():
                shard_dirs.append(int(item.name))

        if not shard_dirs:
            logger.error("No MMR shard directories found")
            raise FileNotFoundError("No MMR shard directories found")

        # Sort shard directories to find the highest one
        shard_dirs.sort()
        latest_shard = shard_dirs[-1]

        # Look for the highest block file in the latest shard
        latest_shard_dir = mmr_roots_dir / str(latest_shard)
        block_files = []

        for item in latest_shard_dir.iterdir():
            if (
                item.is_file()
                and item.name.startswith("block_")
                and item.name.endswith(".json")
            ):
                try:
                    # Extract block height from filename (block_XXXXX.json)
                    block_height = int(
                        item.name[6:-5]
                    )  # Remove "block_" prefix and ".json" suffix
                    block_files.append(block_height)
                except ValueError:
                    continue

        if not block_files:
            logger.error(
                f"No block files found in latest shard directory: {latest_shard_dir}"
            )
            raise FileNotFoundError(
                f"No block files found in latest shard directory: {latest_shard_dir}"
            )

        # Find the highest block height
        latest_height = max(block_files)
        logger.info(f"Latest available MMR block height: {latest_height}")
        return latest_height

    except Exception as e:
        logger.error(f"Failed to get latest block height from MMR directory: {e}")
        raise


def read_block_mmr_roots(height: int) -> Dict[str, Any]:
    """Read MMR roots for a specific block height.

    Args:
        height: The block height to read MMR roots for

    Returns:
        Dictionary containing the MMR roots data

    Raises:
        FileNotFoundError: If the MMR roots file doesn't exist
    """
    shard_name = (height // MMR_SHARD_SIZE + 1) * MMR_SHARD_SIZE
    mmr_roots_file = Path(MMR_ROOTS_DIR) / str(shard_name) / f"block_{height}.json"

    if not mmr_roots_file.exists():
        raise FileNotFoundError(f"MMR roots file not found: {mmr_roots_file}")

    with open(mmr_roots_file, "r") as f:
        return json.load(f)
