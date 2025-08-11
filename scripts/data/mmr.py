import logging
import requests
from pathlib import Path
from typing import Dict, Any

logger = logging.getLogger(__name__)

RAITO_API_URL = "https://api.raito.wtf/head"
RAITO_ROOTS_API_URL = "https://api.raito.wtf/roots"


def get_latest_block_height() -> int:
    logger.debug(f"Fetching latest block height from {RAITO_API_URL}")

    response = requests.get(RAITO_API_URL, timeout=10)
    response.raise_for_status()  # Raise an exception for bad status codes

    # The API returns just the block height as plain text
    latest_height = int(response.text.strip())

    logger.debug(f"Latest block height from API: {latest_height}")
    return latest_height


def read_block_mmr_roots(height: int) -> Dict[str, Any]:
    logger.debug(
        f"Fetching MMR roots for block height {height} from {RAITO_ROOTS_API_URL}"
    )

    response = requests.get(f"{RAITO_ROOTS_API_URL}?chain_height={height}", timeout=10)
    response.raise_for_status()  # Raise an exception for bad status codes

    data = response.json()

    # Validate the response structure
    if "roots" not in data:
        raise ValueError("Invalid API response: missing 'roots' field")

    logger.debug(f"Successfully fetched MMR roots for block height {height}")
    return data
