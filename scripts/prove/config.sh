#!/usr/bin/env bash

# Centralized configuration for proving on GCP Spot instances.
# You can override any of these by exporting env vars before calling the scripts,
# or by passing CLI flags where supported.

# Required
PROJECT_ID=${PROJECT_ID:-"exploration-dev-417917"}

# Defaults
REGION=${REGION:-us-central1}
ZONE=${ZONE:-us-central1-a}
REPO=${REPO:-raito}
IMAGE=${IMAGE:-raito-assumevalid}
TAG=${TAG:-latest}
MACHINE_TYPE=${MACHINE_TYPE:-n2-highmem-32}    # 256GB RAM
BOOT_DISK_TYPE=${BOOT_DISK_TYPE:-pd-balanced}
BOOT_DISK_SIZE_GB=${BOOT_DISK_SIZE_GB:-15}

# Container execution
CONTAINER_COMMAND=${CONTAINER_COMMAND:-/bin/raito-assumevalid}
# Bash array for args; edit below to set defaults for your run
if [[ -z ${CONTAINER_ARGS_SET:-} ]]; then
  # Example defaults mirroring Makefile target rust-prove-pow
  # Edit as needed; the first non-flag should be 'prove'
  CONTAINER_ARGS=(
    --log-level debug
    --bridge-url https://staging.raito.wtf
    prove
    --keep-temp-files
    --output-dir .rust-proofs
  )
  CONTAINER_ARGS_SET=1
fi

# Orchestration behavior
BUILD_FIRST=${BUILD_FIRST:-1}
STREAM_LOGS=${STREAM_LOGS:-0}
AUTO_DELETE=${AUTO_DELETE:-1}

# Instance naming
INSTANCE_PREFIX=${INSTANCE_PREFIX:-av-job}
# Leave empty to auto-generate; otherwise use this fixed name
INSTANCE_NAME=${INSTANCE_NAME:-}

