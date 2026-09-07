#!/bin/bash
# =============================================================================
# 前端部署到Rancher
# =============================================================================
# 用法: ./Scripts/deploy/frontend/deploy-to-rancher.sh [-t <tag>] [-n <namespace>]
# 项目根目录: 自动定位, 可用环境变量 MYAI_ROOT 覆盖

set -euo pipefail

# ---------- 加载公共库 ----------
SCRIPT_DIR="$(cd "$(dirname "${0}")" && pwd)"
source "$SCRIPT_DIR/lib/all.sh"

# ---------- 参数解析 ----------
TAG=""
NAMESPACE="${NAMESPACE:-myai-prod}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    -t|--tag) TAG="$2"; shift 2 ;;
    -n|--namespace) NAMESPACE="$2"; shift 2 ;;
    *) echo "未知参数: $1 (用法: $0 [-t <tag>] [-n <namespace>])" >&2; exit 1 ;;
  esac
done

# ---------- 配置 ----------
HARBOR_URL="${HARBOR_URL:-harbor.100.100.100.101.example.com:8003}"
HARBOR_PROJECT="${HARBOR_PROJECT:-myai}"
HARBOR_USER="${HARBOR_USER:-admin}"
HARBOR_PASS="${HARBOR_PASS:-ChangeMeHarbor123!}"
RANCHER_URL="${RANCHER_URL:-https://100.100.100.101.example.com:8800}"
RANCHER_USER="${RANCHER_USER:-admin}"
HELM_CHART="$PROJECT_ROOT/Scripts/helm/myai"

# ---------- 自动生成 tag ----------
if [[ -z "$TAG" ]]; then
    TAG="$(date +%Y%m%d%H%M)"
    log_info "使用自动生成 TAG: $TAG"
fi

echo "===== MyAI 前端部署到Rancher - TAG: $TAG / NS: $NAMESPACE ====="

# ---------- 检查工具 ----------
if ! podman_check; then
    exit 1
fi
if ! kubectl_check; then
    exit 1
fi

# ---------- 获取 Rancher 密码 ----------
RANCHER_PASS="$(get_rancher_password)"

# ---------- 登录 Harbor ----------
echo '[1/4] Login Harbor...'
if ! harbor_login "$HARBOR_URL" "$HARBOR_PROJECT" "$HARBOR_USER" "$HARBOR_PASS"; then
    echo 'ERROR: Harbor 登录失败' >&2
    exit 1
fi
echo '[OK]'

# ---------- 推送前端镜像 ----------
echo '[2/4] Push myai-admin...'
if harbor_push_image "myai-admin" "$TAG" "$HARBOR_URL" "$HARBOR_PROJECT"; then
    echo '[OK] myai-admin 推送成功'
else
    echo '[WARN] myai-admin 推送失败, 部署将拉取 Harbor 上已存在的 tag' >&2
fi

# ---------- 登录 Rancher ----------
echo '[3/4] Login Rancher + get kubeconfig...'
if ! rancher_login "$RANCHER_URL" "$RANCHER_USER" "$RANCHER_PASS"; then
    echo 'ERROR: Rancher 登录失败' >&2
    exit 1
fi
echo '[OK]'

# ---------- Helm 部署 ----------
echo '[4/4] Helm deploy...'
if helm_deploy "$HELM_CHART" "$NAMESPACE" "$HELM_CHART/values-prod-harbor.yaml" "$TAG" false; then
    echo '[OK] Helm 部署成功'
else
    echo 'ERROR: Helm 部署失败' >&2
    exit 1
fi

# ---------- 等待 Pod 就绪 ----------
echo '等待 Pod 就绪...'
sleep 20
kubectl rollout status deploy -n "$NAMESPACE" -l app.kubernetes.io/instance=myai --timeout=2m >/dev/null 2>&1 || true

if wait_pods_ready "$NAMESPACE"; then
    echo 'SUCCESS: 前端服务就绪!'
else
    echo 'WARNING: 部分 Pod 未就绪' >&2
fi
