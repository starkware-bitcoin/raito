#!/usr/bin/env bash
set -euo pipefail

# Launch a one-shot Spot VM that runs the raito-assumevalid container.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.sh"

INSTANCE_NAME=""
IMAGE_URI=""

CONTAINER_ARGS=("${CONTAINER_ARGS[@]}")

if [[ -z "$PROJECT_ID" ]]; then
  echo "PROJECT_ID is required. Set it in scripts/prove/config.sh or export PROJECT_ID." >&2
  exit 1
fi

if [[ -z "$INSTANCE_NAME" ]]; then
  INSTANCE_NAME="${INSTANCE_PREFIX}-$(date +%s)"
fi

if [[ -z "$IMAGE_URI" ]]; then
  IMAGE_URI="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO}/${IMAGE}:${TAG}"
fi

if [[ ${#CONTAINER_ARGS[@]} -eq 0 ]]; then
  CONTAINER_ARGS=(prove)
fi

echo "Launching Spot VM '$INSTANCE_NAME' in $ZONE using $MACHINE_TYPE"
echo "Container image: $IMAGE_URI"
echo "Container args: ${CONTAINER_ARGS[*]}"

gcloud compute instances create-with-container "$INSTANCE_NAME" \
  --project="$PROJECT_ID" \
  --zone="$ZONE" \
  --provisioning-model=SPOT \
  --machine-type="$MACHINE_TYPE" \
  --maintenance-policy=TERMINATE \
  --boot-disk-type="$BOOT_DISK_TYPE" --boot-disk-size="${BOOT_DISK_SIZE_GB}GB" \
  --scopes=https://www.googleapis.com/auth/cloud-platform \
  --metadata=google-logging-enabled=true \
  --labels="$INSTANCE_LABELS" \
  --container-image="$IMAGE_URI" \
  --container-restart-policy=never \
  --container-command="$CONTAINER_COMMAND" \
  $(printf ' --container-arg=%q' "${CONTAINER_ARGS[@]}")

echo "Instance created: $INSTANCE_NAME"
echo "$INSTANCE_NAME" > "$SCRIPT_DIR/.last_instance"
echo "Tip: stream logs -> scripts/prove/logs.sh"

