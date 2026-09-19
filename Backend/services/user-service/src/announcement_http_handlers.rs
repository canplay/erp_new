//! 公告、系统配置和日志 HTTP Handlers

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::http_handlers::HttpAppState;
use crate::helpers::{json_success, json_ok, json_error, json_error_fmt, json_success_msg, json_ok_msg, json_error_msg, json_error_msg_fmt};

// ============ 请求/响应结构 ============

/// 公告查询参数
#[derive(Debug, Deserialize)]
pub(crate) struct AnnouncementQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub is_active: Option<bool>,
}

/// 创建公告请求
#[derive(Debug, Deserialize)]
pub(crate) struct CreateAnnouncementRequest {
    pub title: String,
    pub content: String,
    pub announcement_type: Option<String>,
    pub priority: Option<i32>,
    pub is_pinned: Option<bool>,
    pub is_active: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// 更新公告请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateAnnouncementRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub announcement_type: Option<String>,
    pub priority: Option<i32>,
    pub is_pinned: Option<bool>,
    pub is_active: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// 更新配置请求
#[derive(Debug, Deserialize)]
pub(crate) struct UpdateConfigRequest {
    pub value: String,
}

/// 登录日志查询参数
#[derive(Debug, Deserialize)]
pub(crate) struct LoginLogQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub user_id: Option<i64>,
    pub success: Option<bool>,
}

/// 公告响应
#[derive(Debug, Serialize)]
pub(crate) struct AnnouncementResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub announcement_type: String,
    pub priority: i32,
    pub is_pinned: bool,
    pub is_active: bool,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 公告列表响应
#[derive(Debug, Serialize)]
pub(crate) struct AnnouncementListResponse {
    pub list: Vec<AnnouncementResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 系统配置响应
#[derive(Debug, Serialize)]
pub(crate) struct ConfigResponse {
    pub id: i64,
    pub category: String,
    pub config_key: String,
    pub config_value: Option<String>,
    pub value_type: String,
    pub label: String,
    pub description: Option<String>,
}

/// 登录日志响应
#[derive(Debug, Serialize)]
pub(crate) struct LoginLogResponse {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub login_status: i32,
    pub login_status_text: String,
    pub fail_reason: Option<String>,
    pub created_at: String,
}

/// 登录日志列表响应
#[derive(Debug, Serialize)]
pub(crate) struct LoginLogListResponse {
    pub list: Vec<LoginLogResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 解析时间字符串为 `DateTime`
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

// ============ 处理器实现 ============

/// 获取公告列表
pub(crate) async fn list_announcements(
    State(state): State<HttpAppState>,
    Query(params): Query<AnnouncementQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state
        .announcement_repository
        .list(page, page_size, params.is_active)
        .await
    {
        Ok(result) => {
            let announcements: Vec<AnnouncementResponse> = result
                .announcements
                .into_iter()
                .map(|a| {
                    AnnouncementResponse {
                        id: a.id,
                        title: a.title,
                        content: String::new(), // 列表不返回内容
                        announcement_type: a.announcement_type,
                        priority: a.priority,
                        is_pinned: a.is_pinned,
                        is_active: a.is_active,
                        start_time: a.start_time.map(|t| t.to_rfc3339()),
                        end_time: a.end_time.map(|t| t.to_rfc3339()),
                        created_by: None,
                        created_by_name: a.created_by_name,
                        created_at: a.created_at.to_rfc3339(),
                        updated_at: String::new(),
                    }
                })
                .collect();

            let data = serde_json::json!({
                "list": announcements,
                "total": result.total,
                "page": page,
                "page_size": page_size
            });
            (StatusCode::OK, json_success(data)).into_response()
        }
        Err(e) => {
            tracing::error!("查询公告列表失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询公告列表失败" ),
            )
                .into_response()
        }
    }
}

/// 获取活跃公告（公开接口）
pub(crate) async fn get_active_announcements(State(state): State<HttpAppState>) -> impl IntoResponse {
    match state.announcement_repository.get_active().await {
        Ok(announcements) => {
            let result: Vec<_> = announcements
                .into_iter()
                .map(|a| {
                    serde_json::json!({
                        "id": a.id,
                        "title": a.title,
                        "announcement_type": a.announcement_type,
                        "priority": a.priority,
                        "is_pinned": a.is_pinned,
                        "created_at": a.created_at.to_rfc3339(),
                    })
                })
                .collect();

            (
                StatusCode::OK,
                json_success(serde_json::json!(result)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("获取活跃公告失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("获取活跃公告失败" ),
            )
                .into_response()
        }
    }
}

/// 获取公告详情
pub(crate) async fn get_announcement(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.announcement_repository.find_by_id(id).await {
        Ok(Some(ann)) => {
            let data = serde_json::json!({
                "id": ann.id,
                "title": ann.title,
                "content": ann.content,
                "announcement_type": ann.announcement_type,
                "priority": ann.priority,
                "is_pinned": ann.is_pinned,
                "is_active": ann.is_active,
                "start_time": ann.start_time.map(|t| t.to_rfc3339()),
                "end_time": ann.end_time.map(|t| t.to_rfc3339()),
                "created_by": ann.created_by,
                "created_by_name": ann.created_by_name,
                "created_at": ann.created_at.to_rfc3339(),
                "updated_at": ann.updated_at.to_rfc3339(),
            });
            (StatusCode::OK, json_success(data)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("公告不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询公告失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询公告失败" ),
            )
                .into_response()
        }
    }
}

/// 创建公告
pub(crate) async fn create_announcement(
    State(state): State<HttpAppState>,
    Json(req): Json<CreateAnnouncementRequest>,
) -> impl IntoResponse {
    if req.title.is_empty() || req.content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json_error("标题和内容不能为空" ),
        )
            .into_response();
    }

    let start_time = req.start_time.as_ref().and_then(|s| parse_datetime(s));
    let end_time = req.end_time.as_ref().and_then(|s| parse_datetime(s));

    match state
        .announcement_repository
        .create(
            &req.title,
            &req.content,
            req.announcement_type.as_deref().unwrap_or("normal" ),
            req.priority.unwrap_or(0),
            req.is_pinned.unwrap_or(false),
            req.is_active.unwrap_or(true),
            start_time,
            end_time,
            None, // created_by 从 JWT token 获取，暂不处理
        )
        .await
    {
        Ok(id) => {
            let data = serde_json::json!({
                "id": id,
                "title": req.title
            });
            (StatusCode::CREATED, json_success(data)).into_response()
        }
        Err(e) => {
            tracing::error!("创建公告失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("创建公告失败" ),
            )
                .into_response()
        }
    }
}

/// 更新公告
pub(crate) async fn update_announcement(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAnnouncementRequest>,
) -> impl IntoResponse {
    let start_time = req.start_time.as_ref().and_then(|s| parse_datetime(s));
    let end_time = req.end_time.as_ref().and_then(|s| parse_datetime(s));

    match state
        .announcement_repository
        .update(
            id,
            req.title,
            req.content,
            req.announcement_type,
            req.priority,
            req.is_pinned,
            req.is_active,
            start_time,
            end_time,
        )
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("公告更新成功" ),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("公告不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新公告失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新公告失败" ),
            )
                .into_response()
        }
    }
}

/// 删除公告
pub(crate) async fn delete_announcement(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.announcement_repository.delete(id).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("公告删除成功" ),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("公告不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("删除公告失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("删除公告失败" ),
            )
                .into_response()
        }
    }
}

// ============ 系统配置 ============

/// 获取所有配置
pub(crate) async fn list_configs(State(state): State<HttpAppState>) -> impl IntoResponse {
    match state.announcement_repository.get_all_configs().await {
        Ok(configs) => {
            let result: Vec<_> = configs
                .into_iter()
                .map(|c| ConfigResponse {
                    id: c.id,
                    category: c.category,
                    config_key: c.config_key,
                    config_value: c.config_value,
                    value_type: c.value_type,
                    label: c.label,
                    description: c.description,
                })
                .collect();

            (
                StatusCode::OK,
                json_success(serde_json::json!(result)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("获取配置失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("获取配置失败" ),
            )
                .into_response()
        }
    }
}

/// 获取单个配置
pub(crate) async fn get_config(
    State(state): State<HttpAppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match state.announcement_repository.get_config(&key).await {
        Ok(Some(config)) => {
            let data = serde_json::json!({
                "id": config.id,
                "category": config.category,
                "config_key": config.config_key,
                "config_value": config.config_value,
                "value_type": config.value_type,
                "label": config.label,
                "description": config.description,
            });
            (StatusCode::OK, json_success(data)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("配置不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取配置失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("获取配置失败" ),
            )
                .into_response()
        }
    }
}

/// 更新配置
pub(crate) async fn update_config(
    State(state): State<HttpAppState>,
    Path(key): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    match state
        .announcement_repository
        .update_config(&key, &req.value)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("配置更新成功" ),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("配置不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新配置失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新配置失败" ),
            )
                .into_response()
        }
    }
}

/// 重置配置
pub(crate) async fn reset_config(
    State(state): State<HttpAppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match state.announcement_repository.reset_config(&key).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("配置重置成功" ),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("配置不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("重置配置失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("重置配置失败" ),
            )
                .into_response()
        }
    }
}

// ============ 登录日志 ============

/// 获取登录日志列表
pub(crate) async fn list_login_logs(
    State(state): State<HttpAppState>,
    Query(params): Query<LoginLogQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state
        .announcement_repository
        .list_login_logs(page, page_size, params.user_id, params.success)
        .await
    {
        Ok(result) => {
            let logs: Vec<LoginLogResponse> = result
                .logs
                .into_iter()
                .map(|log| LoginLogResponse {
                    id: log.id,
                    user_id: log.user_id,
                    username: log.username,
                    ip_address: log.ip_address,
                    user_agent: log.user_agent,
                    login_status: log.login_status,
                    login_status_text: if log.login_status == 1 {
                        "成功".to_string()
                    } else {
                        "失败".to_string()
                    },
                    fail_reason: log.fail_reason,
                    created_at: log.created_at.to_rfc3339(),
                })
                .collect();

            let data = serde_json::json!({
                "list": logs,
                "total": result.total,
                "page": page,
                "page_size": page_size
            });
            (StatusCode::OK, json_success(data)).into_response()
        }
        Err(e) => {
            tracing::error!("查询登录日志失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询登录日志失败" ),
            )
                .into_response()
        }
    }
}

// ============ 路由构建 ============

/// 创建公告管理路由
pub(crate) fn create_announcement_router(state: HttpAppState) -> Router {
    Router::new()
        // 公告路由
        .route("/announcements" , axum::routing::get(list_announcements))
        .route(
            "/announcements/active" ,
            axum::routing::get(get_active_announcements),
        )
        .route("/announcements" , axum::routing::post(create_announcement))
        .route("/announcements/:id" , axum::routing::get(get_announcement))
        .route(
            "/announcements/:id" ,
            axum::routing::put(update_announcement),
        )
        .route(
            "/announcements/:id" ,
            axum::routing::delete(delete_announcement),
        )
        // 配置路由
        .route("/configs" , axum::routing::get(list_configs))
        .route("/configs/:key" , axum::routing::get(get_config))
        .route("/configs/:key" , axum::routing::put(update_config))
        .route("/configs/:key/reset" , axum::routing::post(reset_config))
        // 登录日志路由
        .route("/login-logs" , axum::routing::get(list_login_logs))
        .with_state(state)
}
