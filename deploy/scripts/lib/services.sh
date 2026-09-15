#!/bin/bash
# =============================================================================
# services.sh — MyAI 服务列表和映射定义
# =============================================================================
# 所有服务相关的静态数据定义在此, 避免多处重复
# =============================================================================

# 后端服务列表 (服务名=端口)
declare -A SERVICES=(
    [api-gateway]=8090
    [auth-service]=8081
    [user-service]=8080
    [cms-service]=8082
    [messaging-service]=8083
    [file-service]=8086
    [tenant-service]=8087
    [workflow-service]=8088
    [feedback-service]=8085
    [audit-service]=8089
    [api-key-service]=9091
    [hik-service]=8092
    [pay-service]=8093
    [clean-service]=8095
    [tow-service]=8094
    [ctp-service]=8096
    [xlt-service]=8097
    [ebike-service]=8098
    [lpr-service]=8099
    [social-ops-service]=8110
    [browser-service]=8120
)

# Helm services key -> Harbor 镜像名映射
declare -A SVC_IMAGE_MAP=(
    [apiGateway]=myai-api-gateway
    [authService]=myai-auth-service
    [userService]=myai-user-service
    [cmsService]=myai-cms-service
    [messageService]=myai-messaging-service
    [fileService]=myai-file-service
    [tenantService]=myai-tenant-service
    [workflowService]=myai-workflow-service
    [feedbackService]=myai-feedback-service
    [auditService]=myai-audit-service
    [apiKeyService]=myai-api-key-service
    [hikService]=myai-hik-service
    [payService]=myai-pay-service
    [cleanService]=myai-clean-service
    [towService]=myai-tow-service
    [ctpService]=myai-ctp-service
    [xltService]=myai-xlt-service
    [ebikeService]=myai-ebike-service
    [lprService]=myai-lpr-service
    [socialOpsService]=myai-social-ops-service
    [browserService]=myai-browser-service
    [admin]=myai-admin
)

# 获取服务数量
get_service_count() {
    echo "${#SERVICES[@]}"
}

# 获取所有服务名(空格分隔)
get_service_names() {
    printf '%s' "${!SERVICES[@]}"
}
