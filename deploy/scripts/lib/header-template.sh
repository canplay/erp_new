#!/bin/bash
# =============================================================================
# [脚本名称]
# =============================================================================
# [一句话描述]
#
# 用法: [使用示例]
#
# 项目根目录: 自动定位, 可用环境变量 MYAI_ROOT 覆盖
# =============================================================================

set -euo pipefail
set -o nounset

# ---------- 加载库 ----------
SCRIPT_DIR="/d/Workspace/MyAI/Scripts/lib"
source "$SCRIPT_DIR/lib/project-root.sh"
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/services.sh"

PROJECT_ROOT="$(get_project_root)" || {
    log_error "无法定位项目根目录, 请设置 MYAI_ROOT 环境变量"
    exit 1
}

# ---------- 加载环境变量 ----------
source "$PROJECT_ROOT/Scripts/lib/env.sh"
load_env
