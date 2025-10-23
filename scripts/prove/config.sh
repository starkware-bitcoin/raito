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

# Base image (use COS for fastest boot; includes Docker/containerd by default)
BASE_IMAGE_FAMILY=${BASE_IMAGE_FAMILY:-cos-109-lts}
BASE_IMAGE_PROJECT=${BASE_IMAGE_PROJECT:-cos-cloud}

# Container execution
# Leave empty to rely on the image ENTRYPOINT; override if needed
CONTAINER_COMMAND=${CONTAINER_COMMAND:-}
# Bash array for args; edit below to set defaults for your run
if [[ -z ${CONTAINER_ARGS_SET:-} ]]; then
  # Example defaults mirroring Makefile target rust-prove-pow
  # Edit as needed; the first non-flag should be 'prove'
  CONTAINER_ARGS=(
    --log-level debug
    --bridge-url https://staging.raito.wtf
    prove
    --executable /opt/raito/assumevalid-syscalls.executable.json
    --bootloader /opt/raito/bootloader.json
    --prover-params /opt/raito/prover_params.json
  )
  CONTAINER_ARGS_SET=1
fi

# Orchestration behavior
BUILD_FIRST=${BUILD_FIRST:-1}
STREAM_LOGS=${STREAM_LOGS:-1}
AUTO_DELETE=${AUTO_DELETE:-1}

# Instance naming
INSTANCE_PREFIX=${INSTANCE_PREFIX:-av-job}
# Leave empty to auto-generate; otherwise use this fixed name
INSTANCE_NAME=${INSTANCE_NAME:-}

# Instance labels (comma-separated k=v pairs). Used for stable log filtering.
INSTANCE_LABELS=${INSTANCE_LABELS:-job=assumevalid,component=prove}

