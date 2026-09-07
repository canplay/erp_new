#!/bin/bash
# =============================================================================
# Helm 操作函数库
# =============================================================================
# 提供 Helm 部署、升级等操作

# ---------- Helm 部署 ----------
helm_deploy() {
    local helm_chart="$1"
    local namespace="$2"
    local values_file="$3"
    local tag="$4"
    local auto_tags="${5:-false}"

    log_info "Helm 部署: chart=$helm_chart ns=$namespace tag=$tag"

    local HELM_CMD=(helm upgrade --install myai "$helm_chart" -n "$namespace")
    HELM_CMD+=(-f "$values_file")
    HELM_CMD+=(--set "image.tag=$tag")
    HELM_CMD+=(--set 'global.imagePullSecrets[0].name=harbor-secret')
    HELM_CMD+=(--timeout 15m --rollback-on-failure --wait)

    if [[ "$auto_tags" == true ]]; then
        log_info "自动标签模式: 每个服务使用Harbor最新tag"
        for svc_key in "${!SVC_IMAGE_MAP[@]}"; do
            local img="${SVC_IMAGE_MAP[$svc_key]}"
            local svc_tag
            svc_tag="$(get_harbor_latest_tag "$img" "${HARBOR_URL:-harbor.100.100.100.101.example.com:8003}" "${HARBOR_PROJECT:-erp}")"
            if [[ -n "$svc_tag" ]]; then
                HELM_CMD+=(--set "services.$svc_key.tag=$svc_tag")
                log_info "  $svc_key -> $svc_tag"
            else
                log_warn "$svc_key 未找到最新tag，使用默认: $tag"
                HELM_CMD+=(--set "services.$svc_key.tag=$tag")
            fi
        done
    fi

    log_info "执行: ${HELM_CMD[*]}"
    if "${HELM_CMD[@]}"; then
        log_success "Helm 部署成功"
        return 0
    else
        log_error "Helm 部署失败"
        return $EXIT_DEPLOY_ERROR
    fi
}
