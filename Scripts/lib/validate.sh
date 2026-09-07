#!/bin/bash
# =============================================================================
# validate.sh — 参数验证函数库
# =============================================================================
# 用法: source "$(dirname "${0}")/../lib/validate.sh"
# 
# 该文件提供统一的参数验证函数，所有脚本应从此文件获取:
# - validate_not_empty <param_name> <param_value> — 验证参数不为空
# - validate_command <cmd_name> — 验证命令存在
# - validate_integer <param_name> <param_value> — 验证参数为整数
# =============================================================================

# 验证参数不为空
validate_not_empty() {
    local param_name="$1"
    local param_value="$2"
    
    if [[ -z "$param_value" ]]; then
        echo "[ERROR] 参数 $param_name 不能为空" >&2
        return 1
    fi
    return 0
}

# 验证命令存在
validate_command() {
    local cmd_name="$1"
    
    if ! command -v "$cmd_name" >/dev/null 2>&1; then
        echo "[ERROR] 命令 $cmd_name 未安装" >&2
        return 1
    fi
    return 0
}

# 验证参数为整数
validate_integer() {
    local param_name="$1"
    local param_value="$2"
    
    if [[ ! "$param_value" =~ ^[0-9]+$ ]]; then
        echo "[ERROR] 参数 $param_name 必须为整数，当前值: $param_value" >&2
        return 1
    fi
    return 0
}
