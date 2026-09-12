#!/bin/bash
# =============================================================================
# all.sh — 统一导出所有公共函数库
# =============================================================================
# 用法: source "$(dirname "${0}")/../lib/all.sh"
# 
# 该文件会按依赖顺序 source 所有公共库:
# - logging.sh — 日志函数
# - services.sh — 服务定义
# - harbor.sh — Harbor 相关函数
# - rancher.sh — Rancher 相关函数
# - kubectl.sh — Kubectl 相关函数
# - helm.sh — Helm 相关函数
# - podman.sh — Podman 相关函数
# - wait-for-pods.sh — Pod 等待函数
# - project-root.sh — 项目根定位
# - env.sh — 环境变量加载
# - exit-codes.sh — 退出码定义
# - secrets.sh — 密码读取函数
# - get-harbor-latest-tag.sh — Harbor 最新tag查询
# =============================================================================

# all.sh is at /d/Workspace/MyAI/Scripts/lib/all.sh
# Use dynamic path based on this file's location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 凭证: 优先从环境变量读取，未设置时报错
: "${HARBOR_PASS:?Error: HARBOR_PASS environment variable required}"
: "${RANCHER_PASS:?Error: RANCHER_PASS environment variable required}"

source "$SCRIPT_DIR/logging.sh"
source "$SCRIPT_DIR/services.sh"
source "$SCRIPT_DIR/harbor.sh"
source "$SCRIPT_DIR/rancher.sh"
source "$SCRIPT_DIR/kubectl.sh"
source "$SCRIPT_DIR/helm.sh"
source "$SCRIPT_DIR/podman.sh"
source "$SCRIPT_DIR/wait-for-pods.sh"
source "$SCRIPT_DIR/project-root.sh"
source "$SCRIPT_DIR/env.sh"
source "$SCRIPT_DIR/exit-codes.sh"
source "$SCRIPT_DIR/secrets.sh"
source "$SCRIPT_DIR/get-harbor-latest-tag.sh"
