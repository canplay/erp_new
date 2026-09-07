#!/bin/bash
# =============================================================================
# version.sh — 版本管理函数库
# =============================================================================
# 用法: source "$(dirname "${0}")/../lib/version.sh"
# 
# 该文件提供统一的版本管理函数，所有脚本应从此文件获取:
# - get_version — 获取版本号
# - check_version_compatibility — 检查版本兼容性
# =============================================================================

# 当前版本号
CURRENT_VERSION="0.1.0"

# 获取版本号
get_version() {
    echo "$CURRENT_VERSION"
}

# 检查版本兼容性
check_version_compatibility() {
    local required_version="$1"
    local current_version="$(get_version)"
    
    # 简单版本比较
    if [[ "$current_version" < "$required_version" ]]; then
        echo "[ERROR] 版本不兼容: 需要 $required_version, 当前 $current_version" >&2
        return 1
    fi
    return 0
}
