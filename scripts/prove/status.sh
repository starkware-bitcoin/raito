#!/usr/bin/env bash
set -euo pipefail

# Fix locale warnings from gcloud/perl
export LC_ALL=C.UTF-8

# Check the status of the Spot instance.

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

STATUS=$(gcloud compute instances describe "$INSTANCE_NAME" --zone="$ZONE" --format='value(status)' 2>/dev/null || echo "NOT_FOUND")
echo "Instance: $INSTANCE_NAME"
echo "Status: $STATUS"

