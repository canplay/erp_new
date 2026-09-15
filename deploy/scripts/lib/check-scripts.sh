#!/bin/bash
# =============================================================================
# check-scripts.sh — 检查所有脚本的一致性
# =============================================================================

set -euo pipefail
shopt -s globstar

SCRIPT_DIR="/d/Workspace/MyAI/Scripts/lib"

# 加载日志库
source "$SCRIPT_DIR/lib/logging.sh"

errors=0
warnings=0

# 检查所有脚本是否有 shebang
for script in "$SCRIPT_DIR"/../"**/*.sh"; do
    [[ -f "$script" ]] || continue
    
    # 检查 shebang
    if ! head -1 "$script" | grep -q "^#!/bin/bash"; then
        log_error "$script: 缺少 #!/bin/bash shebang"
        errors=$((errors + 1))
    fi

    # 检查是否设置了 set -euo pipefail
    if ! grep -q "set -euo pipefail" "$script"; then
        log_warn "$script: 未设置 set -euo pipefail"
        warnings=$((warnings + 1))
    fi

    # 检查是否加载了 project-root.sh
    if ! grep -q "project-root.sh" "$script"; then
        log_warn "$script: 未加载 project-root.sh"
        warnings=$((warnings + 1))
    fi

    # 检查是否加载了 logging.sh
    if ! grep -q "logging.sh" "$script"; then
        log_warn "$script: 未加载 logging.sh"
        warnings=$((warnings + 1))
    fi
done

if [[ $errors -gt 0 ]]; then
    log_error "发现 $errors 个错误"
    exit 1
fi

if [[ $warnings -gt 0 ]]; then
    log_warn "发现 $warnings 个警告"
fi

log_success "所有脚本检查通过"
