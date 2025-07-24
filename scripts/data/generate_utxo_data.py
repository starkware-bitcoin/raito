#!/usr/bin/env python3

import os
import sys
import json
import requests
import subprocess
from typing import Dict, Any
import argparse
from google.cloud import storage
from tqdm import tqdm
from functools import lru_cache
from collections import defaultdict

# Constants
BASE_DIR = f"{os.path.dirname(os.path.realpath(__file__))}/.utxo_data"
CHUNK_SIZE = 10
INDEX_SIZE = 50000

GCS_BUCKET_NAME = "shinigami-consensus"
GCS_FOLDER_NAME = "utxos"
GCS_BASE_URL = f"https://storage.googleapis.com/{GCS_BUCKET_NAME}/{GCS_FOLDER_NAME}/"


def list_files_in_gcs():
    """List all files in a GCS bucket under a specific folder (prefix)."""
    print(f"Getting file list from GCS...")
    client = storage.Client.create_anonymous_client()
    bucket = client.get_bucket(GCS_BUCKET_NAME)
    blobs = bucket.list_blobs(prefix=GCS_FOLDER_NAME)

    return [
        os.path.basename(blob.name) for blob in blobs if blob.name.endswith(".json")
    ]


def download_and_split(file_name: str):
    """Download a file from GCS and split it into chunks."""
    os.makedirs(BASE_DIR, exist_ok=True)
    file_dir = os.path.join(BASE_DIR, file_name.split(".")[0])
    os.makedirs(file_dir, exist_ok=True)

    url = f"{GCS_BASE_URL}{file_name}"
    response = requests.get(url)
    if response.status_code != 200:
        raise Exception(f"Failed to download {file_name}")

    if response.headers.get("Content-Encoding") == "gzip":
        print("Content is GZIP encoded")

    file_path = os.path.join(BASE_DIR, file_name)
    with open(file_path, "wb") as f:
        f.write(response.content)

    # Split file
    split_cmd = f"split -l {CHUNK_SIZE} {file_path} {file_dir}/"
    subprocess.run(split_cmd, shell=True, check=True)

    # Remove original file
    os.remove(file_path)


def index_file_name(key):
    return f"{BASE_DIR}/utxo_index_{key}.json"


def partition_and_dump(index, partition_size):
    """Partition the index and dump each partition to a separate file."""

    partitions = defaultdict(dict)
    for key, value in tqdm(index.items(), "Partitioning index"):
        group = int(key) // partition_size
        partitions[group][key] = value

    for key, partition in tqdm(partitions.items(), "Saving partitions"):
        with open(index_file_name(key), "w") as f:
            json.dump(partition, f)


def create_index():
    """Create or update an index mapping block numbers to chunk files."""
    index: Dict[int, str] = {}

    for dir_name in tqdm(os.listdir(BASE_DIR), "Creating index"):
        dir_path = os.path.join(BASE_DIR, dir_name)
        if not os.path.isdir(dir_path):
            continue

        for chunk_file in os.listdir(dir_path):
            chunk_path = os.path.join(dir_path, chunk_file)
            with open(chunk_path, "r") as f:
                for line_num, line in enumerate(f, 1):
                    try:
                        data = json.loads(line.strip())
                        block_number = data["block_number"]
                        if block_number in index:
                            print(f"Error: Duplicate block number {block_number}")
                            sys.exit(1)
                        index[block_number] = os.path.relpath(chunk_path, BASE_DIR)
                    except json.JSONDecodeError as e:
                        print(
                            f"Error decoding JSON in file {chunk_path}, line {line_num}: {e}"
                        )
                        print(f"Problematic line: {line.strip()[:50]}")
                        sys.exit(1)
                    except KeyError as e:
                        print(f"KeyError in file {chunk_path}, line {line_num}: {e}")
                        print(f"Data: {data}")
                        sys.exit(1)

    partition_and_dump(index, INDEX_SIZE)

    print(f"Index created or updated with {len(index)} entries")


@lru_cache(maxsize=None)
def load_index(file_name):
    if not os.path.exists(file_name):
        raise Exception(f"Index file {file_name} not found")

    with open(file_name, "r") as f:
        return json.load(f)


def get_utxo_set(block_number: int) -> Dict[str, Any]:
    index_file = index_file_name(int(block_number) // INDEX_SIZE)
    index = load_index(index_file)

    # Find chunk file
    chunk_file = index.get(str(block_number))
    if not chunk_file:
        return []
        # raise Exception(f"Block number {block_number} not found in index file: {index_file}")

    # Find and return data for the block
    with open(BASE_DIR + "/" + chunk_file, "r") as f:
        for line in f:
            # data = json.loads(line.strip())
            # if data["block_number"] == str(block_number):
            if line.startswith(f'{{"block_number":"{block_number}"'):
                data = json.loads(line.strip())
                return data["outputs"]

    raise Exception(f"Block {block_number} not found in chunk file {chunk_file}")


def process_files(num_files: int):
    """Process a range of files from start_file to end_file."""

    os.makedirs(BASE_DIR, exist_ok=True)

    files = list_files_in_gcs()

    if num_files:
        files = files[:num_files]

    for file_name in tqdm(files, desc="Downloading files"):
        # print(f"\nProcessing file: {file_name}")
        download_and_split(file_name)

    create_index()


# usage
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Process UTXO files.")
    parser.add_argument(
        "--num_files",
        dest="num_files",
        type=int,
        help="Number of files to process, all if not specified",
    )

    args = parser.parse_args()

    process_files(args.num_files)

    print("All files processed successfully.")
