#!/usr/bin/env bash
# 一键构建所有服务镜像
set -euo pipefail

REGISTRY="${REGISTRY:-harbor.example.com:8003/erp}"
TAG="${TAG:-$(date +%Y%m%d%H%M)}"
SERVICES=(api-gateway auth-service user-service tenant-service billing-service pay-service audit-service file-service cms-service messaging-service workflow-service feedback-service api-key-service social-ops-service browser-service hik-service lpr-service xlt-service ctp-service ebike-service tow-service clean-service)

echo "Building ${#SERVICES[@]} services..."
for svc in "${SERVICES[@]}"; do
    port=$(grep -oP "${svc//-/_}_HTTP_PORT=\K\d+" .env 2>/dev/null || echo "8080")
    podman build \
        -t "${REGISTRY}/${svc}:${TAG}" \
        --build-arg SERVICE_NAME="${svc}" \
        --build-arg HTTP_PORT="${port}" \
        -f Backend/Dockerfile .
    echo "✓ ${svc}:${TAG}"
done

echo "All services built successfully."
