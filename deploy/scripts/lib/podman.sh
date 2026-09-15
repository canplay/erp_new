#!/bin/bash
# =============================================================================
# Podman 操作函数库
# =============================================================================
# 提供 Podman 检查、镜像构建等操作

# ---------- 检查 podman 可用 ----------
podman_check() {
    if ! command -v podman >/dev/null 2>&1; then
        echo "ERROR: podman 不可用，请先安装并启动 podman machine" >&2
        return 1
    fi
    if ! podman info >/dev/null 2>&1; then
        echo "INFO: Starting podman machine..."
        podman machine start
    fi
    return 0
}

# ---------- 构建镜像 ----------
podman_build_image() {
    local image_name="$1"
    local tag="$2"
    local dockerfile="$3"
    shift 3
    local build_args="$@"

    echo "  构建 myai-${image_name}:${tag}..."

    # 使用标准格式 tag (YYYYMMDDHHMMSS)
    if podman build ${build_args} \
        --build-arg BUILD_TAG="$tag" \
        -t "myai-${image_name}:${tag}" \
        -f "$dockerfile" .; then
        echo " [OK] myai-${image_name}:${tag}"
        return 0
    else
        echo " [FAIL] myai-${image_name}:${tag}"
        return 1
    fi
}
