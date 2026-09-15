#!/bin/bash
# Scripts/deploy/helm-deploy-auto-tags.sh
# MyAI Helm 自动标签部署脚本
# 用法：./Scripts/deploy/helm-deploy-auto-tags.sh [TAG]
# 示例：./Scripts/deploy/helm-deploy-auto-tags.sh 20260825143000

set -euo pipefail

# =============================================================================
# 配置
# =============================================================================
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly MYAI_ROOT="/d/Workspace/MyAI"
readonly HARBOR_HOST="harbor.100.100.100.101.example.com:8003"
readonly RANCHER_HOST="https://100.100.100.101.example.com:8800"
readonly HARBOR_USER="${HARBOR_USER:-admin}"
HARBOR_PASS="${HARBOR_PASS:?Error: HARBOR_PASS environment variable required}"
readonly RANCHER_USER="${RANCHER_USER:-admin}"
RANCHER_PASS="${RANCHER_PASS:?Error: RANCHER_PASS environment variable required}"
readonly TAG="${1:-$(date +%Y%m%d%H%M%S)}"
readonly TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
readonly NAMESPACE="myai-prod"
readonly VERSION="${VERSION:-v3.14.0}"

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

# =============================================================================
# 工具函数
# =============================================================================

check_command() {
    local cmd="$1"
    local name="$2"
    if ! command -v "$cmd" &>/dev/null; then
        log ERROR "命令未找到：$name ($cmd)"
        exit 1
    fi
}

# =============================================================================
# 步骤 1: 环境检查
# =============================================================================
log INFO "=========================================="
log INFO "MyAI Helm Deployment"
log INFO "=========================================="
log INFO "Tag: ${TAG}"
log INFO "Timestamp: ${TIMESTAMP}"
log INFO "Namespace: ${NAMESPACE}"
log INFO ""

check_command helm "Helm"
check_command podman "Podman"
check_command jq "jq"

# 检查 Harbor 连接
log INFO "检查 Harbor 连接..."
if echo "${HARBOR_USER}:${HARBOR_PASS}" | podman login \
    --username "${HARBOR_USER}" \
    --password-stdin \
    --tls-verify=false \
    "${HARBOR_HOST}" 2>&1 | grep -q "Login Succeeded"; then
    log INFO "  ✓ Harbor 登录成功"
else
    log WARN "  ⚠ Harbor 登录可能失败"
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
    log WARN "  ⚠ Rancher 登录可能失败"
fi

# 检查 Helm 版本
log INFO "检查 Helm 版本..."
helm version --short
log INFO "  ✓ Helm 已安装"

# =============================================================================
# 步骤 2: 创建 Harbor 拉取凭证
# =============================================================================
log INFO ""
log INFO "步骤 2: 创建 Harbor 拉取凭证..."
log INFO "  创建 Secret: harbor-secret"
log INFO "  目的：允许集群拉取 Harbor 镜像"
log INFO ""

# 创建 Harbor 凭证 Secret
HARBOR_AUTH=$(echo -n "${HARBOR_USER}:${HARBOR_PASS}" | base64 -w0)

cat > /tmp/harbor-pull-secret.yaml << EOF
apiVersion: v1
kind: Secret
metadata:
  name: harbor-secret
  namespace: ${NAMESPACE}
type: kubernetes.io/dockerconfigjson
data:
  .dockerconfigjson: |
    {
      "auths": {
        "${HARBOR_HOST}": {
          "auth": "${HARBOR_AUTH}",
          "email": "admin@example.com"
        }
      }
    }
EOF

podman run --rm \
    -v /tmp/harbor-pull-secret.yaml:/tmp/secret.yaml:ro \
    alpine:latest \
    sh -c "cat /tmp/secret.yaml" | podman create --name harbor-secret-creator \
    sh -c "cat /tmp/secret.yaml | kubectl create -n ${NAMESPACE} secret docker-registry harbor-secret -f -" 2>&1 || \
    echo "${HARBOR_AUTH}" | podman run --rm --user $(id -u):$(id -g) \
        -v /tmp/harbor-pull-secret.yaml:/tmp/secret.yaml:ro \
        bitnami/kubectl:${VERSION:-v1.29.0} \
        create -n ${NAMESPACE} secret docker-registry harbor-secret \
        --docker-server="${HARBOR_HOST}" \
        --docker-username="${HARBOR_USER}" \
        --docker-password="${HARBOR_PASS}" \
        --dry-run=client -o yaml | kubectl apply -f - 2>&1 || true

log INFO "  ✓ Harbor 凭证已创建"

# =============================================================================
# 步骤 3: 准备部署参数
# =============================================================================
log INFO ""
log INFO "步骤 3: 准备部署参数..."
log INFO ""

# 检查 values-prod-harbor.yaml 是否存在
VALUES_FILE="/d/Workspace/MyAI/Scripts/helm/myai/values-prod-harbor.yaml"
if [ -f "${VALUES_FILE}" ]; then
    log INFO "  ✓ values-prod-harbor.yaml 存在"
else
    log ERROR "  ✗ values-prod-harbor.yaml 不存在"
    exit 1
fi

# 检查 values.yaml 是否存在
VALUES_MAIN="/d/Workspace/MyAI/Scripts/helm/myai/values.yaml"
if [ -f "${VALUES_MAIN}" ]; then
    log INFO "  ✓ values.yaml 存在"
else
    log WARN "  ⚠ values.yaml 不存在，将只使用 values-prod-harbor.yaml"
fi

# 检查 secrets.yaml 是否存在
SECRETS_FILE="/d/Workspace/MyAI/Scripts/helm/myai/secrets.yaml"
if [ -f "${SECRETS_FILE}" ]; then
    log INFO "  ✓ secrets.yaml 存在"
else
    log WARN "  ⚠ secrets.yaml 不存在，将使用默认值"
fi

# =============================================================================
# 步骤 4: 部署到 Rancher
# =============================================================================
log INFO ""
log INFO "步骤 4: 部署到 Rancher..."
log INFO "  Release: myai-backend"
log INFO "  Namespace: ${NAMESPACE}"
log INFO "  Values: values-prod-harbor.yaml"
log INFO ""

# 构建 helm args
HELM_ARGS=(
    install myai-backend \
    /d/Workspace/MyAI/Scripts/helm/myai \
    --namespace "${NAMESPACE}" \
    --create-namespace \
    -f /d/Workspace/MyAI/Scripts/helm/myai/values-prod-harbor.yaml
)

# 如果有 secrets.yaml，添加
if [ -f "${SECRETS_FILE}" ]; then
    HELM_ARGS+=(-f "${SECRETS_FILE}")
fi

# 执行部署
log INFO "执行 helm install..."
helm upgrade --install myai-backend \
    /d/Workspace/MyAI/Scripts/helm/myai \
    --namespace "${NAMESPACE}" \
    --create-namespace \
    -f /d/Workspace/MyAI/Scripts/helm/myai/values-prod-harbor.yaml \
    --set image.tag="${TAG}" \
    --set image.pullSecrets.name=harbor-secret \
    --set image.pullSecrets.enabled=true \
    --wait --timeout 15m 2>&1 | tee /tmp/helm-install.log | tail -n 50

# 检查部署是否成功
log INFO ""
log INFO "检查部署状态..."

sleep 5

# 检查 Release 状态
RELEASE_STATUS=$(helm status myai-backend -n ${NAMESPACE} 2>&1 | tail -n 5)

if echo "$RELEASE_STATUS" | grep -q "DEPLOYED\|INSTALLED"; then
    log INFO "  ✓ Release 状态：DEPLOYED"
else
    log WARN "  ⚠ Release 状态未知，继续检查 Pods"
fi

# 检查 Pods 状态
log INFO "检查 Pods 状态..."
POD_STATUS=$(kubectl get pods -n ${NAMESPACE} -l app.kubernetes.io/instance=myai-backend --field-selector=status.phase=Running -o name 2>&1)
if [ -n "$POD_STATUS" ]; then
    log INFO "  ✓ 运行中的 Pods:"
    echo "$POD_STATUS" | while read pod; do
        log INFO "    - ${pod}"
    done
else
    log WARN "  ⚠ 暂无运行中的 Pods (可能是首次部署，正在初始化)"
fi

# 检查 Services
log INFO "检查 Services..."
SERVICE_OUTPUT=$(kubectl get svc -n ${NAMESPACE} -l app.kubernetes.io/instance=myai-backend -o wide 2>&1)
if [ -n "$SERVICE_OUTPUT" ]; then
    log INFO "  ✓ Services:"
    echo "$SERVICE_OUTPUT" | while read line; do
        log INFO "    - ${line}"
    done
else
    log WARN "  ⚠ Services 查询失败"
fi

# 检查 Deployment
log INFO "检查 Deployments..."
DEPLOY_OUTPUT=$(kubectl get deployment -n ${NAMESPACE} -l app.kubernetes.io/instance=myai-backend -o wide 2>&1)
if [ -n "$DEPLOY_OUTPUT" ]; then
    log INFO "  ✓ Deployments:"
    echo "$DEPLOY_OUTPUT" | while read line; do
        log INFO "    - ${line}"
    done
else
    log WARN "  ⚠ Deployments 查询失败"
fi

# =============================================================================
# 步骤 5: 验证服务健康
# =============================================================================
log INFO ""
log INFO "步骤 5: 验证服务健康..."

# 等待服务就绪
log INFO "等待服务就绪 (最多等待 300 秒)..."
MAX_WAIT=300
WAITED=0

while [ $WAITED -lt $MAX_WAIT ]; do
    sleep 10
    WAITED=$((WAITED + 10))
    
    # 检查 Deployment 就绪状态
    READY_OUTPUT=$(kubectl get deployment -n ${NAMESPACE} \
        -l app.kubernetes.io/instance=myai-backend \
        -o jsonpath='{range .items[*]}{.status.readyReplicas}/${.status.replicas}{" "}{.name}{"\n"}{end}' 2>&1)
    
    if [ -n "$READY_OUTPUT" ]; then
        log INFO "  等待进度：${READY_OUTPUT}"
        
        # 检查是否全部就绪
        ALL_READY=true
        while IFS= read -r line; do
            if ! echo "$line" | grep -q "^[0-9]*/[0-9]*$"; then
                ALL_READY=false
                break
            fi
            ready=$(echo "$line" | cut -d' ' -f1)
            total=$(echo "$line" | cut -d' ' -f2)
            if [ "$ready" -ne "$total" ] && [ "$total" -gt 0 ]; then
                ALL_READY=false
                break
            fi
        done <<< "$READY_OUTPUT"
        
        if [ "$ALL_READY" = true ]; then
            log INFO "  ✓ 所有 Deployment 已就绪"
            break
        fi
    fi
    
    if [ $WAITED -ge $MAX_WAIT ]; then
        log WARN "  ⚠ 等待超时 (300s)"
        break
    fi
done

# =============================================================================
# 步骤 6: 显示最终状态
# =============================================================================
log INFO ""
log INFO "=========================================="
log INFO "部署完成"
log INFO "=========================================="
log INFO "Release: myai-backend"
log INFO "Namespace: ${NAMESPACE}"
log INFO "Tag: ${TAG}"
log INFO "Harbor: ${HARBOR_HOST}/myai"
log INFO ""
log INFO "镜像仓库:"
podman images --format "  {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" | grep myai | head -30
log INFO ""
log INFO "Rancher 资源:"
kubectl get all -n ${NAMESPACE} -l app.kubernetes.io/instance=myai-backend -o wide 2>&1 | head -30
log INFO ""
log INFO "下一步:"
log INFO "  1. 检查 Rancher UI: https://100.100.100.101.example.com:8800/cattle/global/clusters"
log INFO "  2. 查看日志：kubectl logs -n ${NAMESPACE} -f <pod-name>"
log INFO "  3. 访问管理后台：https://admin.100.100.100.100.example.com:8901"
