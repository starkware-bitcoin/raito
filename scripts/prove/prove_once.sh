#!/usr/bin/env bash
set -euo pipefail

# Orchestrate: launch one-shot Spot VM for proving.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.sh"

IMAGE_URI=""
INSTANCE_NAME="${INSTANCE_NAME:-}"

# Resolve image URI from config
if [[ -z "$PROJECT_ID" ]]; then
  echo "PROJECT_ID is required. Set it in scripts/prove/config.sh or export PROJECT_ID." >&2
  exit 1
fi

if [[ -z "$INSTANCE_NAME" ]]; then
  INSTANCE_NAME="${INSTANCE_PREFIX}-$(date +%s)"
fi

echo "$INSTANCE_NAME" > "$SCRIPT_DIR/.last_instance"

if [[ -z "$IMAGE_URI" ]]; then
  IMAGE_URI="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO}/${IMAGE}:${TAG}"
fi

if [[ ${#CONTAINER_ARGS[@]} -eq 0 ]]; then
  CONTAINER_ARGS=(prove)
fi

# Launch
"$SCRIPT_DIR/launch_spot.sh"

# Logs
if [[ ${STREAM_LOGS} -eq 1 ]]; then
  "$SCRIPT_DIR/logs.sh" || true
fi

# Cleanup
if [[ ${AUTO_DELETE} -eq 1 ]]; then
  "$SCRIPT_DIR/cleanup.sh"
fi

echo "Done. Instance: $INSTANCE_NAME"

