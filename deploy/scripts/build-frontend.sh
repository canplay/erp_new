#!/bin/bash
# Scripts/build/frontend/build-images.sh
# 构建前端镜像并推送到 Harbor
# 用法：./Scripts/build/frontend/build-images.sh [TAG]
# 示例：./Scripts/build/frontend/build-images.sh 20260825143000

set -euo pipefail

# =============================================================================
# 配置
# =============================================================================
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly FRONTEND_DIR="/d/Workspace/MyAI/Frontend/admin"
readonly HARBOR_HOST="harbor.100.100.100.101.example.com:8003"
readonly TAG="${1:-$(date +%Y%m%d%H%M%S)}"
readonly TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

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
to_native_path() {
    local path="$1"
    python3 "${SCRIPT_DIR}/../tools/convert_path.py" "$path"
}

build_image() {
    local service="$1"
    local image_name="localhost/myai-${service}:${TAG}"
    local output_dir="${FRONTEND_DIR}"
    
    log INFO "开始构建 myai-${service}:${TAG}..."
    
    local dockerfile_path="${output_dir}/Dockerfile"
    local context_path_native=$(to_native_path "$output_dir")
    
    log DEBUG "Dockerfile: ${dockerfile_path}"
    log DEBUG "Context: ${context_path_native}"
    
    if timeout 3600 podman build \
        -t "${image_name}" \
        -f "${dockerfile_path}" \
        "${context_path_native}" \
        --progress=plain 2>&1 | tail -n 30; then
        log INFO "  ✓ 构建成功：${image_name}"
        return 0
    else
        log ERROR "  ✗ 构建失败：myai-${service}"
        return 1
    fi
}

push_image() {
    local image="$1"
    local target="$2"
    local retries=0
    
    log INFO "推送 ${image} -> ${target}..."
    
    while [ $retries -lt 3 ]; do
        if timeout 120 podman push \
            --tls-verify=false \
            "${image}" \
            "${target}" 2>&1; then
            log INFO "  ✓ 推送成功：${image} -> ${target}"
            return 0
        fi
        retries=$((retries + 1))
        log WARN "  推送失败 (尝试 $((retries + 1))/3), 等待 30s..."
        sleep 30
    done
    
    log ERROR "  ✗ 推送失败：${image} -> ${target}"
    return 1
}

# =============================================================================
# 步骤 1: 环境检查
# =============================================================================
log INFO "=========================================="
log INFO "MyAI Frontend Build Pipeline"
log INFO "=========================================="
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
HARBOR_USER="${HARBOR_USER:-admin}"
HARBOR_PASS="${HARBOR_PASS:?Error: HARBOR_PASS environment variable required}"
if echo "${HARBOR_USER}:${HARBOR_PASS}" | podman login \
    --username "${HARBOR_USER}" \
    --password-stdin \
    --tls-verify=false \
    "${HARBOR_HOST}" 2>&1 | grep -q "Login Succeeded"; then
    log INFO "  ✓ Harbor 登录成功"
else
    log WARN "  ⚠ Harbor 登录可能失败 (非致命，继续)"
fi

# 检查项目根目录
log INFO "检查项目目录..."
if [ -d "/d/Workspace/MyAI" ]; then
    log INFO "  ✓ /d/Workspace/MyAI 存在"
else
    log ERROR "  ✗ 项目目录不存在：/d/Workspace/MyAI"
    exit 1
fi

# 检查 Frontend 目录
if [ -d "${FRONTEND_DIR}" ]; then
    log INFO "  ✓ Frontend 目录存在：${FRONTEND_DIR}"
else
    log ERROR "  ✗ Frontend 目录不存在"
    exit 1
fi

# 检查 Node 和 npm
log INFO "检查 Node.js 环境..."
if command -v node &>/dev/null; then
    NODE_VERSION=$(node --version 2>&1)
    log INFO "  ✓ Node.js 已安装：${NODE_VERSION}"
else
    log ERROR "  ✗ Node.js 未安装"
    exit 1
fi

if command -v npm &>/dev/null; then
    NPM_VERSION=$(npm --version 2>&1)
    log INFO "  ✓ npm 已安装：${NPM_VERSION}"
else
    log ERROR "  ✗ npm 未安装"
    exit 1
fi

# =============================================================================
# 步骤 2: 运行测试
# =============================================================================
log INFO ""
log INFO "步骤 2: 运行测试"
log INFO "=========================================="
log INFO "执行 npx vitest run..."

cd "${FRONTEND_DIR}"

if npx vitest run 2>&1 | tail -30; then
    log INFO "  ✓ 测试通过"
else
    log ERROR "  ✗ 测试失败"
    exit 1
fi

# =============================================================================
# 步骤 3: 修复前端错误
# =============================================================================
log INFO ""
log INFO "步骤 3: 修复前端错误"
log INFO "=========================================="
log INFO "运行修复脚本..."

cd "${FRONTEND_DIR}"

# 修复 AnnouncementList.vue
echo "修复 AnnouncementList.vue..."
python3 /d/Workspace/MyAI/Frontend/admin/fix-announcement-list.py
if [ $? -ne 0 ]; then
    log WARN "  ⚠ AnnouncementList.vue 修复失败，继续..."
fi

# 修复 LoginPage 组件
echo "修复 LoginPage 组件..."
python3 /d/Workspace/MyAI/Frontend/admin/fix-all.py
if [ $? -ne 0 ]; then
    log WARN "  ⚠ LoginPage 修复失败，继续..."
fi

# 检查是否还有 TypeScript 错误
log INFO "检查 TypeScript 错误..."
TS_ERROR_COUNT=$(npx vue-tsc --noEmit 2>&1 | grep -c "error TS" || echo "0")
if [ "$TS_ERROR_COUNT" -gt 0 ]; then
    log WARN "  仍有 ${TS_ERROR_COUNT} 个 TypeScript 错误"
    npx vue-tsc --noEmit 2>&1 | grep -E "error TS[0-9]+" | sort
else
    log INFO "  ✓ TypeScript 检查通过"
fi

# =============================================================================
# 步骤 3: 前端编译
# =============================================================================
log INFO ""
log INFO "步骤 3: 前端编译"
log INFO "=========================================="
log INFO "执行 npx quasar build --production..."

cd "${FRONTEND_DIR}"

if npx quasar build --production 2>&1 | tail -30; then
    log INFO "  ✓ 前端构建成功"
    log INFO ""
    log INFO "  产物目录：${FRONTEND_DIR}/dist"
    ls -lh "${FRONTEND_DIR}/dist" | head -10
    log INFO ""
    log INFO "  镜像构建上下文：${FRONTEND_DIR}"
else
    log ERROR "  ✗ 前端构建失败"
    exit 1
fi

# =============================================================================
# 步骤 4: 构建镜像
# =============================================================================
log INFO ""
log INFO "步骤 4: 构建镜像"
log INFO "=========================================="
log INFO "目标：${HARBOR_HOST}/myai/admin:${TAG}"
log INFO ""

build_image "admin"
BUILD_RESULT=$?

if [ $BUILD_RESULT -ne 0 ]; then
    log ERROR "  ✗ 镜像构建失败"
    exit 1
fi

# =============================================================================
# 步骤 5: 推送到 Harbor
# =============================================================================
log INFO ""
log INFO "步骤 5: 推送到 Harbor"
log INFO "=========================================="
log INFO "目标：${HARBOR_HOST}/myai/admin:${TAG}"
log INFO ""

push_image "localhost/myai-admin:${TAG}" "${HARBOR_HOST}/myai/admin:${TAG}"
PUSH_RESULT=$?

if [ $PUSH_RESULT -ne 0 ]; then
    log ERROR "  ✗ 镜像推送失败"
    exit 1
fi

# =============================================================================
# 步骤 6: 显示结果
# =============================================================================
log INFO ""
log INFO "=========================================="
log INFO "构建完成"
log INFO "=========================================="
log INFO "Tag: ${TAG}"
log INFO "Harbor: ${HARBOR_HOST}"
log INFO ""
log INFO "镜像列表:"
podman images --format "  {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" | grep myai | head -10
log INFO ""
log INFO "Rancher 部署:"
log INFO "  helm upgrade --install myai-frontend ./Scripts/helm/myai \\"
log INFO "    --namespace myai-prod \\"
log INFO "    -f ./Scripts/helm/myai/values-prod-harbor.yaml \\"
log INFO "    -f /tmp/harbor-pull-secret.yaml \\"
log INFO "    --set image.tag=${TAG} \\"
log INFO "    --set image.pullSecrets.name=harbor-pull-secret \\"
log INFO "    --set image.pullSecrets.enabled=true"

exit 0
