#!/bin/bash
# Scripts/build/backend/wait-for-build.sh
# 轮询检查后端镜像构建状态
# 用法：./Scripts/build/backend/wait-for-build.sh 20260824180000

set -e

TAG="$1"
if [ -z "$TAG" ]; then
    TAG=$(date +%Y%m%d%H%M%S)
fi

# 定义 START_TIME
START_TIME=$(date +%s)

echo "Waiting for build to complete (tag: ${TAG})..."
echo "Polling every 10 seconds..."

while true; do
    status=$(podman images --format "{{.Repository}}:{{.Tag}}\t{{.ID}}\t{{.SizeByType}}" | grep "myai-${TAG}" || true)
    if [ -n "$status" ]; then
        echo "✓ Build complete: $status"
        break
    fi
    echo "  Building... (elapsed: $(($(date +%s) - START_TIME))s)"
    sleep 10
done

echo "=== Build Complete ==="
podman images --format "{{.Repository}}:{{.Tag}}\t{{.ID}}\t{{.SizeByType}}" | grep "myai-${TAG}"
