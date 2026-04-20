#!/usr/bin/env bash
set -euo pipefail

# E2E test runner for umami-cli
# Starts a local Umami instance via Docker and runs integration tests against it.

COMPOSE_FILE="docker-compose.test.yml"
PROJECT_NAME="umami-cli-e2e"
UMAMI_URL="http://localhost:3099"
MAX_WAIT=120

cd "$(dirname "$0")/.."

cleanup() {
    echo "Shutting down test containers..."
    docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" down -v 2>/dev/null || true
}

# Clean up on exit
trap cleanup EXIT

echo "=== Starting Umami test instance ==="
docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" up -d

echo "Waiting for Umami to be ready..."
elapsed=0
until curl -sf "$UMAMI_URL/api/heartbeat" > /dev/null 2>&1; do
    if [ "$elapsed" -ge "$MAX_WAIT" ]; then
        echo "ERROR: Umami did not start within ${MAX_WAIT}s"
        docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" logs umami
        exit 1
    fi
    sleep 2
    elapsed=$((elapsed + 2))
    printf "\r  Waiting... %ds / %ds" "$elapsed" "$MAX_WAIT"
done
echo ""
echo "Umami is ready at $UMAMI_URL"

echo ""
echo "=== Running E2E tests ==="
cargo test --test e2e -- --ignored --test-threads=1 "$@"

echo ""
echo "=== E2E tests complete ==="
