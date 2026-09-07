#!/bin/bash
# =============================================================================
# unify-logs.sh — 统一所有脚本的日志函数调用
# =============================================================================
# 将所有脚本中的 echo "[INFO]"/echo "[WARN]" 等替换为 log_info/log_warn 等
# =============================================================================

set -euo pipefail

SCRIPT_DIR="/d/Workspace/MyAI/Scripts/lib"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 需要替换的日志模式
declare -A LOG_PATTERNS=(
    ["echo \"\[INFO\]"]='log_info'
    ["echo \"\[WARN\]"]='log_warn'
    ["echo \"\[ERROR\]"]='log_error'
    ["echo \"\[DEBUG\]"]='log_debug'
    ["echo \"\[OK\]"]='log_success'
)

# 处理每个脚本
for script in "$PROJECT_ROOT/Scripts/"**/*.sh; do
    [[ -f "$script" ]] || continue

    changed=false

    for pattern in "${!LOG_PATTERNS[@]}"; do
        replacement="${LOG_PATTERNS[$pattern]}"
        
        if grep -q "$pattern" "$script" 2>/dev/null; then
            sed -i "s/$pattern/$replacement/g" "$script"
            log_info "已替换: $script ($pattern -> $replacement)"
            changed=true
        fi
    done
    
    if [[ "$changed" == true ]]; then
        # 确保脚本 source 了 logging.sh
        if ! grep -q "logging.sh" "$script" 2>/dev/null; then
            # 在 set -euo pipefail 后添加 source
            sed -i '/set -euo pipefail/a source "$(dirname "${0}")/../lib/logging.sh"' "$script"
            log_info "已添加 logging.sh source: $script"
        fi
    fi
done

log_success "日志函数统一完成"
