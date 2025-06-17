#!/usr/bin/env python3

import json
import os
import requests
from google.cloud import storage
from tqdm import tqdm
from collections import defaultdict
from functools import lru_cache

INDEX_SIZE = 30000

BASE_DIR = f"{os.path.dirname(os.path.realpath(__file__))}/.timestamps_data"

GCS_BUCKET_NAME = "shinigami-consensus"
GCS_FOLDER_NAME = "timestamps"
GCS_BASE_URL = f"https://storage.googleapis.com/shinigami-consensus/{GCS_FOLDER_NAME}/"


def download_timestamp(file_name: str):
    """Download a file from GCS and save it locally."""
    os.makedirs(BASE_DIR, exist_ok=True)
    file_path = os.path.join(BASE_DIR, file_name)
    if os.path.exists(file_path):
        print(f"{file_name} already exists, skipping download.")
        return

    url = f"{GCS_BASE_URL}{file_name}"

    response = requests.get(url)
    if response.status_code != 200:
        raise Exception(f"Failed to download {file_name}", response)

    with open(file_path, "wb") as f:
        f.write(response.content)


def create_index(folder_path):
    index = {}
    for filename in tqdm(os.listdir(folder_path), "Creating index"):
        if filename.endswith(".json") and not "index" in filename:
            with open(os.path.join(folder_path, filename), "r") as file:
                data = [json.loads(line.rstrip()) for line in file]
                for entry in data:
                    block_number = entry["block_number"]
                    index[block_number] = entry
    return index


def list_files_in_gcs():
    """List all files in a GCS bucket under a specific folder (prefix)."""
    print(f"Getting file list from GCS...")
    client = storage.Client.create_anonymous_client()
    bucket = client.get_bucket(GCS_BUCKET_NAME)
    blobs = bucket.list_blobs(prefix=GCS_FOLDER_NAME)

    return [
        os.path.basename(blob.name) for blob in blobs if blob.name.endswith(".json")
    ]


def index_file_name(key):
    return f"{BASE_DIR}/timestamp_index_{key}.json"


def partition_and_dump(index, partition):
    """Partition the index and dump each partition to a separate file."""

    partitions = defaultdict(dict)
    for key, value in tqdm(index.items(), "Partitioning index"):
        group = int(key) // partition
        partitions[group][key] = value

    for key, partition in tqdm(partitions.items(), "Saving partitions"):
        with open(index_file_name(key), "w") as f:
            json.dump(partition, f)


@lru_cache(maxsize=None)
def load_index(file_name):
    if not os.path.exists(file_name):
        raise Exception(f"Index file {file_name} not found")

    with open(file_name, "r") as f:
        return json.load(f)


def get_timestamp_data(block_number):
    """Get the timestamp data for a given block number."""
    if block_number <= 10:
        return hardcoded_timestamp_data(block_number)
    file_name = index_file_name(int(block_number) // INDEX_SIZE)
    return load_index(file_name)[str(block_number)]


def hardcoded_timestamp_data(block_number):
    """Previous timestamp data for the first 10 chain states."""
    hardcoded_data = {
        0: {
            "block_number": "0",
            "epoch_start_time": "1231006505",
            "previous_timestamps": ["1231006505"],
        },
        1: {
            "block_number": "1",
            "epoch_start_time": "1231006505",
            "previous_timestamps": ["1231006505", "1231469665"],
        },
        2: {
            "block_number": "2",
            "epoch_start_time": "1231006505",
            "previous_timestamps": ["1231006505", "1231469665", "1231469744"],
        },
        3: {
            "block_number": "3",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
            ],
        },
        4: {
            "block_number": "4",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
            ],
        },
        5: {
            "block_number": "5",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
                "1231471428",
            ],
        },
        6: {
            "block_number": "6",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
                "1231471428",
                "1231471789",
            ],
        },
        7: {
            "block_number": "7",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
                "1231471428",
                "1231471789",
                "1231472369",
            ],
        },
        8: {
            "block_number": "8",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
                "1231471428",
                "1231471789",
                "1231472369",
                "1231472743",
            ],
        },
        9: {
            "block_number": "9",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
                "1231471428",
                "1231471789",
                "1231472369",
                "1231472743",
                "1231473279",
            ],
        },
        10: {
            "block_number": "10",
            "epoch_start_time": "1231006505",
            "previous_timestamps": [
                "1231006505",
                "1231469665",
                "1231469744",
                "1231470173",
                "1231470988",
                "1231471428",
                "1231471789",
                "1231472369",
                "1231472743",
                "1231473279",
                "1231473952",
            ],
        },
    }
    return hardcoded_data.get(block_number, {})


if __name__ == "__main__":
    file_names = list_files_in_gcs()
    for file_name in tqdm(file_names, "Downloading files"):
        download_timestamp(file_name)

    index = create_index(BASE_DIR)
    print(f"Index contains {len(index)} entries.")
    partition_and_dump(index, INDEX_SIZE)
    print("Done.")
