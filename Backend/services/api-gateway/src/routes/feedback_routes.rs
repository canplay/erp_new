//! 意见反馈路由 — 真实 gRPC 调用 feedback-service

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::{get, post, put}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct FbQuery { page: Option<i32>, page_size: Option<i32>, status: Option<i32>, r#type: Option<i32>, keyword: Option<String> }

/// 获取 feedback-service gRPC 客户端
async fn get_fb_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::FeedbackGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.feedback_client().await
        .map_err(|e| json_error(&format!("feedback-service 不可用: {e}" )))
}

/// proto Feedback → JSON
fn feedback_to_json(f: &grpc_proto::feedback::Feedback) -> Value {
    json!({
        "id": f.id,
        "user_id": f.user_id,
        "username": f.username,
        "type": f.r#type,
        "status": f.status,
        "priority": f.priority,
        "title": f.title,
        "content": f.content,
        "attachments": f.attachments,
        "device_info": f.device_info,
        "app_version": f.app_version,
        "assigned_to": f.assigned_to,
        "resolved_at": f.resolved_at,
        "created_at": f.created_at,
        "updated_at": f.updated_at,
    })
}

async fn list_feedback(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FbQuery>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_feedbacks(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        0,
        q.r#type.unwrap_or(-1),
        q.status.unwrap_or(-1),
        -1,
        q.keyword.clone().unwrap_or_default(),
    ).await {
        Ok(resp) => json_success(json!({
            "list": resp.feedbacks.iter().map(feedback_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
            "status": q.status,
            "type": q.r#type,
        })),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_feedback(
        0,
        String::new(),
        body["type" ].as_i64().unwrap_or(0) as i32,
        body["title" ].as_str().unwrap_or("" ).to_string(),
        body["content" ].as_str().unwrap_or("" ).to_string(),
        vec![],
        String::new(),
        String::new(),
        std::collections::HashMap::new(),
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id})),
        Err(e) => json_error(&format!("提交失败: {e}" )),
    }
}

async fn get_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_feedback(id).await {
        Ok(resp) => {
            if let Some(f) = resp.feedback {
                json_success(feedback_to_json(&f))
            } else {
                json_error("反馈不存在" )
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn handle_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_feedback(id, body["status" ].as_i64().unwrap_or(2) as i32, -1, 0, body["handler_note" ].as_str().unwrap_or("" ).to_string()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("处理失败: {e}" )),
    }
}

async fn close_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_feedback(id, 3, -1, 0, String::new()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("关闭失败: {e}" )),
    }
}

async fn transfer_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_feedback(id, 1, -1, 0, body["note" ].as_str().unwrap_or("" ).to_string()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("转交失败: {e}" )),
    }
}

async fn add_reply(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.add_reply(id, 0, String::new(), body["content" ].as_str().unwrap_or("" ).to_string(), true).await {
        Ok(resp) => json_success(json!({"id": resp.id})),
        Err(e) => json_error(&format!("回复失败: {e}" )),
    }
}

async fn list_replies(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_replies(id).await {
        Ok(resp) => json_success(json!({
            "list": resp.replies.iter().map(|r| json!({
                "id": r.id,
                "feedback_id": r.feedback_id,
                "user_id": r.user_id,
                "content": r.content,
                "created_at": r.created_at,
            })).collect::<Vec<_>>(),
            "total": resp.replies.len(),
        })),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn get_fb_stats(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_feedback_stats(0, 0).await {
        Ok(resp) => {
            if let Some(s) = resp.stats {
                json_success(json!({
                    "total": s.total_count,
                    "pending": s.open_count + s.in_progress_count,
                    "resolved": s.resolved_count,
                    "closed": s.closed_count,
                }))
            } else {
                json_success(json!({"total": 0, "pending": 0, "resolved": 0, "closed": 0}))
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn batch_handle(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let ids: Vec<i64> = body["ids" ].as_array().map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
    let status = body["status" ].as_i64().unwrap_or(2) as i32;
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    for id in ids {
        if let Err(e) = client.update_feedback(id, status, -1, 0, String::new()).await {
            return json_error(&format!("批量处理失败: {e}" ));
        }
    }
    json_ok()
}

async fn fb_stats_by_type(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_fb_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_feedback_stats(0, 0).await {
        Ok(resp) => {
            let total = resp.stats.as_ref().map(|s| s.total_count).unwrap_or(0);
            json_success(json!({
                "list": [{"type": "all" , "total": total}],
                "total": total,
            }))
        }
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/feedback" , get(list_feedback))
        .route("/api/admin/feedback/{id}" , get(get_feedback))
        .route("/api/admin/feedback/{id}/reply" , post(add_reply))
        .route("/api/admin/feedback/{id}/replies" , get(list_replies))
        .route("/api/admin/feedback/{id}/handle" , put(handle_feedback))
        .route("/api/admin/feedback/{id}/close" , put(close_feedback))
        .route("/api/admin/feedback/{id}/transfer" , put(transfer_feedback))
        .route("/api/admin/feedback/batch-handle" , put(batch_handle))
        .route("/api/admin/feedback/statistics" , get(get_fb_stats))
        .route("/api/admin/feedback/statistics/by-type" , get(fb_stats_by_type))
        .route("/api/admin/feedback/handlers" , get(list_feedback))
        .route("/api/feedback" , post(submit_feedback))
}
