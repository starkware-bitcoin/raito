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

# MIGRATION: The container startup agent is deprecated. Use a standard VM with a startup script.
# Build a startup script that installs Docker, authenticates to Artifact Registry using the
# instance service account, pulls the image, runs it once, and then powers off the VM.

# Render CONTAINER_ARGS as a Bash array literal to preserve argument boundaries
CONTAINER_ARGS_BASH_LITERAL="("
for _arg in "${CONTAINER_ARGS[@]}"; do
  printf -v _q '%q' "$_arg"
  CONTAINER_ARGS_BASH_LITERAL+=" $_q"
done
CONTAINER_ARGS_BASH_LITERAL+=" )"

STARTUP_SCRIPT_FILE=$(mktemp)
{
  echo '#!/usr/bin/env bash'
  echo 'set -euo pipefail'
  printf 'REGION=%q\n' "$REGION"
  printf 'IMAGE_URI=%q\n' "$IMAGE_URI"
  printf 'CONTAINER_COMMAND=%q\n' "$CONTAINER_COMMAND"
  printf 'declare -a CONTAINER_ARGS=%s\n' "$CONTAINER_ARGS_BASH_LITERAL"
  echo ''
  cat "$SCRIPT_DIR/startup_cos.sh"
} > "$STARTUP_SCRIPT_FILE"

gcloud compute instances create "$INSTANCE_NAME" \
  --project="$PROJECT_ID" \
  --zone="$ZONE" \
  --provisioning-model=SPOT \
  --machine-type="$MACHINE_TYPE" \
  --maintenance-policy=TERMINATE \
  --boot-disk-type="$BOOT_DISK_TYPE" --boot-disk-size="${BOOT_DISK_SIZE_GB}GB" \
  --scopes=https://www.googleapis.com/auth/cloud-platform \
  --metadata=google-logging-enabled=true \
  --labels="$INSTANCE_LABELS" \
  --image-family=cos-109-lts \
  --image-project=cos-cloud \
  --metadata-from-file user-data="$STARTUP_SCRIPT_FILE"

echo "Instance created: $INSTANCE_NAME"
echo "$INSTANCE_NAME" > "$SCRIPT_DIR/.last_instance"
echo "Tip: stream logs -> scripts/prove/logs.sh"

