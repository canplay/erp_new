#!/bin/bash
# =============================================================================
# 后端部署到Rancher
# =============================================================================
# 用法: ./Scripts/deploy/backend/deploy-to-rancher.sh [-t <tag>] [-n <namespace>] [--auto-tags]
# 项目根目录: 自动定位, 可用环境变量 MYAI_ROOT 覆盖

set -euo pipefail

# ---------- 加载公共库 ----------
SCRIPT_DIR="$(cd "$(dirname "${0}")" && pwd)"
source "$SCRIPT_DIR/lib/all.sh"

# ---------- 参数解析 ----------
TAG=""
NAMESPACE="${NAMESPACE:-myai-prod}"
AUTO_TAGS=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    -t|--tag) TAG="$2"; shift 2 ;;
    -n|--namespace) NAMESPACE="$2"; shift 2 ;;
    --auto-tags) AUTO_TAGS=true; shift ;;
    *) echo "未知参数: $1 (用法: $0 [-t <tag>] [-n <namespace>] [--auto-tags])" >&2; exit 1 ;;
  esac
done

# ---------- 配置 ----------
HARBOR_URL="${HARBOR_URL:-harbor.100.100.100.101.example.com:8003}"
HARBOR_PROJECT="${HARBOR_PROJECT:-myai}"
HARBOR_USER="${HARBOR_USER:-admin}"
HARBOR_PASS="${HARBOR_PASS:?Error: HARBOR_PASS environment variable required}"
RANCHER_URL="${RANCHER_URL:-https://100.100.100.101.example.com:8800}"
RANCHER_USER="${RANCHER_USER:-admin}"
HELM_CHART="$PROJECT_ROOT/Scripts/helm/myai"

# ---------- 自动生成 tag ----------
if [[ -z "$TAG" ]]; then
    TAG="$(date +%Y%m%d%H%M)"
    log_info "使用自动生成 TAG: $TAG"
fi

echo "===== MyAI 后端部署到Rancher - TAG: $TAG / NS: $NAMESPACE ====="

# ---------- 检查工具 ----------
if ! podman_check; then
    exit 1
fi
if ! kubectl_check; then
    exit 1
fi

# ---------- 获取 Rancher 密码 ----------
source "$PROJECT_ROOT/Scripts/lib/rancher.sh"
RANCHER_PASS="$(get_rancher_password)"

# ---------- 登录 Harbor ----------
echo '[1/7] Login Harbor...'
if ! harbor_login "$HARBOR_URL" "$HARBOR_PROJECT" "$HARBOR_USER" "$HARBOR_PASS"; then
    echo 'ERROR: Harbor 登录失败' >&2
    exit 1
fi
echo '[OK]'

# ---------- 后端服务列表 (21个) ----------
declare -A SERVICES=(
    [api-gateway]=8090
    [auth-service]=8081
    [user-service]=8080
    [cms-service]=8082
    [messaging-service]=8083
    [file-service]=8086
    [tenant-service]=8087
    [workflow-service]=8088
    [feedback-service]=8085
    [audit-service]=8089
    [api-key-service]=9091
    [hik-service]=8092
    [pay-service]=8093
    [clean-service]=8095
    [tow-service]=8094
    [ctp-service]=8096
    [xlt-service]=8097
    [ebike-service]=8098
    [lpr-service]=8099
    [social-ops-service]=8110
    [browser-service]=8120
)

# ---------- 推送所有后端镜像 ----------
echo '[2/7] Push backend services...'
for svc in "${!SERVICES[@]}"; do
    echo ""
    echo "=== push myai-$svc:$TAG ==="
    if harbor_push_image "myai-$svc" "$TAG" "$HARBOR_URL" "$HARBOR_PROJECT"; then
        log_success "myai-$svc"
    else
        log_warn "myai-$svc 推送失败, 将拉取 Harbor 上已存在的 tag"
    fi
done

# ---------- 登录 Rancher ----------
echo '[3/7] Login Rancher + get kubeconfig...'
if ! rancher_login "$RANCHER_URL" "$RANCHER_USER" "$RANCHER_PASS"; then
    echo 'ERROR: Rancher 登录失败' >&2
    exit 1
fi
echo '[OK]'

# ---------- Helm 部署 ----------
echo '[4/7] Helm deploy...'

# ---------- 自动获取每个服务最新tag ----------
HELM_SET_ARGS=()
if [[ "$AUTO_TAGS" == true ]]; then
    log_info "自动获取每个服务在Harbor的最新tag..."
    for svc_key in "${!SERVICES[@]}"; do
        img="myai-${svc_key//-/_}"
        HELM_SET_ARGS+=(--set "services.${svc_key//-/_}.tag=$TAG")
        echo "  $svc_key -> $img:$TAG"
    done
    log_warn "未匹配到Harbor仓库的服务使用全局TAG: $TAG"
fi

if helm_deploy "$HELM_CHART" "$NAMESPACE" "$HELM_CHART/values-prod-harbor.yaml" "$TAG" "$AUTO_TAGS" "${HELM_SET_ARGS[@]+"${HELM_SET_ARGS[@]}"}"; then
    echo '[OK] Helm 部署成功'
else
    echo 'ERROR: Helm 部署失败' >&2
    exit 1
fi

# ---------- 密钥重放 ----------
echo '[5/7] Re-apply secrets...'
APPLY_SECRETS="$PROJECT_ROOT/Scripts/deploy/apply-secrets.sh"
if [[ -f "$APPLY_SECRETS" ]]; then
    bash "$APPLY_SECRETS" "$NAMESPACE"
    kubectl rollout restart deploy -n "$NAMESPACE" -l app.kubernetes.io/instance=myai >/dev/null 2>&1 || true
else
    echo '[WARN] apply-secrets.sh 不存在, 跳过密钥重放' >&2
fi

# ---------- 初始化数据库 ----------
echo '[6/7] Init database...'
SCHEMA_FILE="$PROJECT_ROOT/Backend/sql/schema.sql"
if [[ -f "$SCHEMA_FILE" ]]; then
    echo "  导入 schema.sql..."
    kubectl exec -n "$NAMESPACE" pg-cluster-postgresql-0 -- psql myai -f "$SCHEMA_FILE" 2>/dev/null || true
else
    echo 'WARN: schema.sql not found, skip DB init' >&2
fi

# ---------- 清理异常 Pod ----------
echo '[7/7] Cleanup pods...'
cleanup_pods "$NAMESPACE"

# ---------- 等待 Pod 就绪 ----------
echo '等待 Pod 就绪...'
sleep 30
kubectl rollout status deploy -n "$NAMESPACE" -l app.kubernetes.io/instance=myai --timeout=2m >/dev/null 2>&1 || true

if wait_pods_ready "$NAMESPACE"; then
    echo 'SUCCESS: 所有后端服务就绪!'
else
    echo 'WARNING: 部分 Pod 未就绪' >&2
fi
