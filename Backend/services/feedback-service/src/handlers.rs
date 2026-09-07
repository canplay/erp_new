//! 反馈服务 HTTP 处理器
//!
//! 提供反馈管理的 HTTP REST API

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use auth_core::middleware::AuthenticatedUser;

use crate::repository::{FeedbackQueryParams, FeedbackRepository};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub repository: FeedbackRepository,
}

impl AppState {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: FeedbackRepository::new(pool),
        }
    }
}

// ============ 请求结构 ============

/// 反馈查询参数
#[derive(Debug, Deserialize)]
pub struct FeedbackQuery {
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
    pub handler_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 创建反馈请求（用户提交）
#[derive(Debug, Deserialize)]
pub struct CreateFeedbackRequest {
    pub r#type: String,
    pub title: String,
    pub content: String,
    pub contact: Option<String>,
}

/// 处理反馈请求
#[derive(Debug, Deserialize)]
pub struct HandleFeedbackRequest {
    pub status: String,
    pub handler_reply: String,
}

/// 转交反馈请求
#[derive(Debug, Deserialize)]
pub struct TransferFeedbackRequest {
    pub handler_id: i64,
}

/// 回复反馈请求
#[derive(Debug, Deserialize)]
pub struct ReplyFeedbackRequest {
    pub reply: String,
}

/// 批量处理请求
#[derive(Debug, Deserialize)]
pub struct BatchHandleRequest {
    pub ids: Vec<i64>,
    pub status: String,
    pub handler_reply: Option<String>,
}

// ============ 响应结构 ============

/// 统一响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub const fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }
}

impl ApiResponse<()> {
    #[must_use]
    pub fn error(msg: &str) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Self {
                success: false,
                data: None,
                message: None,
                error: Some(msg.to_string()),
            }),
        )
            .into_response()
    }

    #[must_use]
    pub fn not_found(msg: &str) -> Response {
        (
            StatusCode::NOT_FOUND,
            Json(Self {
                success: false,
                data: None,
                message: None,
                error: Some(msg.to_string()),
            }),
        )
            .into_response()
    }

    #[must_use]
    pub fn ok(msg: &str) -> Response {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                data: None,
                message: Some(msg.to_string()),
                error: None,
            }),
        )
            .into_response()
    }
}

// ============ 管理员 API ============

/// 获取反馈列表
pub async fn list_feedback(
    State(state): State<AppState>,
    Query(params): Query<FeedbackQuery>,
) -> Response {
    let query_params = FeedbackQueryParams {
        r#type: params.r#type,
        status: params.status,
        keyword: params.keyword,
        handler_id: params.handler_id,
        start_date: params.start_date,
        end_date: params.end_date,
        page: params.page,
        page_size: params.page_size,
    };

    match state.repository.list(&query_params).await {
        Ok(result) => (StatusCode::OK, Json(ApiResponse::success(result))).into_response(),
        Err(e) => {
            tracing::error!("获取反馈列表失败: {e}");
            ApiResponse::<()>::error("获取列表失败")
        }
    }
}

/// 获取反馈详情
pub async fn get_feedback_detail(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match state.repository.find_by_id(id).await {
        Ok(Some(feedback)) => {
            (StatusCode::OK, Json(ApiResponse::success(feedback))).into_response()
        }
        Ok(None) => ApiResponse::<()>::not_found("反馈不存在"),
        Err(e) => {
            tracing::error!("获取反馈详情失败: {e}");
            ApiResponse::<()>::error("获取详情失败")
        }
    }
}

/// 处理反馈
pub async fn handle_feedback(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<HandleFeedbackRequest>,
) -> Response {
    match state
        .repository
        .handle(id, &req.status, &req.handler_reply)
        .await
    {
        Ok(true) => ApiResponse::<()>::ok("处理成功"),
        Ok(false) => ApiResponse::<()>::not_found("反馈不存在"),
        Err(e) => {
            tracing::error!("处理反馈失败: {e}");
            ApiResponse::<()>::error("处理失败")
        }
    }
}

/// 转交反馈
pub async fn transfer_feedback(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<TransferFeedbackRequest>,
) -> Response {
    match state.repository.transfer(id, req.handler_id).await {
        Ok(true) => ApiResponse::<()>::ok("转交成功"),
        Ok(false) => ApiResponse::<()>::not_found("反馈不存在"),
        Err(e) => {
            tracing::error!("转交反馈失败: {e}");
            ApiResponse::<()>::error("转交失败")
        }
    }
}

/// 回复反馈
pub async fn reply_feedback(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<ReplyFeedbackRequest>,
) -> Response {
    match state.repository.add_reply(id, &req.reply).await {
        Ok(()) => ApiResponse::<()>::ok("回复成功"),
        Err(e) => {
            tracing::error!("回复反馈失败: {e}");
            ApiResponse::<()>::error("回复失败")
        }
    }
}

/// 关闭反馈
pub async fn close_feedback(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match state.repository.handle(id, "closed", "").await {
        Ok(true) => ApiResponse::<()>::ok("已关闭"),
        Ok(false) => ApiResponse::<()>::not_found("反馈不存在"),
        Err(e) => {
            tracing::error!("关闭反馈失败: {e}");
            ApiResponse::<()>::error("关闭失败")
        }
    }
}

/// 删除反馈
pub async fn delete_feedback(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match state.repository.delete(id).await {
        Ok(true) => ApiResponse::<()>::ok("删除成功"),
        Ok(false) => ApiResponse::<()>::not_found("反馈不存在"),
        Err(e) => {
            tracing::error!("删除反馈失败: {e}");
            ApiResponse::<()>::error("删除失败")
        }
    }
}

/// 批量处理反馈
pub async fn batch_handle_feedback(
    State(state): State<AppState>,
    Json(req): Json<BatchHandleRequest>,
) -> Response {
    match state
        .repository
        .batch_handle(&req.ids, &req.status, req.handler_reply.as_deref())
        .await
    {
        Ok(count) => ApiResponse::<()>::ok(&format!("已处理 {count} 条反馈")),
        Err(e) => {
            tracing::error!("批量处理失败: {e}");
            ApiResponse::<()>::error("批量处理失败")
        }
    }
}

/// 获取反馈统计
pub async fn get_feedback_statistics(
    State(state): State<AppState>,
    Query(params): Query<FeedbackQuery>,
) -> Response {
    let query_params = FeedbackQueryParams {
        r#type: None,
        status: None,
        keyword: None,
        handler_id: None,
        start_date: params.start_date.clone(),
        end_date: params.end_date.clone(),
        page: None,
        page_size: None,
    };

    match state.repository.get_statistics(&query_params).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))).into_response(),
        Err(e) => {
            tracing::error!("获取统计失败: {e}");
            ApiResponse::<()>::error("获取统计失败")
        }
    }
}

/// 获取反馈类型统计
pub async fn get_feedback_type_statistics(
    State(state): State<AppState>,
    Query(params): Query<FeedbackQuery>,
) -> Response {
    let query_params = FeedbackQueryParams {
        r#type: None,
        status: None,
        keyword: None,
        handler_id: None,
        start_date: params.start_date.clone(),
        end_date: params.end_date.clone(),
        page: None,
        page_size: None,
    };

    match state.repository.get_type_statistics(&query_params).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))).into_response(),
        Err(e) => {
            tracing::error!("获取类型统计失败: {e}");
            ApiResponse::<()>::error("获取统计失败")
        }
    }
}

/// 获取处理人列表
pub async fn get_feedback_handlers(State(state): State<AppState>) -> Response {
    match state.repository.get_handlers().await {
        Ok(handlers) => (StatusCode::OK, Json(ApiResponse::success(handlers))).into_response(),
        Err(e) => {
            tracing::error!("获取处理人列表失败: {e}");
            ApiResponse::<()>::error("获取失败")
        }
    }
}

// ============ 用户端 API ============

/// 提交反馈
pub async fn submit_feedback(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateFeedbackRequest>,
) -> Response {
    match state
        .repository
        .create(
            user.user_id,
            &req.r#type,
            &req.title,
            &req.content,
            req.contact.as_deref(),
        )
        .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(serde_json::json!({ "id": id }))),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("提交反馈失败: {e}");
            ApiResponse::<()>::error("提交失败")
        }
    }
}

// ============ 路由构建 ============

/// 创建反馈管理路由器
pub fn create_feedback_router(state: std::sync::Arc<AppState>) -> Router {
    Router::new()
        // 管理员 API
        .route("/api/admin/feedback", axum::routing::get(list_feedback))
        .route(
            "/api/admin/feedback/:id",
            axum::routing::get(get_feedback_detail),
        )
        .route(
            "/api/admin/feedback/:id/handle",
            axum::routing::put(handle_feedback),
        )
        .route(
            "/api/admin/feedback/:id/transfer",
            axum::routing::put(transfer_feedback),
        )
        .route(
            "/api/admin/feedback/:id/reply",
            axum::routing::post(reply_feedback),
        )
        .route(
            "/api/admin/feedback/:id/close",
            axum::routing::put(close_feedback),
        )
        .route(
            "/api/admin/feedback/:id",
            axum::routing::delete(delete_feedback),
        )
        .route(
            "/api/admin/feedback/batch-handle",
            axum::routing::put(batch_handle_feedback),
        )
        .route(
            "/api/admin/feedback/statistics",
            axum::routing::get(get_feedback_statistics),
        )
        .route(
            "/api/admin/feedback/statistics/by-type",
            axum::routing::get(get_feedback_type_statistics),
        )
        .route(
            "/api/admin/feedback/handlers",
            axum::routing::get(get_feedback_handlers),
        )
        // 用户端 API
        .route("/api/feedback", axum::routing::post(submit_feedback))
        .with_state((*state).clone())
}
