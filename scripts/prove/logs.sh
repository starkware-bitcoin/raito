#!/usr/bin/env bash
set -euo pipefail

# Stream serial console logs for the Spot instance (Container-Optimized OS forwards container logs).

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.sh"

INSTANCE_NAME=""

LAST_FILE="$SCRIPT_DIR/.last_instance"
if [[ -z "$INSTANCE_NAME" && -f "$LAST_FILE" ]]; then
  INSTANCE_NAME="$(cat "$LAST_FILE")"
fi

if [[ -z "$INSTANCE_NAME" ]]; then
  echo "No instance specified and $LAST_FILE not found. Set INSTANCE_NAME in config.sh or create $LAST_FILE." >&2
  exit 1
fi

gcloud compute instances get-serial-port-output "$INSTANCE_NAME" \
  --zone="$ZONE" \
  --port=2 \
  --start=0 | sed -e 's/\r//g'

