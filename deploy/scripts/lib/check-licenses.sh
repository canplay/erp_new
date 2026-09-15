#!/bin/bash
# =============================================================================
# check-licenses.sh — 许可证检查脚本
# =============================================================================
# 用法: source "$(dirname "${0}")/../lib/check-licenses.sh"
# 
# 该文件提供许可证检查函数，所有脚本应从此文件获取:
# - check_licenses — 检查所有脚本的许可证声明
# =============================================================================

# 检查所有脚本的许可证声明
check_licenses() {
    local script_dir
    script_dir="/d/Workspace/MyAI/Scripts/lib"
    
    echo "[INFO] 检查脚本许可证声明..."
    local exit_code=0
    
    # 使用 globstar 检查所有脚本
    shopt -s globstar
    for script in "$script_dir"/**/*.sh; do
        if ! head -5 "$script" | grep -q "LICENSE\|MIT\|Apache"; then
            echo "[WARN] $script 缺少许可证声明"
            exit_code=1
        fi
    done
    
    return $exit_code
}
