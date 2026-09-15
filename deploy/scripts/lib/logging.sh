#!/bin/bash
# =============================================================================
# logging.sh — 统一日志库
# =============================================================================
# 提供 log_info/log_warn/log_error/log_debug/log_success 函数
# 支持 LOG_LEVEL 环境变量控制输出级别 (默认: info)
# =============================================================================

# 日志级别: 0=debug, 1=info, 2=warn, 3=error
LOG_LEVEL="${LOG_LEVEL:-1}"

# 颜色定义（仅首次加载时设置）
[[ -z "${_LOGGING_LOADED:-}" ]] || return 0
readonly RED='\033[0;31m'
readonly YELLOW='\033[0;33m'
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# 日志级别数字映射
declare -A LEVEL_NUM=(
    [debug]=0
    [info]=1
    [warn]=2
    [error]=3
)

# 检查日志级别
should_log() {
    local level="$1"
    local level_num="${LEVEL_NUM[$level]:-1}"
    local current_num="${LEVEL_NUM[$(echo "$LOG_LEVEL" | tr '[:upper:]' '[:lower:]')]:-1}"
    [[ $level_num -ge $current_num ]]
}

# log_debug "消息" — 仅 $LOG_LEVEL=debug 时输出
log_debug() {
    [[ $(should_log "debug") ]] || return 0
    echo -e "${BLUE}[DEBUG]${NC} $(date +%H:%M:%S) $*" >&2
}

# log_info "消息" — 默认输出级别
log_info() {
    [[ $(should_log "info") ]] || return 0
    echo "[INFO] $(date +%H:%M:%S) $*"
}

# log_warn "消息" — 警告级别
log_warn() {
    [[ $(should_log "warn") ]] || return 0
    echo -e "${YELLOW}[WARN]$(NC) $(date +%H:%M:%S) $*" >&2
}

# log_error "消息" — 错误级别
log_error() {
    [[ $(should_log "error") ]] || return 0
    echo -e "${RED}[ERROR]$(NC) $(date +%H:%M:%S) $*" >&2
}

# log_success "消息" — 成功标记
log_success() {
    echo -e "${GREEN}[OK]$(NC) $(date +%H:%M:%S) $*"
}

# log_step "N/M 描述" — 步骤编号
log_step() {
    log_info "$*"
}

# log_retry_status "当前次数/最大次数" — 重试提示
log_retry_status() {
    local retries="$1"
    local max_retries="$2"
    local remaining=$((max_retries - retries))
    if (( retries > 0 )); then
        log_warn "第 $retries/$max_retries 次重试，剩余 $remaining 次机会"
    fi
}

# 标记已加载，防止重复 source
_LOGGING_LOADED=1
