#!/bin/bash
# =============================================================================
# 环境变量加载库
# =============================================================================
# 优先级: 已存在的环境变量 > Scripts/.env > 脚本内默认值
# 用法:
#   source "$PROJECT_ROOT/Scripts/lib/env.sh"
#   load_env
# 说明:
#   - Scripts/.env 不存在时静默跳过（不报错）
#   - 已 export 的环境变量不会被 .env 覆盖（环境变量优先语义）
#   - 可用 MYAI_ENV_FILE 覆盖 .env 路径
#   - 支持 # 注释、空行、KEY=VALUE 格式
#   - 支持值两侧的双引号/单引号剥离: FOO="bar baz" → bar baz
#   - 不支持行内注释与转义字符求值 (保持字面值)
# =============================================================================

load_env() {
    # Ensure PROJECT_ROOT is available (may already be set by project-root.sh)
    if [[ -z "${PROJECT_ROOT:-}" ]]; then
        PROJECT_ROOT="$(get_project_root)" || true
    fi
    local ENV_FILE="${MYAI_ENV_FILE:-$PROJECT_ROOT/Scripts/.env}"
    [[ -f "$ENV_FILE" ]] || return 0

    local line key value
    while IFS= read -r line || [[ -n "$line" ]]; do
        # 跳过空行与注释
        [[ -z "$line" ]] && continue
        [[ "$line" == \#* ]] && continue
        [[ "$line" =~ ^[[:space:]]*# ]] && continue

        # 提取 KEY (等号之前), VALUE (等号之后)
        key="${line%%=*}"
        key="$(printf '%s' "$key" | tr -d '[:space:]')"
        [[ -z "$key" ]] && continue

        # 环境变量优先: 已存在的变量跳过, 不被 .env 覆盖
        if [[ -n "${!key+x}" ]]; then
            continue
        fi

        value="${line#*=}"
        # 剥离首尾成对引号 (双引号或单引号)
        if [[ "$value" == \"*\" ]] || [[ "$value" == \'*\' ]]; then
            value="${value:1:${#value}-2}"
        fi

        export "$key=$value"
    done < "$ENV_FILE"
    return 0
}

# =============================================================================
# TAG 生成函数
# =============================================================================
# 生成唯一 TAG，格式: YYYYMMDDHHMM
# 用法:
#   TAG="$(generate_tag)"
# =============================================================================
generate_tag() {
    date +%Y%m%d%H%M
}