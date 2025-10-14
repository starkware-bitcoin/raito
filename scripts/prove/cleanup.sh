#!/usr/bin/env bash
set -euo pipefail

# Delete the Spot instance to stop billing.

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

gcloud compute instances delete "$INSTANCE_NAME" --zone="$ZONE" --quiet
echo "Deleted instance: $INSTANCE_NAME"

