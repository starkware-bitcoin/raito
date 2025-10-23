#!/usr/bin/env bash
set -euo pipefail

# Mirror all output to the serial console for easy log streaming
exec > >(tee -a /dev/console) 2>&1

echo "[startup] Beginning setup on $(hostname)"
echo "[startup] Using COS built-in Docker runtime"

# Expect the following variables to be defined before sourcing/concatenation:
#   REGION, IMAGE_URI, CONTAINER_COMMAND, CONTAINER_ARGS[]

REGISTRY="${REGION}-docker.pkg.dev"

# Use a writable Docker config directory on COS (root FS is read-only)
export DOCKER_CONFIG=/mnt/stateful_partition/docker-config
echo "[startup] Using Docker config dir: ${DOCKER_CONFIG}"
mkdir -p "${DOCKER_CONFIG}" || true

echo "[startup] Fetching access token from metadata server"
TOKEN=$(curl -s -H "Metadata-Flavor: Google" \
  "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token" \
  | sed -n 's/.*"access_token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

if [[ -z "${TOKEN}" ]]; then
  echo "[startup] ERROR: empty access token from metadata server" | tee /dev/console
  exit 1
fi

echo "[startup] Acquired access token"

echo "[startup] Logging in to Artifact Registry: ${REGISTRY}"
set +e
{ echo "${TOKEN}" | timeout 30s docker login -u oauth2accesstoken --password-stdin "${REGISTRY}"; } 2>&1 \
  | while IFS= read -r line; do echo "[startup] docker login: $line"; done
LOGIN_RC=${PIPESTATUS[0]}
set -e
if [[ ${LOGIN_RC} -ne 0 ]]; then
  echo "[startup] Docker login failed with code ${LOGIN_RC}" | tee /dev/console
  exit ${LOGIN_RC}
fi

echo "[startup] Pulling image: ${IMAGE_URI}"
set +e
{ timeout 15m docker pull "${IMAGE_URI}"; } 2>&1 \
  | while IFS= read -r line; do echo "[startup] docker pull: $line"; done
PULL_RC=${PIPESTATUS[0]}
set -e
if [[ ${PULL_RC} -ne 0 ]]; then
  echo "[startup] Docker pull failed with code ${PULL_RC}" | tee /dev/console
  exit ${PULL_RC}
fi

echo "[startup] Running container"
set +e
# Prefix container lines with [raito] so the log streamer can filter
if [[ -n "${CONTAINER_COMMAND}" ]]; then
  docker run --rm --name raito --network host "${IMAGE_URI}" "${CONTAINER_COMMAND}" "${CONTAINER_ARGS[@]}" \
    2>&1 | sed -e 's/^/[raito] /'
else
  docker run --rm --name raito --network host "${IMAGE_URI}" "${CONTAINER_ARGS[@]}" \
    2>&1 | sed -e 's/^/[raito] /'
fi
EXIT_CODE=${PIPESTATUS[0]}
set -e

echo "[startup] Container exited with code ${EXIT_CODE}. Shutting down VM."
/sbin/shutdown -h now || true
exit ${EXIT_CODE}


