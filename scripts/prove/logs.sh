#!/usr/bin/env bash
set -euo pipefail

# Fix locale warnings from gcloud/perl
export LC_ALL=C.UTF-8

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

START=0
PORT=1
while true; do
  OUTPUT=$(gcloud compute instances get-serial-port-output "$INSTANCE_NAME" \
    --zone="$ZONE" \
    --port="$PORT" \
    --start="$START" 2>&1 || true)

  # Print only startup and raito lines, normalized and cleaned
  echo "$OUTPUT" \
    | sed -e 's/\r//g' \
    | perl -pe 's/\e\[[\d;]*[A-Za-z]//g; s/\f//g' \
    | sed -E 's/^\[[[:space:]]*[0-9]+\.[0-9]+\][[:space:]]*cloud-init\[[0-9]+\]:[[:space:]]*//g' \
    | grep -E '^(\[startup\]|\[raito\])' \
    | awk '!seen[$0]++' || true

  # Break early if we detect the startup script completion marker
  if echo "$OUTPUT" | grep -q "\[startup\] Container exited with code"; then
    break
  fi

  # Extract next start hint from gcloud message, e.g., "Specify --start=442 ..."
  NEXT=$(printf '%s\n' "$OUTPUT" | grep -o 'Specify --start=[0-9]\+' | awk -F= '{print $2}' | tail -n1)
  if [[ -n "$NEXT" ]]; then
    START="$NEXT"
  fi

  # Break when instance is no longer RUNNING
  STATUS=$(gcloud compute instances describe "$INSTANCE_NAME" --zone="$ZONE" --format='value(status)' 2>/dev/null || true)
  if [[ "$STATUS" != "RUNNING" ]]; then
    break
  fi

  sleep 2
done

