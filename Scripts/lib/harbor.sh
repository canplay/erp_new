#!/bin/bash
# Scripts/lib/harbor.sh — Harbor 操作函数库
# =============================================================================
# 用于 Harbor 镜像仓库的登录、推送、拉取等操作
# =============================================================================

# ---------- Harbor 登录 ----------
harbor_login() {
    local harbor_url="$1"
    local harbor_project="$2"
    local harbor_user="$3"
    local harbor_pass="$4"
    
    # 检查是否已登录
    if command -v podman >/dev/null 2>&1; then
        echo "$harbor_pass" | podman login "$harbor_url" -u "$harbor_user" --password-stdin
        return $?
    else
        log_error "podman 命令不存在"
        return 1
    fi
}

# ---------- Harbor 推送镜像 ----------
harbor_push_image() {
    local image_name="$1"
    local tag="$2"
    local harbor_url="$3"
    local harbor_project="$4"

    local image="$image_name:$tag"
    local target="$harbor_url/$harbor_project/$image_name:$tag"

    log_info "推送 $image -> $target"

    if podman push --tls-verify=false "$image" "$target" 2>&1 | tee /dev/stderr; then
        log_success "$image_name 推送成功"
        return 0
    else
        log_error "$image_name 推送失败"
        return $EXIT_PUSH_ERROR
    fi
}

# ---------- Harbor 拉取镜像 ----------
harbor_pull_image() {
    local image_name="$1"
    local harbor_url="$2"
    local harbor_project="$3"
    
    # 从 Harbor 拉取镜像
    podman pull "$harbor_url/$harbor_project/$image_name"
    return $?
}

# ---------- 获取 Harbor 密码 ----------
# 优先级: 环境变量(HARBOR_PASSWORD) > 密钥文件(~/.myai/secrets/harbor-password)
harbor_get_password() {
    local harbor_url="$1"
    local secret_dir="$HOME/.myai/secrets"
    local secret_file="$secret_dir/harbor-password"
    
    if [[ -f "$secret_file" ]]; then
        cat "$secret_file" 2>/dev/null && return 0
    fi
    if [[ -n "${HARBOR_PASSWORD:-}" ]]; then
        echo "$HARBOR_PASSWORD"
        return 0
    fi
    log_error "无法获取 Harbor 密码 ($secret_file 不存在且 HARBOR_PASSWORD 未设置)"
    return 1
}

# ---------- 环境变量验证 ----------
validate_env() {
    local required_vars=("$@")
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            log_error "缺少必需的环境变量: $var"
            log_error "请设置: export $var=<value>"
            exit 1
        fi
    done
}
