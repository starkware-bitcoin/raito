#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


def serialize(obj):
    """Serializes Cairo data in JSON format to a Python object with reduced types.
    Supports the following conversions:
        bool -> int  # bool = felt252
        integer -> int  # felt252
        dec string (0-9) -> (int, int) -> u256 = { lo: felt252, hi: felt252 }
        hex string (0-F), 64 len -> (int, int, int, int, int, int, int, int) -> Hash !reversed!
        hex string 0x prefixed -> ([int, ...], int, int) -> ByteArray
        list -> tuple(len(list), *list)
        dict -> tuple(dict.values)
    """
    if isinstance(obj, bool):
        return 1 if obj else 0
    elif isinstance(obj, int):
        # This covers u8, u16, u32, u64, u128, felt252
        assert obj >= 0 and obj < 2**252
        return obj
    elif isinstance(obj, str):
        if obj == "0" * 64:
            # special case - zero hash
            return (0, 0, 0, 0, 0, 0, 0, 0)
        elif obj.isdigit():
            # TODO: there might still be collisions with hashes
            # Try to cast to int and then to low/high parts
            num = int(obj)
            assert num >= 0 and num < 2**256
            lo = num % 2**128
            hi = num // 2**128
            return (lo, hi)
        elif obj.startswith("0x"):
            # Split into 31-byte chunks and save the remainder
            src = bytes.fromhex(obj[2:])
            num_chunks = len(src) // 31
            main_len = num_chunks * 31
            rem_len = len(src) - main_len
            main = [
                int.from_bytes(src[i : i + 31], "big") for i in range(0, main_len, 31)
            ]
            # TODO: check if this is how byte31 is implemented
            rem = int.from_bytes(src[main_len:].rjust(31, b"\x00"), "big")
            return tuple([len(main)] + main + [rem, rem_len])
        else:
            # Reversed hex string into 4-byte words then into BE u32
            assert len(obj) == 64, f"expected 32-byte hash: {obj}"
            rev = list(reversed(bytes.fromhex(obj)))
            return tuple(int.from_bytes(rev[i : i + 4], "big") for i in range(0, 32, 4))
    elif isinstance(obj, list):
        arr = list(map(serialize, obj))
        return tuple([len(arr)] + arr)
    elif isinstance(obj, dict):
        # Inline dict properties that have keys starting with "_"
        return tuple(
            tuple(v) if k.startswith("_") else serialize(v) for k, v in obj.items()
        )
    elif isinstance(obj, tuple):
        return obj
    elif obj is None:
        # Option::None
        return 1
    else:
        raise NotImplementedError(obj)


def flatten_tuples(src) -> list:
    """Recursively flattens tuples.
    Example: (0, (1, 2), [(3, 4, [5, 6])]) -> [0, 1, 2, [3, 4, [5, 6]]]

    :param src: an object that can be int|list|tuple or their nested combination.
    :return: an object that can only contain integers and lists, top-level tuple converts to a list.
    """
    res = []

    def append_obj(obj, to):
        if isinstance(obj, int):
            to.append(obj)
        elif isinstance(obj, list):
            inner = []
            for item in obj:
                append_obj(item, inner)
            to.append(inner)
        elif isinstance(obj, tuple):
            for item in obj:
                append_obj(item, to)
        else:
            raise NotImplementedError(obj)

    append_obj(src, res)
    return res


def format_args_to_cairo_serde(input_file):
    """Reads arguments from JSON file and returns formatted result as a list of hex values.
    Output is compatible with the Scarb runner arguments format.

    Args:
        input_file (str): Path to the input JSON file

    Returns:
        list: List of hex values representing the Cairo serde format
    """
    args = json.loads(Path(input_file).read_text())
    res = flatten_tuples(serialize(args))
    return list(map(hex, res))


def format_args(input_file):
    """Reads arguments from JSON file and returns formatted result as a list of hex values.
    Output is compatible with the Scarb runner arguments format.

    Args:
        input_file (str): Path to the input JSON file
    """
    return json.dumps(format_args_to_cairo_serde(input_file))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Prepare arguments for Scarb runner.")

    parser.add_argument(
        "--input_file",
        dest="input_file",
        required=True,
        type=str,
        help="Input file with arguments in JSON format",
    )

    args = parser.parse_args()

    print(format_args(args.input_file))
