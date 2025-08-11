#!/usr/bin/env python3

import json
import os
import logging
import requests
from pathlib import Path
from typing import Dict, Any

logger = logging.getLogger(__name__)

# MMR configuration
MMR_ROOTS_DIR = f"{os.path.dirname(os.path.realpath(__file__))}/.mmr_data/roots"
MMR_SHARD_SIZE = 10000
RAITO_API_URL = "https://api.raito.wtf/head"
RAITO_ROOTS_API_URL = "https://api.raito.wtf/roots"


def get_latest_block_height() -> int:
    """Get the latest block height from the Raito API."""
    try:
        logger.debug(f"Fetching latest block height from {RAITO_API_URL}")

        response = requests.get(RAITO_API_URL, timeout=10)
        response.raise_for_status()  # Raise an exception for bad status codes

        # The API returns just the block height as plain text
        latest_height = int(response.text.strip())

        logger.debug(f"Latest block height from API: {latest_height}")
        return latest_height

    except requests.RequestException as e:
        logger.error(f"Failed to fetch latest block height from API: {e}")
        raise
    except ValueError as e:
        logger.error(f"Invalid response from API (expected integer): {e}")
        raise
    except Exception as e:
        logger.error(f"Unexpected error while fetching latest block height: {e}")
        raise


def read_block_mmr_roots(height: int) -> Dict[str, Any]:
    """Read MMR roots for a specific block height from the Raito API.

    Args:
        height: The block height to read MMR roots for

    Returns:
        Dictionary containing the MMR roots data

    Raises:
        requests.RequestException: If the API request fails
        ValueError: If the API response is invalid
    """
    try:
        logger.debug(
            f"Fetching MMR roots for block height {height} from {RAITO_ROOTS_API_URL}"
        )

        response = requests.get(
            f"{RAITO_ROOTS_API_URL}?chain_height={height}", timeout=10
        )
        response.raise_for_status()  # Raise an exception for bad status codes

        data = response.json()

        # Validate the response structure
        if "roots" not in data:
            raise ValueError("Invalid API response: missing 'roots' field")

        logger.debug(f"Successfully fetched MMR roots for block height {height}")
        return data

    except requests.RequestException as e:
        logger.error(f"Failed to fetch MMR roots from API for height {height}: {e}")
        raise
    except json.JSONDecodeError as e:
        logger.error(f"Invalid JSON response from API for height {height}: {e}")
        raise
    except ValueError as e:
        logger.error(f"Invalid response structure from API for height {height}: {e}")
        raise
    except Exception as e:
        logger.error(
            f"Unexpected error while fetching MMR roots for height {height}: {e}"
        )
        raise
