#!/usr/bin/env python3

from dataclasses import dataclass
import json
import re
import os
import threading
import queue
import argparse
import subprocess
import logging
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import random
import signal
from generate_data import generate_data
from format_args import format_args
from logging.handlers import TimedRotatingFileHandler

logger = logging.getLogger(__name__)

# Constants
MAX_WEIGHT_LIMIT = 8000  # Total weight limit for all jobs
THREAD_POOL_SIZE = os.cpu_count()  # Number of threads for processing
QUEUE_MAX_SIZE = THREAD_POOL_SIZE * 2  # Maximum size of the job queue

BASE_DIR = Path(".client_cache")

# Shared state variables
current_weight = 0
weight_lock = threading.Condition()
job_queue = queue.Queue(maxsize=QUEUE_MAX_SIZE)
shutdown_requested = False


class ShutdownRequested(Exception):
    """Raised when shutdown is requested during process execution"""

    pass


def run(cmd, timeout=None):
    """
    Run a subprocess with proper shutdown handling
    """

    global shutdown_requested

    if shutdown_requested:
        raise ShutdownRequested("Shutdown requested before process start")

    process = subprocess.Popen(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )

    try:
        stdout, stderr = process.communicate(timeout=timeout)

        if shutdown_requested:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            raise ShutdownRequested("Shutdown requested during process execution")

        return stdout, stderr, process.returncode

    except subprocess.TimeoutExpired:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        raise


# Function to calculate weight of a block
def calculate_batch_weight(block_data, mode):
    if mode == "light":
        return len(block_data["blocks"])
    else:
        return sum(
            len(tx["inputs"]) + len(tx["outputs"])
            for block in block_data["blocks"]
            for tx in block["data"]["transactions"]
        )


@dataclass
class Job:
    height: int
    step: int
    mode: str
    weight: int
    batch_file: Path

    def __str__(self):
        return f"Job(height='{self.height}', step={self.step}, weight='{self.weight}')"


# Generator function to create jobs
def job_generator(start, blocks, step, mode, strategy):
    BASE_DIR.mkdir(exist_ok=True)
    end = start + blocks

    height_range, step = (
        ([random.randint(start, end) for _ in range(start, end)], 1)
        if strategy == "random"
        else (range(start, end, step), step)
    )

    for height in height_range:
        try:
            batch_file = BASE_DIR / f"{mode}_{height}_{step}.json"

            batch_data = generate_data(
                mode=mode, initial_height=height, num_blocks=step, fast=True
            )

            Path(batch_file).write_text(json.dumps(batch_data, indent=2))

            batch_weight = calculate_batch_weight(batch_data, mode)
            yield Job(
                height,
                step,
                mode,
                batch_weight,
                batch_file,
            ), batch_weight
        except Exception as e:
            logger.error(f"Error while generating data for: {height}:\n{e}")


def process_batch(job):
    arguments_file = job.batch_file.as_posix().replace(".json", "-arguments.json")

    with open(arguments_file, "w") as af:
        af.write(str(format_args(job.batch_file)))

    stdout, stderr, returncode = run(
        [
            "scarb",
            "--profile",
            "proving",
            "execute",
            "--no-build",
            "--package",
            "client",
            "--arguments-file",
            str(arguments_file),
        ]
    )

    if returncode != 0 or "FAIL" in stdout or "error" in stdout or "panicked" in stdout:
        error = stdout or stderr
        if returncode == -9:
            match = re.search(r"gas_spent=(\d+)", stdout)
            gas_info = (
                f", gas spent: {int(match.group(1))}"
                if match
                else ", no gas info found"
            )
            error = f"Return code -9, killed by OOM?{gas_info}"
            message = error
        else:
            error_match = re.search(r"error='([^']*)'", error)
            if error_match:
                message = error_match.group(1)
            else:
                error_match = re.search(r"error: (.*)", error, re.DOTALL)

                if error_match:
                    message = error_match.group(1)
                else:
                    message = error

        message = re.sub(r"\s+", " ", message)

        logger.error(f"{job} error: {message}")
        logger.debug(f"Full error while processing: {job}:\n{error}")
    else:
        match = re.search(r"gas_spent=(\d+)", stdout)
        gas_info = f"gas spent: {int(match.group(1))}" if match else "no gas info found"
        logger.info(f"{job} done, {gas_info}")
        if not match:
            logger.warning(f"{job}: no gas info found")


# Producer function: Generates data and adds jobs to the queue
def job_producer(job_gen):
    global current_weight
    global shutdown_requested

    try:
        for job, weight in job_gen:
            # if shutdown_requested:
            #     break

            # Wait until there is enough weight capacity to add the new block
            with weight_lock:
                logger.debug(
                    f"Adding job: {job}, current total weight: {current_weight}..."
                )
                while not shutdown_requested and (
                    (current_weight + weight > MAX_WEIGHT_LIMIT)
                    and current_weight != 0
                    or job_queue.full()
                ):
                    logger.debug("Producer is waiting for weight to be released.")
                    weight_lock.wait(timeout=1.0)

                if shutdown_requested:
                    break

                if (current_weight + weight > MAX_WEIGHT_LIMIT) and current_weight == 0:
                    logger.warning(f"{job} over the weight limit: {MAX_WEIGHT_LIMIT}")

                # Add the job to the queue and update the weight
                job_queue.put((job, weight))
                current_weight += weight
                logger.debug(
                    f"Produced job: {job}, current total weight: {current_weight}"
                )

                # Notify consumers that a new job is available
                weight_lock.notify_all()
    finally:
        logger.debug("Producer is exiting...")
        # Signal end of jobs to consumers
        if not shutdown_requested:
            for _ in range(THREAD_POOL_SIZE):
                job_queue.put(None, block=False)

        with weight_lock:
            weight_lock.notify_all()

        logger.debug("Consumers notified")


# Consumer function: Processes blocks from the queue
def job_consumer(process_job):
    global current_weight
    global shutdown_requested

    while not shutdown_requested:
        try:
            logger.debug(
                f"Consumer is waiting for a job. Queue length: {job_queue.qsize()}"
            )
            # Get a job from the queue with timeout to check shutdown
            try:
                work_to_do = job_queue.get(timeout=1.0)
            except queue.Empty:
                continue

            if work_to_do is None:
                logger.debug("No more work to do, consumer is exiting.")
                job_queue.task_done()
                break

            (job, weight) = work_to_do

            if shutdown_requested:
                with weight_lock:
                    current_weight -= weight
                    weight_lock.notify_all()
                job_queue.task_done()
                break

            # Process the block
            try:
                logger.debug(f"Executing job: {job}...")
                process_job(job)
            except ShutdownRequested:
                logger.debug(f"Shutdown requested while processing {job}")
                return
            except subprocess.TimeoutExpired:
                logger.warning(f"Timeout while terminating subprocess for {job}")
            except Exception as e:
                logger.error(f"Error while processing job: {job}:\n{e}")

            with weight_lock:
                current_weight -= weight
                logger.debug(
                    f"Finished processing job, current total weight: {current_weight}"
                )
                weight_lock.notify_all()  # Notify producer to add more jobs

            # Mark job as done
            job_queue.task_done()

        except Exception as e:
            # TODO: is this necessary?
            if not shutdown_requested:
                logger.error("Error in the consumer: %s", e)
            break

    logger.debug("Job consumer done.")


def main(start, blocks, step, mode, strategy):
    global shutdown_requested

    # Set up signal handlers
    def signal_handler(signum, frame):
        global shutdown_requested
        signal_name = signal.Signals(signum).name
        logger.info(f"Received signal {signal_name}. Initiating graceful shutdown...")
        shutdown_requested = True

    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    logger.info(
        "Starting client, initial height: %d, blocks: %d, step: %d, mode: %s, strategy: %s",
        start,
        blocks,
        step,
        mode,
        strategy,
    )
    logger.info(
        "Max weight limit: %d, Thread pool size: %d, Queue max size: %d",
        MAX_WEIGHT_LIMIT,
        THREAD_POOL_SIZE,
        QUEUE_MAX_SIZE,
    )

    # Create the job generator
    job_gen = job_generator(start, blocks, step, mode, strategy)

    # Start the job producer thread
    producer_thread = threading.Thread(target=job_producer, args=(job_gen,))
    producer_thread.start()

    # Start the consumer threads using ThreadPoolExecutor
    with ThreadPoolExecutor(max_workers=THREAD_POOL_SIZE) as executor:
        futures = [
            executor.submit(job_consumer, process_batch)
            for _ in range(THREAD_POOL_SIZE)
        ]

        # Wait for producer to finish or shutdown
        producer_thread.join()

        # Wait for all items in the queue to be processed or shutdown
        while not shutdown_requested and not job_queue.empty():
            try:
                job_queue.join()
                break
            except KeyboardInterrupt:
                shutdown_requested = True

    if shutdown_requested:
        logger.info("Shutdown complete.")
    else:
        logger.info("All jobs have been processed.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run client script")
    parser.add_argument("--start", type=int, required=True, help="Start block height")
    parser.add_argument(
        "--blocks",
        type=int,
        default=1,
        help="Number of blocks to process",
    )
    parser.add_argument(
        "--step", type=int, default=1, help="Step size for block processing"
    )
    parser.add_argument(
        "--mode",
        default="light",
        choices=["light", "full", "utreexo"],
        help="Client mode",
    )
    parser.add_argument(
        "--strategy",
        default="sequential",
        choices=["sequential", "random"],
        help="Processing strategy",
    )

    parser.add_argument(
        "--maxweight", type=int, default=MAX_WEIGHT_LIMIT, help="Max weight limit"
    )

    parser.add_argument("--verbose", action="store_true", help="Verbose")

    args = parser.parse_args()

    MAX_WEIGHT_LIMIT = args.maxweight

    # Logging setup
    file_handler = TimedRotatingFileHandler(
        filename="client.log",
        when="midnight",
        interval=1,
        backupCount=14,
        encoding="utf8",
    )
    file_handler.setLevel(logging.INFO)
    file_handler.setFormatter(
        logging.Formatter("%(asctime)s - %(name)-10.10s - %(levelname)s - %(message)s")
    )

    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.DEBUG)
    console_handler.setFormatter(
        logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
    )
    root_logger = logging.getLogger()
    root_logger.addHandler(console_handler)
    root_logger.addHandler(file_handler)

    if args.verbose:
        root_logger.setLevel(logging.DEBUG)
    else:
        root_logger.setLevel(logging.INFO)

    logging.getLogger("urllib3").setLevel(logging.WARNING)
    logging.getLogger("generate_data").setLevel(logging.WARNING)

    main(
        args.start,
        args.blocks,
        args.step,
        args.mode,
        args.strategy,
    )
