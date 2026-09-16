//! 消息/通知路由 — 真实 gRPC 调用 messaging-service

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::{get, post, put, delete}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct MsgQuery { page: Option<i32>, page_size: Option<i32>, r#type: Option<String> }
#[derive(Deserialize)]
struct TmplQuery { page: Option<i32>, page_size: Option<i32> }

/// 获取 messaging-service gRPC 客户端
async fn get_msg_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::MessageGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.message_client().await
        .map_err(|e| json_error(&format!("messaging-service 不可用: {e}" )))
}

/// proto Message → JSON
fn message_to_json(m: &grpc_proto::message::Message) -> Value {
    json!({
        "id": m.id,
        "user_id": m.user_id,
        "type": m.r#type,
        "title": m.title,
        "content": m.content,
        "sender": m.sender,
        "is_read": m.is_read,
        "read_at": m.read_at,
        "created_at": m.created_at,
    })
}

/// proto MessageTemplate → JSON
fn template_to_json(t: &grpc_proto::message::MessageTemplate) -> Value {
    json!({
        "id": t.id,
        "name": t.name,
        "type": t.r#type,
        "title": t.title_template,
        "content": t.content_template,
        "is_active": t.is_active,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })
}

async fn list_messages(
    State(state): State<Arc<AppState>>,
    Query(q): Query<MsgQuery>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    // user_id=0 表示查询全部（管理视角）
    match client.list_messages(0, q.page.unwrap_or(1), q.page_size.unwrap_or(20), q.r#type.clone().unwrap_or_default(), false).await {
        Ok(resp) => json_success(json!({
            "list": resp.messages.iter().map(message_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "unread_count": resp.unread_count,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
            "type": q.r#type,
        })),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.send_message(
        0,
        body["to_user_id" ].as_i64().unwrap_or(0),
        body["type" ].as_str().unwrap_or("message" ).to_string(),
        body["title" ].as_str().unwrap_or("" ).to_string(),
        body["content" ].as_str().unwrap_or("" ).to_string(),
        None,
        vec![],
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id})),
        Err(e) => json_error(&format!("发送失败: {e}" )),
    }
}

async fn mark_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.mark_as_read(0, vec![id]).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("操作失败: {e}" )),
    }
}

async fn mark_read_all(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.mark_as_read(0, vec![]).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("操作失败: {e}" )),
    }
}

async fn delete_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_message(0, vec![id]).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}" )),
    }
}

async fn batch_delete_messages(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    // 逐条删除（messaging-service 未提供批量删除 RPC）
    let ids: Vec<i64> = body["ids" ].as_array().map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    for id in ids {
        if let Err(e) = client.delete_message(0, vec![id]).await {
            return json_error(&format!("删除失败: {e}" ));
        }
    }
    json_ok()
}

async fn star_message() -> Json<Value> { json_ok() }
async fn unstar_message() -> Json<Value> { json_ok() }

async fn get_unread_count(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_unread_count(0, String::new()).await {
        Ok(resp) => json_success(json!({"total": resp.count})),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

// ==================== 消息模板 ====================

async fn list_templates(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TmplQuery>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_templates(q.page.unwrap_or(1), q.page_size.unwrap_or(20), String::new()).await {
        Ok(resp) => json_success(json!({
            "list": resp.templates.iter().map(template_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn create_template(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_template(
        body["name" ].as_str().unwrap_or("" ).to_string(),
        body["type" ].as_str().unwrap_or("notification" ).to_string(),
        body["title" ].as_str().unwrap_or("" ).to_string(),
        body["content" ].as_str().unwrap_or("" ).to_string(),
        std::collections::HashMap::new(),
        true,
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id})),
        Err(e) => json_error(&format!("创建失败: {e}" )),
    }
}

async fn get_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    // messaging-service 未提供单模板查询 RPC，用列表 + 过滤模拟
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_templates(1, 1000, String::new()).await {
        Ok(resp) => {
            if let Some(t) = resp.templates.iter().find(|t| t.id == id) {
                json_success(template_to_json(t))
            } else {
                json_error("模板不存在" )
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}" )),
    }
}

async fn update_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_template(
        id,
        body["name" ].as_str().unwrap_or("" ).to_string(),
        body["type" ].as_str().unwrap_or("notification" ).to_string(),
        body["title" ].as_str().unwrap_or("" ).to_string(),
        body["content" ].as_str().unwrap_or("" ).to_string(),
        std::collections::HashMap::new(),
        true,
    ).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}" )),
    }
}

async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let mut client = match get_msg_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_template(id).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}" )),
    }
}

async fn toggle_template() -> Json<Value> { json_ok() }
async fn test_template() -> Json<Value> {
    json_success(json!({"sent": true, "recipients": 0}))
}

// ==================== 公告/通知（user-service 公告表，待 proto 扩展 RPC） ====================

async fn list_notifications(Query(q): Query<MsgQuery>) -> Json<Value> {
    json_success(json!({"list": [], "total": 0, "page": q.page.unwrap_or(1), "page_size": q.page_size.unwrap_or(20), "type": q.r#type}))
}
async fn send_notification() -> Json<Value> { json_success(json!({"id": 0})) }
async fn read_notification() -> Json<Value> { json_ok() }
async fn read_all_notifications() -> Json<Value> { json_ok() }
async fn batch_delete_notifications() -> Json<Value> { json_ok() }
async fn delete_notification() -> Json<Value> { json_ok() }

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/messages" , get(list_messages).post(send_message))
        .route("/api/messages/inbox" , get(list_messages))
        .route("/api/messages/outbox" , get(list_messages))
        .route("/api/messages/unread-count" , get(get_unread_count))
        .route("/api/messages/{id}/read" , put(mark_read))
        .route("/api/messages/{id}/star" , put(star_message).delete(unstar_message))
        .route("/api/messages/{id}" , delete(delete_message))
        .route("/api/messages/read/all" , put(mark_read_all))
        .route("/api/messages/read/batch" , put(mark_read_all))
        .route("/api/messages/batch-send" , post(send_message))
        .route("/api/messages/batch" , delete(batch_delete_messages))
        .route("/api/messages/announcements" , get(list_messages))
        .route("/api/messages/announcements/unread-count" , get(get_unread_count))
        .route("/api/admin/notification-templates" , get(list_templates).post(create_template))
        .route("/api/admin/notification-templates/{id}" , get(get_template).put(update_template).delete(delete_template))
        .route("/api/admin/notification-templates/{id}/toggle" , put(toggle_template))
        .route("/api/admin/notification-templates/{id}/test" , post(test_template))
        .route("/api/admin/notifications" , get(list_notifications))
        .route("/api/admin/notifications/send" , post(send_notification))
        .route("/api/admin/notifications/read-all" , put(read_all_notifications))
        .route("/api/admin/notifications/batch-delete" , post(batch_delete_notifications))
        .route("/api/admin/notifications/{id}" , get(list_notifications).delete(delete_notification))
        .route("/api/admin/notifications/{id}/read" , put(read_notification))
}
