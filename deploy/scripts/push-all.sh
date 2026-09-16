#!/usr/bin/env bash
# 一键推送所有服务镜像到 Harbor
set -euo pipefail

REGISTRY="${REGISTRY:-harbor.example.com:8003/erp}"
TAG="${TAG:-$(date +%Y%m%d%H%M)}"
SERVICES=(api-gateway auth-service user-service tenant-service billing-service pay-service audit-service file-service cms-service messaging-service workflow-service feedback-service api-key-service social-ops-service browser-service hik-service lpr-service xlt-service ctp-service ebike-service tow-service clean-service)

echo "Pushing ${#SERVICES[@]} services to ${REGISTRY}..."
for svc in "${SERVICES[@]}"; do
    podman push --tls-verify=false "${REGISTRY}/${svc}:${TAG}"
    echo "✓ ${svc}:${TAG}"
done

echo "All services pushed successfully."
