#!/bin/bash
# =============================================================================
# secrets.sh — 统一的密码读取函数库
# =============================================================================
# 用法: source "$(dirname "${0}")/../lib/secrets.sh"
# 
# 该文件提供统一的密码读取函数，所有脚本应从此文件获取密码:
# - read_secret <secret_name> — 读取密码文件并返回密码内容
# - get_secret_path <secret_name> — 返回密码文件路径
# 
# 环境变量:
# - MYAI_SECRET_DIR — 密码文件目录（默认: $HOME/.myai/secrets）
# =============================================================================

# 获取密码文件路径
get_secret_path() {
    local secret_name="$1"
    local secret_dir="${MYAI_SECRET_DIR:-$HOME/.myai/secrets}"
    echo "$secret_dir/$secret_name"
}

# 读取密码（带存在性检查）
read_secret() {
    local secret_name="$1"
    local secret_path="$(get_secret_path "$secret_name")"
    
    if [[ ! -f "$secret_path" ]]; then
        echo "[ERROR] 密码文件不存在: $secret_path" >&2
        return 1
    fi
    
    local password
    password="$(cat "$secret_path")"
    echo "$password"
}
