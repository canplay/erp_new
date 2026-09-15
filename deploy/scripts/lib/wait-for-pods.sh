#!/bin/bash
# =============================================================================
# wait-for-pods.sh — Pod就绪等待库
# =============================================================================
# 提供 wait_pods_ready() 函数，轮询检查Pod就绪状态
# 支持超时和轮询间隔配置
# =============================================================================

# ---------- 加载公共库 ----------
SCRIPT_DIR="/d/Workspace/MyAI/Scripts/lib"
source "$SCRIPT_DIR/logging.sh"

# 日志级别: 0=debug, 1=info, 2=warn, 3=error
LOG_LEVEL="${LOG_LEVEL:-1}"

# 等待所有Pod就绪, 带超时和轮询
# 用法: wait_pods_ready <namespace> [timeout_seconds] [poll_interval_seconds]
# 默认超时: 300秒 (5分钟)
# 默认轮询间隔: 5秒

wait_pods_ready() {
    local namespace="${1:-myai-prod}"
    local timeout="${2:-300}"
    local poll_interval="${3:-5}"
    local start_time
    start_time=$(date +%s)

    log_info "等待命名空间 '$namespace' 中所有Pod就绪..."

    while true; do
        local ready=0
        local total=0

        # 解析 kubectl get pods 输出
        while read -r _ status _; do
            total=$((total + 1))
            if [[ "$status" == "Running" ]] || [[ "$status" == "Ready" ]]; then
                ready=$((ready + 1))
            fi
        done < <(kubectl get pods -n "$namespace" --no-headers 2>/dev/null || true)

        if [[ $total -eq 0 ]]; then
            log_warn "命名空间 '$namespace' 中没有Pod"
            return 0
        fi

        if [[ $ready -eq $total ]]; then
            log_success "所有 $total 个Pod已就绪"
            return 0
        fi

        # 检查超时
        local elapsed=$(( $(date +%s) - start_time ))
        if [[ $elapsed -ge $timeout ]]; then
            log_error "等待Pod就绪超时 (${timeout}秒): $ready/$total 就绪"
            return 1
        fi

        log_info "等待中... $ready/$total 就绪 (${elapsed}s/${timeout}s)"
        sleep "$poll_interval"
    done
}
