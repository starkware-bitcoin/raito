#!/usr/bin/env bash
set -euo pipefail

# Build and push the raito-assumevalid container image to Artifact Registry.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.sh"

if [[ -z "$PROJECT_ID" ]]; then
  echo "PROJECT_ID is required. Set it in scripts/prove/config.sh or export PROJECT_ID." >&2
  exit 1
fi

IMAGE_URI="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO}/${IMAGE}:${TAG}"

# Discover repo root for proper Docker build context
if REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null); then
  :
else
  REPO_ROOT=$(realpath "$SCRIPT_DIR/../..")
fi

echo "Using build context: $REPO_ROOT"
echo "Target image: $IMAGE_URI"

echo "Enabling Artifact Registry API (idempotent)"
gcloud services enable artifactregistry.googleapis.com --project "$PROJECT_ID" || true

echo "Creating Artifact Registry repo '$REPO' in $REGION (idempotent)"
gcloud artifacts repositories create "$REPO" \
  --repository-format=docker \
  --location="$REGION" \
  --project "$PROJECT_ID" || true

echo "Configuring Docker auth for Artifact Registry"
gcloud auth configure-docker "$REGION-docker.pkg.dev" --quiet

echo "Building Docker image with multi-stage Dockerfile (using BuildKit with SSH)"
export DOCKER_BUILDKIT=1
docker build --ssh default -f "$REPO_ROOT/scripts/prove/Dockerfile" -t "$IMAGE_URI" "$REPO_ROOT"

echo "Pushing image to Artifact Registry"
docker push "$IMAGE_URI"

echo "Done. Pushed: $IMAGE_URI"

