#!/bin/bash
# =============================================================================
# rancher.sh — Rancher 认证函数库
# =============================================================================
# 提供 get_rancher_password() 函数，从环境变量或密钥文件获取 Rancher 密码
# =============================================================================

# ---------- 获取 Rancher 密码 ----------
# 优先级: 环境变量(RANCHER_PASSWORD) > 密钥文件(~/.myai/secrets/rancher-password) > 交互式输入
get_rancher_password() {
    local secret_dir="$HOME/.myai/secrets"

    # 1. 环境变量
    if [[ -n "${RANCHER_PASSWORD:-}" ]]; then
        echo "$RANCHER_PASSWORD"
        return 0
    fi

    # 2. 密钥文件
    if [[ -d "$secret_dir" ]]; then
        local secret_file="$secret_dir/rancher-password"
        if [[ -f "$secret_file" ]]; then
            cat "$secret_file" 2>/dev/null && return 0
        fi
    fi

    # 3. 交互式输入 (后备)
    read -rsp 'Rancher password: ' RANCHER_PASS
    echo "$RANCHER_PASS"
    return 0
}
