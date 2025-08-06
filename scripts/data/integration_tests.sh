#!/usr/bin/env bash

SCARB=scarb

GREEN='\033[0;32m'
RED='\033[1;31m'
RESET='\033[0m' # No Color

num_ok=0
num_fail=0
num_ignored=0
failures=()
test_files=()
nocapture=0
forceall=0
lightonly=0

# Process arguments
for arg in "$@"; do
  if [[ "$arg" == "--nocapture" ]]; then
    nocapture=1
  elif [[ "$arg" == "--lightonly" ]]; then
    lightonly=1
  elif [[ "$arg" == "--forceall" ]]; then
    forceall=1
  else
    test_files+=("$arg")  # Add only non-flag arguments as test files
  fi
done

ignored_files=(
)

ignore_file="tests/data/ignore"
if [[ -f "$ignore_file" ]]; then
  while IFS= read -r line; do
    ignored_files+="tests/data/$line"
  done < "$ignore_file"
fi

ignored="${ignored_files[@]}"

# If no test files are explicitly specified, default to tests/data/*
if [[ $lightonly -eq 1 && ${#test_files[@]} -eq 0 ]]; then
  test_files=("tests/data"/light*.json)
elif [[ ${#test_files[@]} -eq 0 ]]; then
  test_files=("tests/data"/*.json)
fi

echo "running integration tests ..."

for test_file in "${test_files[@]}"; do
    if [ -f "$test_file" ]; then
        echo -n "test $test_file ..."
        if [[ $forceall -ne 1 && "$ignored" =~ "$test_file" ]]; then
            echo " ignored"
            num_ignored=$((num_ignored + 1))
        else
            arguments_file="$(dirname "$test_file")/.arguments-$(basename "$test_file")"
            python ../../scripts/data/format_args.py --input_file ${test_file} > $arguments_file
            output=$($SCARB --profile release execute --no-build --print-resource-usage --arguments-file $arguments_file)
            # See https://github.com/software-mansion/scarb/pull/2276
            rm -rf ../../target/execute
            steps=$(echo $output | grep -o 'steps: [0-9,]*' | sed 's/steps: //')

            if [[ "$nocapture" -eq 1 ]]; then
                echo -e "\n$output"
            fi

            if [[ "$output" == *"FAIL"* ]]; then
                echo -e "${RED} fail ${RESET}(steps: $steps)"
                num_fail=$((num_fail + 1))
                error=$(echo $output | grep -o "error='[^']*'" | sed "s/error=//")
                failures+="\t$test_file — Panicked with $error\n"
            elif [[ "$output" == *"OK"* ]]; then
                echo -e "${GREEN} ok ${RESET}(steps: $steps)"
                num_ok=$((num_ok + 1))
                rm $arguments_file
            else
                echo -e "${RED} fail ${RESET}"
                num_fail=$((num_fail + 1))
                error=$(echo "$output" | sed '1d' | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g') #spellchecker:disable-line
                failures+="\t$test_file — $error\n"
            fi
        fi
    fi
done

if [[ $num_fail == 0 ]]; then
    echo -e "test result: ${GREEN}ok${RESET}. ${num_ok} passed; 0 failed; ${num_ignored} ignored"
else
    echo -e "failures:\n$failures"
    echo -e "test result: ${RED}FAILED${RESET}. ${num_ok} passed; ${num_fail} failed; ${num_ignored} ignored"
    false
fi