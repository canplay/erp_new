#!/bin/bash
# =============================================================================
# MyAI 全栈构建-部署流水线
# =============================================================================
# 用法: ./Scripts/build-and-deploy.sh [OPTIONS]
#   -t <TAG>          指定镜像TAG (默认: 自动生成 YYYYMMDDHHMM)
#   -n <NAMESPACE>    K8s 命名空间 (默认: myai-prod)
#   --skip-build      跳过构建，仅部署 (使用Harbor已有镜像)
#   --skip-deploy     仅构建，不部署
#   --dry-run         仅显示将要执行的操作，不实际执行
#   --auto-tags       每个服务使用Harbor最新tag
#   --rollback        部署失败时自动回滚
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${0}")" && pwd)"
source "$SCRIPT_DIR/lib/all.sh"
load_env

# ---------- 参数解析 ----------
TAG=""
NAMESPACE="${NAMESPACE:-myai-prod}"
SKIP_BUILD=false
SKIP_DEPLOY=false
DRY_RUN=false
AUTO_TAGS=false
ROLLBACK=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -t|--tag) TAG="$2"; shift 2 ;;
        -n|--namespace) NAMESPACE="$2"; shift 2 ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --skip-deploy) SKIP_DEPLOY=true; shift ;;
        --dry-run) DRY_RUN=true; shift ;;
        --auto-tags) AUTO_TAGS=true; shift ;;
        --rollback) ROLLBACK=true; shift ;;
        -h|--help)
            echo "用法: $0 [-t <TAG>] [-n <NAMESPACE>] [--skip-build] [--skip-deploy] [--dry-run] [--auto-tags] [--rollback]"
            exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 1 ;;
    esac
done

# ---------- 自动生成 TAG ----------
if [[ -z "$TAG" ]]; then
    TAG="$(date +%Y%m%d%H%M)"
    log_info "使用自动生成 TAG: $TAG"
fi

log_info "=== MyAI 全栈构建-部署流水线 ==="
log_info "TAG: $TAG | NAMESPACE: $NAMESPACE | 模式: $(if $SKIP_BUILD; then echo '仅部署'; elif $SKIP_DEPLOY; then echo '仅构建'; else echo '全量'; fi)"

# ---------- 工具链检查 ----------
check_toolchain() {
    log_step "检查工具链..."

    local missing=()
    command -v podman >/dev/null 2>&1 || missing+=("podman")
    command -v helm >/dev/null 2>&1 || missing+=("helm")
    command -v kubectl >/dev/null 2>&1 || missing+=("kubectl")
    command -v cargo >/dev/null 2>&1 || missing+=("cargo")
    command -v pnpm >/dev/null 2>&1 || missing+=("pnpm")

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "缺少工具: ${missing[*]}"
        log_error "请安装后重试"
        exit 1
    fi

    log_success "工具链检查通过"
}

# ---------- 构建前端 ----------
build_frontend() {
    log_step "构建前端镜像..."

    local admin_dir="$PROJECT_ROOT/Frontend/admin"
    if [[ ! -d "$admin_dir" ]]; then
        log_error "前端 admin 目录不存在: $admin_dir"
        return 1
    fi

    # 构建上下文切到 admin 目录，确保 COPY nginx.conf 和 COPY dist/spa/ 路径正确
    cd "$admin_dir"
    log_info "运行 pnpm build..."
    pnpm --filter myai-admin build

    # 构建镜像
    log_info "构建前端镜像 myai-admin:$TAG..."
    podman build \
        --build-arg BUILD_SERVICE=admin \
        --build-arg BUILD_TAG="$TAG" \
        -t "myai-admin:$TAG" \
        -f Dockerfile.local-admin .

    log_success "前端镜像构建成功: myai-admin:$TAG"
}

# ---------- 构建后端 ----------
build_backend() {
    log_step "构建后端镜像..."

    local backend_dir="$PROJECT_ROOT/Backend"
    if [[ ! -d "$backend_dir" ]]; then
        log_error "后端目录不存在: $backend_dir"
        return 1
    fi

    # 编译所有服务
    cd "$backend_dir"
    log_info "运行 cargo build --release..."
    export SQLX_OFFLINE=true
    export DATABASE_URL="postgresql://100.100.100.100:5432/myai_db"
    cargo build --release --all-targets

    # 为每个服务构建镜像
    for svc in "${!SERVICES[@]}"; do
        local port="${SERVICES[$svc]}"
        log_info "构建后端镜像 myai-$svc:$TAG..."
        podman build \
            --build-arg SERVICE_NAME="$svc" \
            --build-arg HTTP_PORT="$port" \
            --build-arg BUILD_TAG="$TAG" \
            -t "myai-$svc:$TAG" \
            -f compile.Dockerfile .
    done

    log_success "所有后端镜像构建成功"
}

# ---------- 推送所有镜像到 Harbor ----------
push_all_images() {
    log_step "推送镜像到 Harbor..."

    local harbor_url="${HARBOR_URL:-harbor.100.100.100.101.example.com:8003}"
    local harbor_project="${HARBOR_PROJECT:-erp}"
    local harbor_user="${HARBOR_USER:-admin}"

    # 登录 Harbor
    log_info "登录 Harbor..."
    local harbor_pass
    harbor_pass="$(harbor_get_password "$harbor_url")"
    echo "$harbor_pass" | podman login "$harbor_url" -u "$harbor_user" --password-stdin

    # 推送前端镜像
    log_info "推送前端镜像..."
    podman push "myai-admin:$TAG" "$harbor_url/$harbor_project/myai-admin:$TAG"

    # 推送所有后端镜像
    for svc in "${!SERVICES[@]}"; do
        log_info "推送 myai-$svc:$TAG..."
        podman push "myai-$svc:$TAG" "$harbor_url/$harbor_project/myai-$svc:$TAG"
    done

    log_success "所有镜像已推送到 Harbor"
}

# ---------- 部署到 Rancher ----------
deploy_all() {
    log_step "部署到 Rancher..."

    local rancher_url="${RANCHER_URL:-https://100.100.100.101.example.com:8800}"
    local rancher_user="${RANCHER_USER:-admin}"
    local rancher_pass
    rancher_pass="$(get_rancher_password)"

    # 登录 Rancher
    log_info "登录 Rancher..."
    rancher_login "$rancher_url" "$rancher_user" "$rancher_pass"

    # Helm 部署
    log_info "执行 Helm 部署..."
    local helm_chart="$PROJECT_ROOT/Scripts/helm/myai"
    local values_file="$helm_chart/values-prod-harbor.yaml"

    local HELM_CMD=(
        helm upgrade --install myai "$helm_chart"
        -n "$NAMESPACE"
        -f "$values_file"
        --set "image.tag=$TAG"
        --set 'global.imagePullSecrets[0].name=harbor-secret'
        --timeout 15m
        --rollback-on-failure
        --wait
    )

    if [[ "$AUTO_TAGS" == true ]]; then
        for svc_key in "${!SVC_IMAGE_MAP[@]}"; do
            local img="${SVC_IMAGE_MAP[$svc_key]}"
            HELM_CMD+=(--set "services.$svc_key.tag=$TAG")
        done
    fi

    log_info "执行: ${HELM_CMD[*]}"
    if [[ "$DRY_RUN" == true ]]; then
        log_info "DRY RUN: ${HELM_CMD[*]}"
    else
        "${HELM_CMD[@]}"
    fi

    log_success "Helm 部署成功"

    # 等待 Pod 就绪
    log_info "等待 Pod 就绪..."
    if wait_pods_ready "$NAMESPACE" 300 10; then
        log_success "所有 Pod 已就绪"
    else
        log_warn "部分 Pod 未就绪，请手动检查"
    fi
}

# ---------- 验证部署 ----------
verify_deployment() {
    log_step "验证部署..."

    local errors=0

    # 检查 Pod 状态
    local pod_status
    pod_status="$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/instance=myai --no-headers 2>/dev/null || true)"

    if [[ -z "$pod_status" ]]; then
        log_error "未找到 Pod"
        return 1
    fi

    # 检查每个 Pod 状态
    while read -r pod_name status rest; do
        if [[ "$status" != "Running" ]]; then
            log_error "Pod $pod_name 状态: $status"
            ((errors++))
        fi
    done <<< "$pod_status"

    # 检查 API 网关健康
    log_info "检查 API 网关健康..."
    local health_response
    health_response="$(curl -sk --max-time 10 "https://api.100.100.100.100.example.com/api/health" 2>/dev/null || true)"

    if [[ -z "$health_response" ]]; then
        log_warn "API 网关健康检查返回空"
    else
        log_success "API 网关健康检查通过"
    fi

    if [[ $errors -gt 0 ]]; then
        log_error "检测到 $errors 个异常 Pod"
        return 1
    fi

    log_success "部署验证通过"
}

# ---------- 回滚 ----------
rollback() {
    local namespace="$1"
    local release_name="myai"

    log_step "回滚 Helm 发布..."

    # 获取当前版本
    local current_version
    current_version="$(helm history "$release_name" -n "$namespace" -o json 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d[-1].get("revision","?"))' 2>/dev/null || echo "?")"

    # 回滚到上一个版本
    helm rollback "$release_name" $((current_version - 1 - 1)) -n "$namespace" --wait --timeout 10m

    log_success "已回滚到上一个版本"
}

# ---------- 主流程 ----------
main() {
    # 1. 检查工具链
    check_toolchain

    # 2. 构建前端
    if [[ "$SKIP_BUILD" != true ]]; then
        build_frontend
    fi

    # 3. 构建后端
    if [[ "$SKIP_BUILD" != true ]]; then
        build_backend
    fi

    # 4. 推送镜像到 Harbor
    if [[ "$SKIP_DEPLOY" != true ]]; then
        push_all_images
    fi

    # 5. 部署到 Rancher
    if [[ "$SKIP_DEPLOY" != true ]]; then
        deploy_all
    fi

    # 6. 验证部署
    if [[ "$SKIP_DEPLOY" != true ]]; then
        verify_deployment
    fi
}

# ---------- 主入口 ----------
main
