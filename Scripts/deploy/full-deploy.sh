#!/bin/bash
# MyAI 完整部署流程脚本
# 执行流程：编译后端 → 编译前端 → 构建镜像 → 推送 Harbor → Rancher 部署
#
# 用法：
#   ./Scripts/deploy/full-deploy.sh [TAG]
#   ./Scripts/deploy/full-deploy.sh 20260825143000
#
# 依赖：
#   - podman
#   - cargo (Rust)
#   - node / npm (前端)
#   - helm
#
# 环境变量：
#   HARBOR_USER=admin
#   HARBOR_PASS=ChangeMeHarbor123!
#   RANCHER_USER=admin
#   RANCHER_PASS=canplay745144
#   HARBOR_HOST=harbor.100.100.100.101.example.com:8003
#   RANCHER_HOST=https://100.100.100.101.example.com:8800

set -euo pipefail

# =============================================================================
# 配置
# =============================================================================
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly MYAI_ROOT="/d/Workspace/MyAI"
readonly BACKEND_DIR="/d/Workspace/MyAI/Backend"
readonly FRONTEND_DIR="/d/Workspace/MyAI/Frontend/admin"
readonly HARBOR_HOST="harbor.100.100.100.101.example.com:8003"
readonly RANCHER_HOST="https://100.100.100.101.example.com:8800"
readonly HARBOR_USER="admin"
readonly HARBOR_PASS="ChangeMeHarbor123!"
readonly RANCHER_USER="admin"
readonly RANCHER_PASS="canplay745144"
readonly TAG="${1:-$(date +%Y%m%d%H%M%S)}"
readonly TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

# 颜色
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

# =============================================================================
# 步骤 1: 后端编译（cargo build --release --workspace）
# =============================================================================
deploy_backend() {
    log INFO "=========================================="
    log INFO "步骤 1: 后端编译"
    log INFO "=========================================="
    log INFO "  目标：${BACKEND_DIR}/target/release/*"
    log INFO "  构建模式：cargo build --release --workspace"
    log INFO ""
    
    log INFO "进入 Backend 目录..."
    cd "${BACKEND_DIR}"
    
    log INFO "执行 cargo build --release --workspace..."
    # 在后台运行，避免阻塞
    cargo build --release --workspace --manifest-path Cargo.toml &
    local cargo_pid=$!
    log INFO "  Cargo 进程 PID: ${cargo_pid}"
    
    # 等待构建完成（超时 600 秒）
    local timeout=600
    local elapsed=0
    while [ $elapsed -lt $timeout ]; do
        if kill -0 $cargo_pid 2>/dev/null; then
            log INFO "  构建中... (${elapsed}s / ${timeout}s)"
            sleep 10
            elapsed=$((elapsed + 10))
        else
            log INFO "  ✓ Cargo 构建完成"
            break
        fi
    done
    
    # 检查构建结果
    if [ -d "${BACKEND_DIR}/target/release" ]; then
        log INFO "  ✓ 编译成功"
        log INFO ""
        log INFO "  编译产物:"
        ls -lh "${BACKEND_DIR}/target/release" | grep -E "\.(rlib|so|dylib|dll)$" | head -20
        log INFO ""
        return 0
    else
        log ERROR "  ✗ 编译失败"
        return 1
    fi
}

# =============================================================================
# 步骤 2: 前端编译（vue-tsc + quasar build）
# =============================================================================
deploy_frontend() {
    log INFO "=========================================="
    log INFO "步骤 2: 前端编译"
    log INFO "=========================================="
    log INFO "  目标：${FRONTEND_DIR}/dist"
    log INFO "  命令：vue-tsc --noEmit && quasar build"
    log INFO ""
    
    cd "${FRONTEND_DIR}"
    
    # 1. 类型检查
    log INFO "执行 vue-tsc --noEmit..."
    if npx vue-tsc --noEmit 2>&1 | tail -20; then
        log INFO "  ✓ 类型检查通过"
    else
        log ERROR "  ✗ 类型检查失败"
        return 1
    fi
    
    # 2. ESLint 检查
    log INFO "执行 eslint..."
    if npx eslint src/ --max-warnings 0 2>&1 | tail -20; then
        log INFO "  ✓ ESLint 通过"
    else
        log WARN "  ⚠ ESLint 有警告，继续"
    fi
    
    # 3. 构建
    log INFO "执行 quasar build..."
    if npx quasar build --production 2>&1 | tail -20; then
        log INFO "  ✓ 前端构建成功"
        log INFO ""
        log INFO "  产物目录：${FRONTEND_DIR}/dist"
        ls -lh "${FRONTEND_DIR}/dist" | head -10
        return 0
    else
        log ERROR "  ✗ 前端构建失败"
        return 1
    fi
}

# =============================================================================
# 步骤 3: 构建镜像（podman build）
# =============================================================================
build_image() {
    local service="$1"
    local image_name="localhost/myai-${service}:${TAG}"
    local output_dir="${BACKEND_DIR}"
    
    log INFO "开始构建 myai-${service}:${TAG}..."
    
    local dockerfile_path="${output_dir}/multi-stage.Dockerfile"
    local context_path="${output_dir}"
    
    if timeout 3600 podman build \
        --build-arg SERVICE_NAME="${service}" \
        -t "${image_name}" \
        -f "${dockerfile_path}" \
        "${context_path}" \
        --progress=plain 2>&1 | tail -n 30; then
        log INFO "  ✓ 构建成功：${image_name}"
        return 0
    else
        log ERROR "  ✗ 构建失败：myai-${service}"
        return 1
    fi
}

build_all_images() {
    log INFO "=========================================="
    log INFO "步骤 3: 构建镜像"
    log INFO "=========================================="
    log INFO "  目标：${HARBOR_HOST}/myai/*:${TAG}"
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
    
    local success_count=0
    local fail_count=0
    
    for service in "${SERVICES[@]}"; do
        if build_image "${service}"; then
            ((success_count++))
        else
            ((fail_count++))
        fi
    done
    
    log INFO ""
    log INFO "  ✓ 构建完成：${success_count} 成功 / ${#SERVICES[@]} 总计"
    
    # 显示镜像列表
    log INFO ""
    log INFO "  本地镜像列表:"
    podman images --format "  {{.Repository}}:{{.Tag}}\t{{.Size}}" | grep myai | head -30
    
    return $fail_count
}

# =============================================================================
# 步骤 4: 推送到 Harbor
# =============================================================================
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

push_all_images() {
    log INFO "=========================================="
    log INFO "步骤 4: 推送到 Harbor"
    log INFO "=========================================="
    log INFO "  目标：${HARBOR_HOST}/myai/*:${TAG}"
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
    
    local success_count=0
    local fail_count=0
    
    for service in "${SERVICES[@]}"; do
        local local_image="localhost/myai-${service}:${TAG}"
        local target="${HARBOR_HOST}/myai/${service}:${TAG}"
        
        if push_image "${local_image}" "${target}"; then
            ((success_count++))
        else
            ((fail_count++))
        fi
    done
    
    log INFO ""
    log INFO "  ✓ 推送完成：${success_count} 成功 / ${#SERVICES[@]} 总计"
    
    return $fail_count
}

# =============================================================================
# 步骤 5: Rancher 部署
# =============================================================================
deploy_to_rancher() {
    local tag="$1"
    log INFO "=========================================="
    log INFO "步骤 5: Rancher 部署"
    log INFO "=========================================="
    log INFO "  目标：${RANCHER_HOST}"
    log INFO "  命名空间：myai-prod"
    log INFO "  Release: myai-backend"
    log INFO "  Tag: ${tag}"
    log INFO ""
    
    # 1. 登录 Rancher
    log INFO "登录 Rancher..."
    local rancher_login_output
    rancher_login_output=$(echo "${RANCHER_USER}:${RANCHER_PASS}" | podman login \
        --username "${RANCHER_USER}" \
        --password-stdin \
        --tls-verify=false \
        "${RANCHER_HOST}" 2>&1)
    
    if echo "${rancher_login_output}" | grep -q "Login Succeeded"; then
        log INFO "  ✓ Rancher 登录成功"
    else
        log ERROR "  ✗ Rancher 登录失败"
        echo "${rancher_login_output}"
        return 1
    fi
    
    # 2. 创建 Harbor pull secret（如果不存在）
    log INFO "检查 Harbor pull secret..."
    local secret_yaml="${SCRIPT_DIR}/deploy/harbor-pull-secret.yaml"
    
    # base64 编码 .dockerconfigjson
    local base64_encoded
    base64_encoded=$(base64 -w0 <<< '{
  "auths": {
    "'${HARBOR_HOST}'": {
      "username": "'${HARBOR_USER}'",
      "password": "'${HARBOR_PASS}'"
    }
  }
}')
    
    cat > "${secret_yaml}" << EOF
apiVersion: v1
kind: Secret
metadata:
  name: harbor-pull-secret
  namespace: myai-prod
type: kubernetes.io/dockerconfigjson
data:
  .dockerconfigjson: ${base64_encoded}
EOF
    
    log INFO "  Harbor pull secret 已生成"
    
    # 3. 部署到 Rancher
    log INFO "部署 myai-backend 到 Rancher..."
    
    # 使用 helm 部署
    local helm_deploy_output
    helm_deploy_output=$(helm upgrade --install myai-backend \
        "${SCRIPT_DIR}/helm/myai" \
        --namespace myai-prod \
        -f "${SCRIPT_DIR}/helm/myai/values-prod-harbor.yaml" \
        --set image.tag="${tag}" \
        --set image.pullSecrets.name=harbor-pull-secret \
        --set image.pullSecrets.enabled=true \
        --wait 2>&1 | tail -30)
    
    if echo "${helm_deploy_output}" | grep -q "release \"myai-backend\" has been updated"; then
        log INFO "  ✓ Rancher 部署成功"
        log INFO ""
        log INFO "  部署结果:"
        echo "${helm_deploy_output}"
        return 0
    else
        log ERROR "  ✗ Rancher 部署失败"
        echo "${helm_deploy_output}"
        return 1
    fi
}

# =============================================================================
# 主流程
# =============================================================================
main() {
    log INFO "=========================================="
    log INFO "MyAI 完整部署流程"
    log INFO "=========================================="
    log INFO "Tag: ${TAG}"
    log INFO "Timestamp: ${TIMESTAMP}"
    log INFO ""
    
    # 步骤 1: 后端编译
    log INFO "=========================================="
    log INFO "步骤 1: 后端编译 (cargo build --release)"
    log INFO "=========================================="
    deploy_backend
    STEP1_RESULT=$?
    
    if [ $STEP1_RESULT -ne 0 ]; then
        log ERROR "步骤 1 失败，终止流程"
        exit 1
    fi
    
    # 步骤 2: 前端编译
    log INFO ""
    log INFO "=========================================="
    log INFO "步骤 2: 前端编译 (vue-tsc + quasar build)"
    log INFO "=========================================="
    deploy_frontend
    STEP2_RESULT=$?
    
    if [ $STEP2_RESULT -ne 0 ]; then
        log ERROR "步骤 2 失败，终止流程"
        exit 1
    fi
    
    # 步骤 3: 构建镜像
    log INFO ""
    log INFO "=========================================="
    log INFO "步骤 3: 构建镜像 (podman build)"
    log INFO "=========================================="
    build_all_images
    STEP3_RESULT=$?
    
    # 步骤 4: 推送到 Harbor
    log INFO ""
    log INFO "=========================================="
    log INFO "步骤 4: 推送到 Harbor"
    log INFO "=========================================="
    push_all_images
    STEP4_RESULT=$?
    
    # 步骤 5: Rancher 部署
    log INFO ""
    log INFO "=========================================="
    log INFO "步骤 5: Rancher 部署 (helm)"
    log INFO "=========================================="
    deploy_to_rancher "${TAG}"
    STEP5_RESULT=$?
    
    log INFO ""
    log INFO "=========================================="
    log INFO "部署完成"
    log INFO "=========================================="
    log INFO "Tag: ${TAG}"
    log INFO ""
    log INFO "镜像列表:"
    podman images --format "  {{.Repository}}:{{.Tag}}\t{{.Size}}" | grep myai | head -30
    log INFO ""
    log INFO "Rancher 部署:"
    log INFO "  helm list -n myai-prod | grep myai-backend"
    
    exit $STEP5_RESULT
}

main "$@"
