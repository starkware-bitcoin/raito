#!/usr/bin/env bash

#set -e;
#set -o pipefail;

base_dir=".client_cache"

start=${1:-0}
no_of_blocks=${2:-100}
end=$(($start+$no_of_blocks))
step=${3:-1}
mode=${4:-"light"}
strategy=${5:-"sequential"}

mkdir -p $base_dir

run_client() {
  local initial_height=$1
  local num_blocks=$2
  
  first=$((initial_height+1))
  second=$((initial_height+num_blocks))

  batch_file=${base_dir}/${mode}_${initial_height}_${num_blocks}.json
  arguments_file=${base_dir}/arguments-${mode}_${initial_height}_${num_blocks}.json
  if [ ! -f "$batch_file" ]; then
    python ../../scripts/data/generate_data.py --fast --mode $mode --height $initial_height --num_blocks $num_blocks --output_file $batch_file
  fi

  echo -n "Running $mode client on blocks $first - $second "
  python ../../scripts/data/format_args.py --input_file $batch_file > $arguments_file
  output=$(scarb --profile release execute --no-build --package client --arguments-file $arguments_file)
  if [[ $? -ne 0 || "$output" == *"FAIL"* || "$output" == *error* || "$output" == *panicked* ]]; then
    echo "fail"
    echo $output
    exit 1
  else
    echo "ok"
    echo $output
  fi
}

for (( height=start; height<end; height+=step )); do
  if [[ "$strategy" == "sequential" ]]; then
    run_client $height $step
  elif [[ "$strategy" == "random" ]]; then
    random_height=$(( RANDOM * 25 + RANDOM % 25 ))
    run_client $random_height 1
  else
    echo "Unsupported strategy"
    exit 1
  fi
done
