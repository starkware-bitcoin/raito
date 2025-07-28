#!/usr/bin/env python3

import os
import sys
import json
import subprocess
import argparse
import csv
from pathlib import Path
from typing import Dict, List, Tuple, Any
import re

# Import format_args function directly
sys.path.append(os.path.join(os.path.dirname(__file__)))
from format_args import format_args_to_cairo_serde

# Configuration
CSV_FILE = "../../docs/resource_usage_report.csv"
SCARB = "scarb"

# Builtin types for tracking
BUILTIN_TYPES = [
    "range_check_builtin",
    "bitwise_builtin",
    "poseidon_builtin",
    "pedersen_builtin",
    "range_check96_builtin",
    "ecdsa_builtin",
    "ec_op_builtin",
    "keccak_builtin",
    "mul_mod_builtin",
    "add_mod_builtin",
    "output_builtin",
]


class ResourceCollector:
    def __init__(self):
        self.test_results: Dict[str, Dict[str, Any]] = {}
        self.all_steps: List[int] = []
        self.all_memory_holes: List[int] = []
        self.all_builtins: Dict[str, int] = {}
        self.all_syscalls: Dict[str, int] = {}
        self.all_syscall_types: List[str] = []

        # Counters
        self.num_ok = 0
        self.num_fail = 0
        self.num_ignored = 0
        self.failures: List[str] = []

        # Initialize builtins tracking
        for builtin in BUILTIN_TYPES:
            self.all_builtins[builtin] = 0

    def parse_resources(self, output: str, test_name: str, status: str) -> None:
        """Parse resource usage from scarb output"""
        # Extract basic metrics
        steps_match = re.search(r"steps: (\d+)", output)
        memory_holes_match = re.search(r"memory holes: (\d+)", output)

        steps = int(steps_match.group(1)) if steps_match else 0
        memory_holes = int(memory_holes_match.group(1)) if memory_holes_match else 0

        # Extract builtins section
        builtins_section = ""
        lines = output.split("\n")
        in_builtins = False
        for line in lines:
            if "builtins:" in line:
                in_builtins = True
                builtins_section += line + "\n"
            elif in_builtins and line.strip() == ")":
                builtins_section += line + "\n"
                break
            elif in_builtins:
                builtins_section += line + "\n"

        # Parse builtins
        builtins = {}
        for builtin in BUILTIN_TYPES:
            pattern = rf'"{builtin}": (\d+)'
            match = re.search(pattern, builtins_section)
            builtins[builtin] = int(match.group(1)) if match else 0

        # Extract syscalls section
        syscalls_section = ""
        in_syscalls = False
        for line in lines:
            if "syscalls:" in line:
                in_syscalls = True
                syscalls_section += line + "\n"
            elif in_syscalls and line.strip() == ")":
                syscalls_section += line + "\n"
                break
            elif in_syscalls:
                syscalls_section += line + "\n"

        # Parse syscalls
        syscalls = {}
        syscall_pattern = r'"([^"]+)": (\d+)'
        for match in re.finditer(syscall_pattern, syscalls_section):
            syscall_name = match.group(1)
            syscall_count = int(match.group(2))
            syscalls[syscall_name] = syscall_count

        # Store results
        self.test_results[test_name] = {
            "steps": steps,
            "memory_holes": memory_holes,
            "status": status,
            "builtins": builtins,
            "syscalls": syscalls,
        }

        # Add to global arrays for statistics
        if steps > 0:
            self.all_steps.append(steps)
            self.all_memory_holes.append(memory_holes)

        # Accumulate builtins
        for builtin, count in builtins.items():
            self.all_builtins[builtin] += count

        # Accumulate syscalls
        for syscall, count in syscalls.items():
            self.all_syscalls[syscall] = self.all_syscalls.get(syscall, 0) + count

            # Track unique syscall types for CSV columns
            if syscall not in self.all_syscall_types:
                self.all_syscall_types.append(syscall)

    def generate_csv_header(self) -> List[str]:
        """Generate CSV header"""
        header = ["test_type", "block_number", "status", "steps", "memory_holes"]

        # Add builtin columns
        header.extend(BUILTIN_TYPES)

        # Add syscall columns
        header.extend(self.all_syscall_types)

        return header

    def parse_test_name(self, test_name: str) -> Tuple[str, str]:
        """Parse test name into test type and block number"""
        parts = test_name.split("_")
        if len(parts) >= 2:
            test_type = parts[0]
            try:
                # Try to extract block number from the second part
                block_number = parts[1]
                # Validate it's a number
                int(block_number)
                return test_type, block_number
            except ValueError:
                # If second part is not a number, it might be part of the type
                if len(parts) >= 3:
                    try:
                        block_number = parts[2]
                        int(block_number)
                        return f"{parts[0]}_{parts[1]}", block_number
                    except ValueError:
                        pass
                # Fallback: treat everything after first underscore as block number
                return test_type, "_".join(parts[1:])
        else:
            # Single word test name
            return test_name, ""

    def write_csv(self) -> None:
        """Write CSV file with all collected data"""
        with open(CSV_FILE, "w", newline="") as csvfile:
            writer = csv.writer(csvfile)

            # Write header
            header = self.generate_csv_header()
            writer.writerow(header)

            # Sort test names by type and block number
            test_names = []
            for test_name in self.test_results.keys():
                test_names.append(test_name)

            # Sort by type (light, full, utreexo) and then by block number
            def sort_key(name):
                test_type, block_number = self.parse_test_name(name)
                try:
                    block_num = int(block_number) if block_number else 0
                    return (test_type, block_num)
                except ValueError:
                    return (test_type, 0)

            test_names.sort(key=sort_key)

            # Write data rows
            for test_name in test_names:
                test_data = self.test_results[test_name]
                test_type, block_number = self.parse_test_name(test_name)

                row = [
                    test_type,
                    block_number,
                    test_data["status"],
                    test_data["steps"],
                    test_data["memory_holes"],
                ]

                # Add builtin values
                for builtin in BUILTIN_TYPES:
                    row.append(test_data["builtins"].get(builtin, 0))

                # Add syscall values
                for syscall in self.all_syscall_types:
                    row.append(test_data["syscalls"].get(syscall, 0))

                writer.writerow(row)

    def load_ignored_files(self, ignore_file: str) -> List[str]:
        """Load list of ignored files"""
        ignored_files = []
        if os.path.exists(ignore_file):
            with open(ignore_file, "r") as f:
                for line in f:
                    line = line.strip()
                    if line:
                        ignored_files.append(f"tests/data/{line}")
        return ignored_files

    def run_test(
        self, test_file: str, nocapture: bool, forceall: bool, ignored_files: List[str]
    ) -> None:
        """Run a single test and collect resources"""
        if not os.path.exists(test_file):
            return

        print(f"test {test_file} ...", end="", flush=True)

        if not forceall and test_file in ignored_files:
            print(" ignored")
            self.num_ignored += 1
            return

        # Create arguments file
        arguments_file = (
            f"{os.path.dirname(test_file)}/.arguments-{os.path.basename(test_file)}"
        )

        try:
            # Format arguments directly using the imported function
            formatted_args = format_args_to_cairo_serde(test_file)
            with open(arguments_file, "w") as f:
                json.dump(formatted_args, f)

            # Run scarb
            result = subprocess.run(
                [
                    SCARB,
                    "--profile",
                    "proving",
                    "execute",
                    "--no-build",
                    "--print-resource-usage",
                    "--arguments-file",
                    arguments_file,
                ],
                capture_output=True,
                text=True,
                check=False,
            )

            # Clean up execute directory
            execute_dir = "../../target/execute"
            if os.path.exists(execute_dir):
                subprocess.run(["rm", "-rf", execute_dir])

            output = result.stdout + result.stderr

            if nocapture:
                print(f"\n{output}")

            test_name = os.path.splitext(os.path.basename(test_file))[0]

            if "FAIL" in output:
                print("\033[1;31m fail \033[0m")
                self.num_fail += 1
                error_match = re.search(r"error='([^']*)'", output)
                error = error_match.group(1) if error_match else "Unknown error"
                self.failures.append(f"\t{test_file} — Panicked with {error}")
                self.parse_resources(output, test_name, "FAIL")

            elif "OK" in output:
                print("\033[0;32m ok \033[0m")
                self.num_ok += 1
                os.remove(arguments_file)
                self.parse_resources(output, test_name, "OK")

            else:
                print("\033[1;31m fail \033[0m")
                self.num_fail += 1
                error = output.strip().replace("\n", " ")
                self.failures.append(f"\t{test_file} — {error}")
                self.parse_resources(output, test_name, "ERROR")

        except subprocess.CalledProcessError as e:
            print("\033[1;31m fail \033[0m")
            self.num_fail += 1
            self.failures.append(f"\t{test_file} — Subprocess error: {e}")
        except Exception as e:
            print("\033[1;31m fail \033[0m")
            self.num_fail += 1
            self.failures.append(f"\t{test_file} — Exception: {e}")

    def run(
        self,
        test_files: List[str],
        nocapture: bool,
        forceall: bool,
        lightonly: bool,
        collect_only: bool,
    ) -> None:
        """Main execution function"""
        print("\033[0;34mCollecting resource usage data...\033[0m")

        # Load ignored files
        ignored_files = self.load_ignored_files("tests/data/ignore")

        # Determine test files if not specified
        if not test_files:
            if lightonly:
                test_files = list(Path("tests/data").glob("light*.json"))
            else:
                test_files = list(Path("tests/data").glob("*.json"))

        # Run tests
        for test_file in test_files:
            self.run_test(str(test_file), nocapture, forceall, ignored_files)

        # Generate CSV report
        print("\n\033[0;34mGenerating CSV report...\033[0m")
        self.write_csv()

        # Print summary
        print("\n\033[0;32mResource collection completed!\033[0m")
        print(f"CSV report generated: \033[0;34m{CSV_FILE}\033[0m")

        if self.num_fail == 0:
            print(
                f"\ntest result: \033[0;32mok\033[0m. {self.num_ok} passed; 0 failed; {self.num_ignored} ignored"
            )
        else:
            print("\nfailures:")
            for failure in self.failures:
                print(failure)
            print(
                f"\ntest result: \033[1;31mFAILED\033[0m. {self.num_ok} passed; {self.num_fail} failed; {self.num_ignored} ignored"
            )
            sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Collect resource usage data from Scarb tests"
    )
    parser.add_argument("--nocapture", action="store_true", help="Show test output")
    parser.add_argument("--lightonly", action="store_true", help="Only run light tests")
    parser.add_argument(
        "--forceall",
        action="store_true",
        help="Force run all tests including ignored ones",
    )
    parser.add_argument(
        "--collect-only", action="store_true", help="Only collect data, don't run tests"
    )
    parser.add_argument("test_files", nargs="*", help="Specific test files to run")

    args = parser.parse_args()

    collector = ResourceCollector()
    collector.run(
        args.test_files,
        args.nocapture,
        args.forceall,
        args.lightonly,
        args.collect_only,
    )


if __name__ == "__main__":
    main()
