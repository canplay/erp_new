#!/bin/bash
# =============================================================================
# shellcheck.sh — ShellCheck 检查脚本
# =============================================================================
# 用法: source "$(dirname "${0}")/../lib/shellcheck.sh"
# 
# 该文件提供 ShellCheck 检查函数，所有脚本应从此文件获取:
# - run_shellcheck — 运行 ShellCheck 检查所有脚本
# =============================================================================

# 检查是否安装 shellcheck
run_shellcheck() {
    if ! command -v shellcheck >/dev/null 2>&1; then
        echo "[WARN] shellcheck 未安装，跳过检查" >&2
        return 0
    fi

    # 检查所有脚本
    echo "[INFO] 运行 ShellCheck 检查..."
    local script_dir
    script_dir="/d/Workspace/MyAI/Scripts/lib"
    
    # 使用 globstar 检查所有脚本
    shopt -s globstar
    local exit_code=0
    for script in "$script_dir"/**/*.sh; do
        if ! shellcheck "$script" >/dev/null 2>&1; then
            echo "[ERROR] $script 存在语法错误" >&2
            exit_code=1
        fi
    done
    
    return $exit_code
}
