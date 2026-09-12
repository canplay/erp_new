#!/bin/bash
# Scripts/build/backend/build-images.sh
# 构建所有后端服务镜像并推送到 Harbor
# 用法：./Scripts/build.backend/build-images.sh [TAG]
# 示例：./Scripts/build/backend/build-images.sh 20260825143000

set -euo pipefail

# =============================================================================
# 配置
# =============================================================================
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly BACKEND_DIR="/d/Workspace/MyAI/Backend"
readonly FRONTEND_DIR="/d/Workspace/MyAI/Frontend/admin"
readonly HARBOR_HOST="harbor.100.100.100.101.example.com:8003"
readonly RANCHER_HOST="https://100.100.100.101.example.com:8800"
readonly HARBOR_USER="${HARBOR_USER:-admin}"
HARBOR_PASS="${HARBOR_PASS:?Error: HARBOR_PASS environment variable required}"
readonly RANCHER_USER="${RANCHER_USER:-admin}"
RANCHER_PASS="${RANCHER_PASS:?Error: RANCHER_PASS environment variable required}"
readonly TAG="${1:-$(date +%Y%m%d%H%M%S)}"
readonly TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
readonly BUILD_TIMEOUT=3600  # 60 分钟超时
readonly PUSH_RETRIES=3
readonly PUSH_DELAY=30       # 重试间隔 (秒)

# 颜色输出
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

log() {
    local level="$1"
    shift
    local msg="$*"
    local ts=$(date +"%Y-%m-%d %H:%M:%S")
    local color=""
    case "$level" in
        INFO)   color="$GREEN" ;;
        WARN)   color="$YELLOW" ;;
        ERROR)  color="$RED" ;;
        DEBUG)  color="$BLUE" ;;
    esac
    echo -e "[${ts}] ${color}[${level}]${NC} ${msg}"
}

# 将 /d/ 路径转换为 Windows 原生路径
# 使用 cd + pwd 获取当前工作目录的 Windows 原生路径
TO_NATIVE_PATH() {
    local wsl_path="$1"
    # cd 到目录并获取其 Windows 原生路径（cd 会处理 /d/ -> D:\ 的转换）
    local native_path
    native_path=$(cd "$(dirname "$wsl_path")" && pwd)
    echo "${native_path}/$(basename "$wsl_path")"
}

build_image() {
    local service="$1"
    local image_name="localhost/myai-${service}:${TAG}"
    local output_dir="${BACKEND_DIR}"
    
    log INFO "开始构建 myai-${service}:${TAG}..."
    
    # 构建带超时控制（使用 TO_NATIVE_PATH 确保路径正确）
    local dockerfile_path
    local dockerfile_path_native
    local context_path_native
    
    dockerfile_path="${output_dir}/multi-stage.Dockerfile"
    dockerfile_path_native=$(TO_NATIVE_PATH "$dockerfile_path")
    context_path_native=$(TO_NATIVE_PATH "$output_dir")
    
    log DEBUG "Dockerfile: ${dockerfile_path_native}"
    log DEBUG "Context: ${context_path_native}"
    
    if timeout "$BUILD_TIMEOUT" podman build \
        --build-arg SERVICE_NAME="${service}" \
        -t "${image_name}" \
        -f "${dockerfile_path_native}" \
        "${context_path_native}" \
        --progress=plain 2>&1 | tail -n 30; then
        log INFO "  ✓ 构建成功：${image_name}"
        return 0
    else
        log ERROR "  ✗ 构建超时或失败：myai-${service}"
        return 1
    fi
}

push_image() {
    local image="$1"
    local target="$2"
    local retries=0
    
    log INFO "推送 ${image} -> ${target}..."
    
    while [ $retries -lt $PUSH_RETRIES ]; do
        if timeout 120 podman push \
            --tls-verify=false \
            "${image}" \
            "${target}" 2>&1; then
            log INFO "  ✓ 推送成功：${image} -> ${target}"
            return 0
        fi
        retries=$((retries + 1))
        log WARN "  推送失败 (尝试 $((retries + 1))/${PUSH_RETRIES}), 等待 ${PUSH_DELAY}s..."
        sleep "$PUSH_DELAY"
    done
    
    log ERROR "  ✗ 推送失败：${image} -> ${target} (已达最大重试次数)"
    return 1
}

verify_image() {
    local image="$1"
    log INFO "验证镜像：${image}"
    if podman images --format "{{.ID}}\t{{.Repository}}:{{.Tag}}" | grep -q "${image}"; then
        log INFO "  ✓ 镜像存在"
        return 0
    else
        log ERROR "  ✗ 镜像不存在：${image}"
        return 1
    fi
}

# =============================================================================
# 步骤 1: 环境检查
# =============================================================================
log INFO "==========================================="
log INFO "MyAI Backend Build Pipeline"
log INFO "==========================================="
log INFO "Tag: ${TAG}"
log INFO "Timestamp: ${TIMESTAMP}"
log INFO ""

# 检查 podman
if ! command -v podman &>/dev/null; then
    log ERROR "podman 未安装"
    exit 1
fi
log INFO "  ✓ podman 已安装"

# 检查 Harbor 连接
log INFO "检查 Harbor 连接..."
if echo "${HARBOR_USER}:${HARBOR_PASS}" | podman login \
    --username "${HARBOR_USER}" \
    --password-stdin \
    --tls-verify=false \
    "${HARBOR_HOST}" 2>&1 | grep -q "Login Succeeded"; then
    log INFO "  ✓ Harbor 登录成功"
else
    log WARN "  ⚠ Harbor 登录可能失败 (非致命，继续)"
fi

# 检查 Rancher 连接
log INFO "检查 Rancher 连接..."
if echo "${RANCHER_USER}:${RANCHER_PASS}" | podman login \
    --username "${RANCHER_USER}" \
    --password-stdin \
    --tls-verify=false \
    "${RANCHER_HOST}" 2>&1 | grep -q "Login Succeeded"; then
    log INFO "  ✓ Rancher 登录成功"
else
    log WARN "  ⚠ Rancher 登录可能失败 (非致命，继续)"
fi

# 检查项目根目录
log INFO "检查项目目录..."
if [ -d "/d/Workspace/MyAI" ]; then
    log INFO "  ✓ /d/Workspace/MyAI 存在"
else
    log ERROR "  ✗ 项目目录不存在：/d/Workspace/MyAI"
    exit 1
fi

# 检查 Backend 目录
if [ -d "${BACKEND_DIR}" ]; then
    log INFO "  ✓ Backend 目录存在：${BACKEND_DIR}"
else
    log ERROR "  ✗ Backend 目录不存在"
    exit 1
fi

# 检查 Cargo 和 Rust 工具链
log INFO "检查 Rust 工具链..."
if command -v cargo &>/dev/null; then
    CARGO_VERSION=$(cargo --version 2>&1)
    log INFO "  ✓ Cargo 已安装：${CARGO_VERSION}"
else
    log ERROR "  ✗ Cargo 未安装"
    exit 1
fi

if command -v rustc &>/dev/null; then
    RUSTC_VERSION=$(rustc --version 2>&1)
    log INFO "  ✓ Rustc 已安装：${RUSTC_VERSION}"
else
    log WARN "  ⚠ Rustc 未安装 (可能影响构建)"
fi

# =============================================================================
# 步骤 2: 运行测试
# =============================================================================
log INFO ""
log INFO "步骤 2: 运行测试..."
cd "${BACKEND_DIR}"
log INFO "执行 cargo test --workspace --lib..."
if cargo test --workspace --lib 2>&1 | tail -n 50; then
    log INFO "  ✓ 测试通过"
else
    log ERROR "  ✗ 测试失败"
    exit 1
fi

# =============================================================================
# 步骤 3: 清理旧镜像
# =============================================================================
log INFO ""
log INFO "步骤 3: 清理旧镜像..."
podman rmi -f "${HARBOR_HOST}/myai-builder:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-api-gateway:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-auth-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-user-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-cms-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-messaging-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-file-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-tenant-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-workflow-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-feedback-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-audit-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-api-key-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-hik-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-pay-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-clean-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-tow-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-ctp-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-xlt-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-ebike-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-lpr-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-social-ops-service:${TAG}" 2>/dev/null || true
podman rmi -f "${HARBOR_HOST}/myai-browser-service:${TAG}" 2>/dev/null || true
log INFO "  ✓ 清理完成"

# =============================================================================
# 步骤 3: 构建 Builder 镜像（统一编译所有服务）
# =============================================================================
log INFO ""
log INFO "步骤 3: 构建 Builder 镜像 (统一编译所有服务)..."
log INFO "  目标：${HARBOR_HOST}/myai/builder:${TAG}"
log INFO ""

# 构建 builder 镜像
if build_image "builder"; then
    log INFO "  ✓ Builder 镜像构建成功"
fi

# 推送到 Harbor
if push_image "${HARBOR_HOST}/myai-builder:${TAG}" "${HARBOR_HOST}/myai/builder:${TAG}"; then
    log INFO "  ✓ Builder 镜像推送成功"
else
    log WARN "  ⚠ Builder 镜像推送失败 (非致命，继续)"
fi

# =============================================================================
# 步骤 4: 构建各服务镜像
# =============================================================================
log INFO ""
log INFO "步骤 4: 构建各服务镜像..."
log INFO ""

declare -a SERVICES=(
    "api-gateway"
    "auth-service"
    "user-service"
    "cms-service"
    "messaging-service"
    "file-service"
    "tenant-service"
    "workflow-service"
    "feedback-service"
    "audit-service"
    "api-key-service"
    "hik-service"
    "pay-service"
    "clean-service"
    "tow-service"
    "ctp-service"
    "xlt-service"
    "ebike-service"
    "lpr-service"
    "social-ops-service"
    "browser-service"
)

# 并行构建（最多 4 个并发）
concurrent=0
max_concurrent=4
declare -a pending_services=("${SERVICES[@]}")
declare -a running_jobs=()

for service in "${SERVICES[@]}"; do
    # 等待一个槽位
    while [ $concurrent -ge $max_concurrent ]; do
        # 检查是否有作业完成
        for job in "${running_jobs[@]}"; do
            if [ -z "$(podman images --format "{{.Names}}" | grep -q "${job}" && echo $?)"]; then
                running_jobs=("${running_jobs[@]/$job/}")
                concurrent=$((concurrent - 1))
                break
            fi
        done
        sleep 1
    done
    
    # 启动构建
    log INFO "  构建 myai-${service}..."
    podman build \
        --build-arg SERVICE_NAME="${service}" \
        -t "localhost/myai-${service}:${TAG}" \
        -f "${BACKEND_DIR}/multi-stage.Dockerfile" \
        "${BACKEND_DIR}" \
        --progress=plain 2>&1 | tee /tmp/build-${service}.log | tail -n 30
    
    # 验证本地镜像
    if verify_image "localhost/myai-${service}:${TAG}"; then
        log INFO "  ✓ myai-${service} 构建成功"
    else
        log ERROR "  ✗ myai-${service} 构建失败"
    fi
    
    # 推送到 Harbor
    if push_image "localhost/myai-${service}:${TAG}" "${HARBOR_HOST}/myai/${service}:${TAG}"; then
        log INFO "  ✓ myai-${service} 推送成功"
    else
        log WARN "  ⚠ myai-${service} 推送失败 (非致命，继续)"
    fi
    
    running_jobs+=("${service}")
    concurrent=$((concurrent + 1))
done

# 等待所有作业完成
log INFO "等待所有构建任务完成..."
while [ ${#running_jobs[@]} -gt 0 ]; do
    sleep 10
    log INFO "  剩余任务：${#running_jobs[@]} / ${#SERVICES[@]}"
    # 检查是否全部完成
    all_done=true
    for job in "${running_jobs[@]}"; do
        if podman images --format "{{.Names}}" | grep -q "${job}"; then
            all_done=false
            break
        fi
    done
    if [ "$all_done" = true ]; then
        break
    fi
done

log INFO ""
log INFO "  ✓ 所有服务镜像构建完成"

# =============================================================================
# 步骤 5: 验证镜像
# =============================================================================
log INFO ""
log INFO "步骤 5: 验证所有镜像..."
log INFO ""

all_verified=true
for service in "${SERVICES[@]}"; do
    if verify_image "${HARBOR_HOST}/myai/${service}:${TAG}"; then
        log INFO "  ✓ ${service}"
    else
        log ERROR "  ✗ ${service}"
        all_verified=false
    fi
done

if [ "$all_verified" = true ]; then
    log INFO "  ✓ 所有镜像验证通过"
else
    log WARN "  ⚠ 部分镜像验证失败"
fi

# =============================================================================
# 步骤 6: 显示结果
# =============================================================================
log INFO ""
log INFO "==========================================="
log INFO "构建完成"
log INFO "==========================================="
log INFO "Tag: ${TAG}"
log INFO "Harbor: ${HARBOR_HOST}"
log INFO ""
log INFO "镜像列表:"
podman images --format "  {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" | grep myai | head -30
log INFO ""
log INFO "Rancher 部署:"
log INFO "  helm install myai-backend ./Scripts/helm/myai \\\\"
log INFO "    --namespace myai-prod \\\\"
log INFO "    -f ./Scripts/helm/myai/values-prod-harbor.yaml \\\\"
log INFO "    -f /tmp/harbor-pull-secret.yaml"
