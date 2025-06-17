#!/usr/bin/env python3

import argparse
import json
from pathlib import Path
from format_args import format_args_to_cairo_serde


def read_proof_file(proof_path):
    """Read and parse the proof file.

    Args:
        proof_path (str): Path to the proof file

    Returns:
        list: List of hex values from the proof file
    """
    with open(proof_path, "r") as f:
        return json.loads(f.read())


def generate_assumevalid_args(block_data_path, proof_path=None):
    """Generate assumevalid arguments with optional proof.

    Args:
        block_data_path (str): Path to the block data JSON file
        proof_path (str, optional): Path to the proof file

    Returns:
        list: List of hex values representing the assumevalid arguments
    """
    # Convert block data to Cairo serde format
    result = format_args_to_cairo_serde(block_data_path)

    # Append proof indicator and proof data if available
    if proof_path:
        result.append("0x0")  # Proof exists
        proof_data = read_proof_file(proof_path)
        result.extend(proof_data)
    else:
        result.append("0x1")  # No proof (None)

    return result


def main():
    parser = argparse.ArgumentParser(
        description="Generate assumevalid arguments with optional proof."
    )

    parser.add_argument(
        "--proof-path",
        dest="proof_path",
        type=str,
        help="Path to the Cairo serde proof file",
    )

    parser.add_argument(
        "--block-data",
        dest="block_data",
        required=True,
        type=str,
        help="Path to the block data JSON file",
    )

    parser.add_argument(
        "--output-path",
        dest="output_path",
        required=True,
        type=str,
        help="Path to save the resulting JSON file",
    )

    args = parser.parse_args()

    # Generate the arguments
    result = generate_assumevalid_args(args.block_data, args.proof_path)

    # Write the result to the output file
    with open(args.output_path, "w") as f:
        json.dump(result, f, indent=2)


if __name__ == "__main__":
    main()
