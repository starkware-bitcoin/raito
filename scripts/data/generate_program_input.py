#!/usr/bin/env python3

import argparse
import json
import os
from pathlib import Path


def generate_program_input(
    executable_path,
    args_file,
    program_hash_function="blake",
    output_file="program-input.json",
):
    """Generate a program-input.json file for Cairo program execution.

    Args:
        executable_path (str): Path to the Cairo executable JSON file
        args_file (str): Path to the arguments JSON file
        program_hash_function (str): Hash function to use (default: "blake")
        output_file (str): Output file path (default: program-input.json)
    """
    # Convert to absolute paths if they're relative
    executable_path = os.path.abspath(executable_path)
    args_file = os.path.abspath(args_file)

    # Create the program input structure
    program_input = {
        "single_page": True,
        "tasks": [
            {
                "type": "Cairo1Executable",
                "path": executable_path,
                "program_hash_function": program_hash_function,
                "user_args_file": args_file,
            }
        ],
    }

    # Write to output file
    with open(output_file, "w") as f:
        json.dump(program_input, f, indent=2)


def main():
    parser = argparse.ArgumentParser(
        description="Generate program-input.json for Cairo program execution"
    )

    parser.add_argument(
        "--executable", required=True, help="Path to the Cairo executable JSON file"
    )

    parser.add_argument(
        "--args-file", required=True, help="Path to the arguments JSON file"
    )

    parser.add_argument(
        "--program-hash-function",
        default="blake",
        choices=["blake", "pedersen", "poseidon"],
        help="Hash function to use for program hashing (default: blake)",
    )

    parser.add_argument(
        "--output",
        default="program-input.json",
        help="Output file path (default: program-input.json)",
    )

    args = parser.parse_args()

    # Validate that files exist
    if not os.path.exists(args.executable):
        print(f"Error: Executable file not found: {args.executable}")
        exit(1)

    if not os.path.exists(args.args_file):
        print(f"Error: Args file not found: {args.args_file}")
        exit(1)

    generate_program_input(
        args.executable, args.args_file, args.program_hash_function, args.output
    )


if __name__ == "__main__":
    main()
