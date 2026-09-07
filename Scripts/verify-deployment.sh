#!/bin/bash
# =============================================================================
# verify-deployment.sh — 部署后验证
# =============================================================================
# 用法: ./Scripts/verify-deployment.sh [-n <NAMESPACE>] [-u <API_BASE_URL>]
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${0}")" && pwd)"
source "$SCRIPT_DIR/lib/all.sh"

NAMESPACE="${1:-myai-prod}"
API_BASE_URL="${2:-https://api.100.100.100.100.example.com/api}"

log_step "验证部署..."

# 1. 检查所有Pod状态
log_info "检查 Pod 状态..."
kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/instance=myai --no-headers | while read -r line; do
    status="$(echo "$line" | awk '{print $3}')"
    if [[ "$status" != "Running" ]]; then
        log_error "Pod 异常: $line"
    else
        log_success "Pod 正常: $(echo "$line" | awk '{print $1}')"
    fi
done

# 2. 检查API网关健康
log_info "检查 API 网关健康..."
response="$(curl -sk --max-time 10 "$API_BASE_URL/health" 2>/dev/null || true)"
if [[ -n "$response" ]]; then
    log_success "API 网关健康检查通过"
else
    log_warn "API 网关健康检查返回空"
fi

# 3. 检查Ingress状态
log_info "检查 Ingress..."
kubectl get ingress -n "$NAMESPACE" -l app.kubernetes.io/instance=myai 2>/dev/null || true

log_success "验证完成"
