#!/usr/bin/env python3

import logging
from logging.handlers import TimedRotatingFileHandler
import colorlog
from pythonjsonlogger import jsonlogger


def setup(verbose=False, log_filename=None):
    """
    Set up logging configuration with JSON file logging and colored console output.

    Args:
        verbose (bool): If True, set DEBUG level; otherwise INFO level
        log_filename (str): Name of the log file
    """

    # Root logger setup
    root_logger = logging.getLogger()

    if log_filename is not None:
        # JSON file handler setup
        file_handler = TimedRotatingFileHandler(
            filename=log_filename,
            when="midnight",
            interval=1,
            backupCount=14,
            encoding="utf8",
        )
        file_handler.setLevel(logging.INFO)
        root_logger.addHandler(file_handler)

    # Console handler
    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.DEBUG)

    if verbose:
        # Use colored formatter in verbose mode
        console_handler.setFormatter(
            colorlog.ColoredFormatter(
                "%(asctime)s - %(log_color)s%(levelname)s%(reset)s - %(message)s",
                log_colors={
                    "DEBUG": "cyan",
                    "INFO": "green",
                    "WARNING": "yellow",
                    "ERROR": "red",
                    "CRITICAL": "red,bg_white",
                },
            )
        )
    else:
        # Use simple formatter in non-verbose mode
        console_handler.setFormatter(
            logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
        )

    root_logger.addHandler(console_handler)

    # Set log level based on verbose flag
    if verbose:
        root_logger.setLevel(logging.DEBUG)
    else:
        root_logger.setLevel(logging.INFO)

    # Set specific log levels for external modules
    logging.getLogger("urllib3").setLevel(logging.WARNING)
    logging.getLogger("generate_data").setLevel(logging.WARNING)
