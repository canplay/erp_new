//! 租户服务 HTTP 处理器
//!
//! 提供租户管理的 HTTP REST API

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::repository::TenantRepository;
use common::ApiResponse;

// ============ 辅助函数 ============

/// 从请求头提取租户 ID
/// 从 Authorization header 获取 JWT token，解析 `tenant_id`
fn extract_tenant_id_from_headers(headers: &HeaderMap) -> Option<i64> {
    // 获取 Authorization header
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    // 提取 Bearer token
    let token = auth_str.strip_prefix("Bearer ")?;

    // 解析 JWT payload（base64url 编码）
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    // 解码 payload（第二部分）
    let payload = parts[1];
    let decoded = decode_base64_url(payload).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;

    // 解析 JSON 查找 tenant_id
    let value: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    value.get("tenant_id")?.as_i64()
}

/// Base64 URL 解码（简化实现）
fn decode_base64_url(input: &str) -> Result<Vec<u8>, ()> {
    // 处理 URL-safe base64
    let input = input.replace('-', "+").replace('_', "/");
    // 添加 padding
    let len = input.len();
    let padding = if len.is_multiple_of(4) {
        0
    } else {
        4 - len % 4
    };
    let padded = format!("{}{}", input, "=".repeat(padding));

    // 使用 Engine API 解码（替代已弃用的 decode 函数）
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&padded)
        .map_err(|_| ())
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub repository: TenantRepository,
}

impl AppState {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: TenantRepository::new(pool),
        }
    }
}

// ============ 请求结构 ============

/// 租户查询参数
#[derive(Debug, Deserialize)]
pub struct TenantQueryParams {
    pub keyword: Option<String>,
    pub status: Option<i32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 创建租户请求
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub code: String,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub max_users: Option<i32>,
    pub max_storage: Option<i64>,
    pub expires_at: Option<String>,
}

/// 更新租户请求
#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
    pub max_users: Option<i32>,
    pub max_storage: Option<i64>,
    pub expires_at: Option<String>,
}

/// 租户用户查询参数
#[derive(Debug, Deserialize)]
pub struct TenantUserQueryParams {
    pub role: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 添加租户用户请求
#[derive(Debug, Deserialize)]
pub struct AddTenantUserRequest {
    pub user_id: i64,
    pub role: String,
    pub department: Option<String>,
    pub position: Option<String>,
}

/// 更新租户用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateTenantUserRequest {
    pub role: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
}

/// 切换租户请求
#[derive(Debug, Deserialize)]
pub struct SwitchTenantRequest {
    pub tenant_id: i64,
}

/// 升级套餐请求
#[derive(Debug, Deserialize)]
pub struct UpgradePlanRequest {
    pub plan_id: String,
    pub interval: String,
}

// ============ 响应结构 ============

// ============ 处理器实现 ============

/// 获取当前租户信息
pub async fn get_current_tenant(State(state): State<AppState>) -> Response {
    // 默认租户 ID，后续从 JWT token 获取
    let tenant_id = 1i64;

    match state.repository.find_by_id(tenant_id).await {
        Ok(Some(tenant)) => (
            StatusCode::OK,
            Json(ApiResponse::success(serde_json::json!({
                "id": tenant.id,
                "name": tenant.name,
                "code": tenant.code,
                "domain": tenant.domain,
                "description": tenant.description,
                "max_users": tenant.max_users,
                "max_storage": tenant.max_storage,
                "status": tenant.status,
                "expires_at": tenant.expires_at.map(|t| t.to_rfc3339()),
                "created_at": tenant.created_at.to_rfc3339(),
            }))),
        )
            .into_response(),
        Ok(None) => ApiResponse::<()>::not_found_response("租户不存在"),
        Err(e) => {
            tracing::error!("获取租户信息失败: {e}");
            ApiResponse::<()>::error_response("获取租户信息失败")
        }
    }
}

/// 更新租户信息
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTenantRequest>,
) -> Response {
    use crate::repository::UpdateTenantParams;

    let params = UpdateTenantParams {
        name: req.name.clone(),
        domain: req.domain.clone(),
        description: req.description.clone(),
        status: req.status,
        max_users: req.max_users,
        max_storage: req.max_storage,
    };

    match state.repository.update(id, params).await {
        Ok(true) => ApiResponse::<()>::ok_response("租户更新成功"),
        Ok(false) => ApiResponse::<()>::not_found_response("租户不存在"),
        Err(e) => {
            tracing::error!("更新租户失败: {e}");
            ApiResponse::<()>::error_response("更新租户失败")
        }
    }
}

/// 获取租户用户列表
pub async fn list_tenant_users(
    State(state): State<AppState>,
    Path(tenant_id): Path<i64>,
    Query(params): Query<TenantUserQueryParams>,
) -> Response {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state
        .repository
        .list_users(
            tenant_id,
            page,
            page_size,
            params.role.as_deref(),
            params.keyword.as_deref(),
        )
        .await
    {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse::success(serde_json::json!({
                "list": result.users,
                "total": result.total,
                "page": page,
                "page_size": page_size
            }))),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取租户用户列表失败: {e}");
            ApiResponse::<()>::error_response("获取用户列表失败")
        }
    }
}

/// 添加租户用户
pub async fn add_tenant_user(
    State(state): State<AppState>,
    Path(tenant_id): Path<i64>,
    Json(req): Json<AddTenantUserRequest>,
) -> Response {
    match state
        .repository
        .add_user(
            tenant_id,
            req.user_id,
            &req.role,
            req.department.as_deref(),
            req.position.as_deref(),
        )
        .await
    {
        Ok(()) => (StatusCode::CREATED, Json(ApiResponse::<()>::success(()))).into_response(),
        Err(e) => {
            tracing::error!("添加租户用户失败: {e}");
            ApiResponse::<()>::error_response("添加用户失败")
        }
    }
}

/// 更新租户用户
pub async fn update_tenant_user(
    State(state): State<AppState>,
    Path((tenant_id, user_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateTenantUserRequest>,
) -> Response {
    match state
        .repository
        .update_user(
            tenant_id,
            user_id,
            req.role.as_deref(),
            req.department.as_deref(),
            req.position.as_deref(),
        )
        .await
    {
        Ok(true) => ApiResponse::<()>::ok_response("用户更新成功"),
        Ok(false) => ApiResponse::<()>::not_found_response("用户不存在"),
        Err(e) => {
            tracing::error!("更新租户用户失败: {e}");
            ApiResponse::<()>::error_response("更新用户失败")
        }
    }
}

/// 移除租户用户
pub async fn remove_tenant_user(
    State(state): State<AppState>,
    Path((tenant_id, user_id)): Path<(i64, i64)>,
) -> Response {
    match state.repository.remove_user(tenant_id, user_id).await {
        Ok(true) => ApiResponse::<()>::ok_response("用户移除成功"),
        Ok(false) => ApiResponse::<()>::not_found_response("用户不存在"),
        Err(e) => {
            tracing::error!("移除租户用户失败: {e}");
            ApiResponse::<()>::error_response("移除用户失败")
        }
    }
}

/// 切换租户
pub async fn switch_tenant(
    State(state): State<AppState>,
    Json(req): Json<SwitchTenantRequest>,
) -> Response {
    match state.repository.find_by_id(req.tenant_id).await {
        Ok(Some(tenant)) => {
            if tenant.status != 1 {
                return ApiResponse::<()>::error_response("租户已禁用");
            }
            ApiResponse::<()>::ok_response("切换成功")
        }
        Ok(None) => ApiResponse::<()>::not_found_response("租户不存在"),
        Err(e) => {
            tracing::error!("切换租户失败: {e}");
            ApiResponse::<()>::error_response("切换租户失败")
        }
    }
}

/// 获取套餐列表
pub async fn list_plans(State(_state): State<AppState>) -> Response {
    // 返回静态套餐列表，实际应从数据库或外部服务获取
    (
        StatusCode::OK,
        Json(ApiResponse::success(vec![
            serde_json::json!({
                "id": "free",
                "name": "免费版",
                "price": 0,
                "interval": "month",
                "max_users": 5,
                "max_storage": 1024 * 1024 * 1024, // 1GB
                "features": ["基础功能", "5用户"]
            }),
            serde_json::json!({
                "id": "basic",
                "name": "基础版",
                "price": 99,
                "interval": "month",
                "max_users": 50,
                "max_storage": 100 * 1024 * 1024 * 1024, // 100GB
                "features": ["全部基础功能", "50用户", "技术支持"]
            }),
            serde_json::json!({
                "id": "pro",
                "name": "专业版",
                "price": 299,
                "interval": "month",
                "max_users": 200,
                "max_storage": 500 * 1024 * 1024 * 1024, // 500GB
                "features": ["全部专业功能", "200用户", "优先支持", "API访问"]
            }),
            serde_json::json!({
                "id": "enterprise",
                "name": "企业版",
                "price": 999,
                "interval": "month",
                "max_users": -1, // 无限制
                "max_storage": -1, // 无限制
                "features": ["全部企业功能", "无限制用户", "专属支持", "私有部署"]
            }),
        ])),
    )
        .into_response()
}

/// 获取当前套餐
pub async fn get_current_plan(State(_state): State<AppState>) -> Response {
    // 返回默认套餐信息
    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "plan_id": "basic",
            "name": "基础版",
            "price": 99,
            "interval": "month",
            "max_users": 50,
            "current_users": 10,
            "max_storage": 100 * 1024 * 1024 * 1024,
            "current_storage": 5 * 1024 * 1024 * 1024,
            "expires_at": "2025-12-31T23:59:59Z"
        }))),
    )
        .into_response()
}

/// 升级/续费套餐
pub async fn upgrade_plan(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpgradePlanRequest>,
) -> Response {
    // 从请求头获取租户ID
    let tenant_id = extract_tenant_id_from_headers(&headers).unwrap_or(1);

    tracing::info!(
        "升级套餐请求: tenant_id={}, plan_id={}, interval={}",
        tenant_id,
        req.plan_id,
        req.interval
    );

    // 定义套餐价格映射
    let price = match req.plan_id.as_str() {
        "free" => 0.0,
        "basic" => 99.0,
        "pro" => 299.0,
        "enterprise" => 999.0,
        _ => return ApiResponse::<()>::error_response("无效的套餐ID"),
    };

    // 计算续费时长（月）
    let months = match req.interval.as_str() {
        "month" => 1,
        "quarter" => 3,
        "year" => 12,
        _ => return ApiResponse::<()>::error_response("无效的付费周期"),
    };

    let total_amount = (price as i64) * months;

    // 模拟支付成功（实际生产环境需要集成真实支付网关）
    tracing::info!(
        "模拟支付完成: plan_id={}, amount={}",
        req.plan_id,
        total_amount
    );

    // 返回支付结果
    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "success": true,
            "plan_id": req.plan_id,
            "interval": req.interval,
            "amount": total_amount,
            "message": "套餐升级成功"
        }))),
    )
        .into_response()
}

/// 获取使用统计
pub async fn get_usage_stats(State(state): State<AppState>) -> Response {
    let tenant_id = 1i64;

    match state.repository.get_usage_stats(tenant_id).await {
        Ok(stats) => (
            StatusCode::OK,
            Json(ApiResponse::success(serde_json::json!({
                "users": {
                    "total": stats.total_users,
                    "active": stats.active_users
                },
                "storage": {
                    "used": stats.used_storage,
                    "limit": stats.max_storage
                },
                "api_calls": {
                    "monthly": stats.monthly_api_calls,
                    "limit": stats.api_call_limit
                }
            }))),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取使用统计失败: {e}");
            ApiResponse::<()>::error_response("获取使用统计失败")
        }
    }
}

/// 获取审计日志
pub async fn get_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<TenantUserQueryParams>,
    headers: HeaderMap, // 从请求头获取 Token
) -> Response {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    // 从 Authorization header 获取 tenant_id
    let tenant_id = extract_tenant_id_from_headers(&headers).unwrap_or(1); // 默认租户 ID

    match state
        .repository
        .get_audit_logs(tenant_id, page, page_size, params.keyword.as_deref())
        .await
    {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse::success(serde_json::json!({
                "list": result.logs,
                "total": result.total,
                "page": page,
                "page_size": page_size
            }))),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取审计日志失败: {e}");
            ApiResponse::<()>::error_response("获取审计日志失败")
        }
    }
}

// ============ 路由构建 ============

/// 创建租户管理路由器
pub fn create_tenant_router(state: std::sync::Arc<AppState>) -> Router {
    Router::new()
        // 租户信息
        .route(
            "/api/tenant/current",
            axum::routing::get(get_current_tenant),
        )
        .route("/api/tenant/:id", axum::routing::put(update_tenant))
        // 租户用户管理
        .route("/api/tenant/users", axum::routing::get(list_tenant_users))
        .route("/api/tenant/users", axum::routing::post(add_tenant_user))
        .route(
            "/api/tenant/users/:user_id",
            axum::routing::put(update_tenant_user),
        )
        .route(
            "/api/tenant/users/:user_id",
            axum::routing::delete(remove_tenant_user),
        )
        // 租户切换
        .route("/api/tenant/switch", axum::routing::post(switch_tenant))
        // 套餐管理
        .route("/api/tenant/plans", axum::routing::get(list_plans))
        .route(
            "/api/tenant/plan/current",
            axum::routing::get(get_current_plan),
        )
        .route(
            "/api/tenant/plan/upgrade",
            axum::routing::post(upgrade_plan),
        )
        // 使用统计
        .route("/api/tenant/usage", axum::routing::get(get_usage_stats))
        // 审计日志
        .route("/api/tenant/audit-logs", axum::routing::get(get_audit_logs))
        .with_state((*state).clone())
}
