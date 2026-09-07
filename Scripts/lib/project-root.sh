#!/bin/bash
# =============================================================================
# project-root.sh — 项目根目录定位库
# =============================================================================
# 提供 get_project_root() 函数，返回 MyAI 项目根目录绝对路径。
# 优先级: MYAI_ROOT 环境变量 > 自动向上搜索(4层) > 失败返回空
# =============================================================================

set -euo pipefail

get_project_root() {
    # 优先使用环境变量
    if [[ -n "${MYAI_ROOT:-}" ]] && [[ -d "${MYAI_ROOT}/Scripts/lib" ]]; then
        printf '%s' "$MYAI_ROOT"
        return 0
    fi

    # 自动向上搜索 (最多4层)
    local SCRIPT_DIR
    SCRIPT_DIR="/d/Workspace/MyAI/Scripts/lib"
    local depth
    for depth in 0 1 2 3; do
        local candidate="$SCRIPT_DIR"
        for ((i=0; i<depth; i++)); do
            candidate="$(cd "$candidate/.." && pwd)"
        done
        if [[ -d "$candidate/Scripts/lib" ]]; then
            printf '%s' "$candidate"
            return 0
        fi
    done

    # 搜索失败
    return 1
}
