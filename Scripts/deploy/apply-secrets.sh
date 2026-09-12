#!/bin/bash
# Scripts/deploy/apply-secrets.sh
# 应用密钥到 Kubernetes 集群
# 用法：./Scripts/deploy/apply-secrets.sh [environment]
# 示例：./Scripts/deploy/apply-secrets.sh prod
#        ./Scripts/deploy/apply-secrets.sh prod --namespace myai-prod

set -euo pipefail

# =============================================================================
# 配置
# =============================================================================
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly MYAI_ROOT="/d/Workspace/MyAI"
readonly RANCHER_HOST="https://100.100.100.101.example.com:8800"
readonly RANCHER_USER="admin"
readonly RANCHER_PASS="${RANCHER_PASS:?Error: RANCHER_PASS environment variable required}"
readonly NAMESPACE="${1:-myai-prod}"
readonly TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
readonly VERSION="${VERSION:-v1.29.0}"

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
log INFO "MyAI Secrets Application"
log INFO "=========================================="
log INFO "Namespace: ${NAMESPACE}"
log INFO "Timestamp: ${TIMESTAMP}"
log INFO ""

check_command podman "Podman"
check_command jq "jq"
check_command kubectl "kubectl"

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

# 检查 Namespace 是否存在
log INFO "检查 Namespace: ${NAMESPACE}..."
if kubectl get namespace "${NAMESPACE}" &>/dev/null; then
    log INFO "  ✓ Namespace 存在"
else
    log WARN "  ⚠ Namespace 不存在，将尝试创建"
fi

# =============================================================================
# 步骤 2: 应用前端密钥
# =============================================================================
log INFO ""
log INFO "步骤 2: 应用前端密钥..."
log INFO "  Namespace: ${NAMESPACE}"
log INFO ""

# 前端密钥列表
FRONTEND_SECRETS=(
    "VITE_OPENAI_API_KEY"
    "VITE_ANTHROPIC_API_KEY"
    "VITE_GROQ_API_KEY"
    "VITE_MISTRAL_API_KEY"
    "VITE_GITHUB_PERSONAL_ACCESS_TOKEN"
    "VITE_TAVILY_API_KEY"
    "VITE_GOOGLE_GEMINI_API_KEY"
)

# 创建前端密钥 Secret
log INFO "创建前端密钥 Secret..."

cat > /tmp/frontend-secrets.yaml << 'EOF'
apiVersion: v1
kind: Secret
metadata:
  name: frontend-secrets
  namespace: myai-prod
type: Opaque
data:
EOF

# 添加每个密钥
for key in "${FRONTEND_SECRETS[@]}"; do
    # 从 .env.example 读取默认值
    DEFAULT_VALUE=$(grep "^${key}=" /d/Workspace/MyAI/.env.example 2>/dev/null | cut -d'=' -f2 | tr -d '"')
    if [ -n "$DEFAULT_VALUE" ]; then
        ENCODED_VALUE=$(echo -n "$DEFAULT_VALUE" | base64)
        echo "  $key: $ENCODED_VALUE" >> /tmp/frontend-secrets.yaml
        log INFO "  ✓ ${key}: ${DEFAULT_VALUE}"
    else
        log WARN "  ⚠ ${key}: 未找到默认值，跳过"
    fi
done

# 应用 Secret
podman run --rm \
    -v /tmp:/tmp:ro \
    bitnami/kubectl:${VERSION:-v1.29.0} \
    apply -f /tmp/frontend-secrets.yaml -n ${NAMESPACE} 2>&1 | tail -n 10

log INFO "  ✓ 前端密钥已应用"

# =============================================================================
# 步骤 3: 应用后端密钥
# =============================================================================
log INFO ""
log INFO "步骤 3: 应用后端密钥..."
log INFO "  Namespace: ${NAMESPACE}"
log INFO ""

# 后端密钥列表（从 .env.example 提取）
BACKEND_SECRETS=(
    "JWT_SECRET"
    "JWT_SECRET_GRPC"
    "JWT_SECRET_HTTP"
    "JWT_SECRET_GRPC_ADMIN"
    "JWT_SECRET_HTTP_ADMIN"
    "JWT_SECRET_REFRESH"
    "JWT_SECRET_REFRESH_ADMIN"
    "JWT_SECRET_ACCESS"
    "JWT_SECRET_ACCESS_ADMIN"
    "JWT_SECRET_SESSION"
    "JWT_SECRET_SESSION_ADMIN"
    "JWT_SECRET_DEVICE"
    "JWT_SECRET_DEVICE_ADMIN"
    "JWT_SECRET_LOGIN"
    "JWT_SECRET_LOGIN_ADMIN"
    "REDIS_PASSWORD"
    "REDIS_PASSWORD_ADMIN"
    "POSTGRES_PASSWORD"
    "POSTGRES_PASSWORD_ADMIN"
    "GRPC_AUTH_TOKEN"
    "GRPC_AUTH_TOKEN_ADMIN"
    "PAY_KEY"
    "HAIKON_KEY"
    "HAIKON_SECRET"
    "HAIKON_SECRET_KEY"
)

# 创建后端密钥 Secret
log INFO "创建后端密钥 Secret..."

cat > /tmp/backend-secrets.yaml << 'EOF'
apiVersion: v1
kind: Secret
metadata:
  name: backend-secrets
  namespace: myai-prod
type: Opaque
data:
EOF

# 添加每个密钥
for key in "${BACKEND_SECRETS[@]}"; do
    # 从 .env.example 读取默认值
    DEFAULT_VALUE=$(grep "^${key}=" /d/Workspace/MyAI/.env.example 2>/dev/null | cut -d'=' -f2 | tr -d '"')
    if [ -n "$DEFAULT_VALUE" ]; then
        ENCODED_VALUE=$(echo -n "$DEFAULT_VALUE" | base64)
        echo "  $key: $ENCODED_VALUE" >> /tmp/backend-secrets.yaml
        log INFO "  ✓ ${key}: ${DEFAULT_VALUE}"
    else
        log WARN "  ⚠ ${key}: 未找到默认值，跳过"
    fi
done

# 应用 Secret
podman run --rm \
    -v /tmp:/tmp:ro \
    bitnami/kubectl:${VERSION:-v1.29.0} \
    apply -f /tmp/backend-secrets.yaml -n ${NAMESPACE} 2>&1 | tail -n 10

log INFO "  ✓ 后端密钥已应用"

# =============================================================================
# 步骤 4: 应用 GRPC 认证 Token
# =============================================================================
log INFO ""
log INFO "步骤 4: 应用 GRPC 认证 Token..."
log INFO "  Namespace: ${NAMESPACE}"
log INFO ""

# 创建 GRPC Token Secret
cat > /tmp/grpc-token-secret.yaml << 'EOF'
apiVersion: v1
kind: Secret
metadata:
  name: grpc-token
  namespace: myai-prod
type: Opaque
data:
  grpc-token: ${GRPC_AUTH_TOKEN:-}
  grpc-token-admin: ${GRPC_AUTH_TOKEN_ADMIN:-}
EOF

# 如果有值，则应用
if [ -n "${GRPC_AUTH_TOKEN:-}" ] || [ -n "${GRPC_AUTH_TOKEN_ADMIN:-}" ]; then
    podman run --rm \
        -v /tmp:/tmp:ro \
        bitnami/kubectl:${VERSION:-v1.29.0} \
        apply -f /tmp/grpc-token-secret.yaml -n ${NAMESPACE} 2>&1 | tail -n 10
    log INFO "  ✓ GRPC Token 已应用"
else
    log WARN "  ⚠ GRPC Token: 未设置环境变量"
fi

# =============================================================================
# 步骤 5: 显示结果
# =============================================================================
log INFO ""
log INFO "=========================================="
log INFO "密钥应用完成"
log INFO "=========================================="
log INFO "Namespace: ${NAMESPACE}"
log INFO ""
log INFO "已应用的密钥:"
log INFO "  前端密钥:"
echo "$FRONTEND_SECRETS" | while read key; do
    DEFAULT_VALUE=$(grep "^${key}=" /d/Workspace/MyAI/.env.example 2>/dev/null | cut -d'=' -f2 | tr -d '"')
    if [ -n "$DEFAULT_VALUE" ]; then
        log INFO "    - ${key}: ${DEFAULT_VALUE}"
    fi
done
log INFO "  后端密钥:"
echo "$BACKEND_SECRETS" | while read key; do
    DEFAULT_VALUE=$(grep "^${key}=" /d/Workspace/MyAI/.env.example 2>/dev/null | cut -d'=' -f2 | tr -d '"')
    if [ -n "$DEFAULT_VALUE" ]; then
        log INFO "    - ${key}: ${DEFAULT_VALUE}"
    fi
done
log INFO ""
log INFO "下一步:"
log INFO "  1. 检查 Secret: kubectl get secret -n ${NAMESPACE}"
log INFO "  2. 部署应用：./Scripts/deploy/helm-deploy-auto-tags.sh"
