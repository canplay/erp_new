//! 定时任务路由

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json,
};
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;
use crate::repository::ScheduledTaskEntry as RepoScheduledTaskEntry;

use crate::routes::helpers::*;

fn now_str() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

macro_rules! task_read_store {
    ($state:expr, $field:ident) => {{
        $state.$field.read().await
    }};
}

macro_rules! task_write_store {
    ($state:expr, $field:ident) => {{
        $state.$field.write().await
    }};
}

pub async fn list_scheduled_tasks(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = task_read_store!(state, scheduled_task_store);
    let items: Vec<RepoScheduledTaskEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |i| i.name.clone());
    let items: Vec<RepoScheduledTaskEntry> = match &q.status {
        Some(s) if s == "enabled" => items.into_iter().filter(|i| i.is_active).collect(),
        Some(s) if s == "disabled" => items.into_iter().filter(|i| !i.is_active).collect(),
        _ => items,
    };
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    json_success(json!({"list": list, "total": total}))
}

pub async fn create_scheduled_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    let new_id = store.entries().iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let now = now_str();
    let task = RepoScheduledTaskEntry {
        id: new_id,
        name: body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        cron: body.get("cron_expr").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        handler: body.get("handler").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        is_active: body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
        created_at: now.clone(),
    };
    store.entries_mut().push(task.clone());
    json_success(json!(task))
}

async fn get_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = task_read_store!(state, scheduled_task_store);
    match store.entries().iter().find(|t| t.id == id) {
        Some(task) => json_success(json!(task)),
        None => json_error("任务不存在"),
    }
}

pub async fn update_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    if let Some(task) = store.entries_mut().iter_mut().find(|t| t.id == id) {
        if let Some(name) = body.get("name").and_then(|v| v.as_str()) { task.name = name.to_string(); }
        if let Some(expr) = body.get("cron_expr").and_then(|v| v.as_str()) { task.cron = expr.to_string(); }
        if let Some(_secs) = body.get("interval_secs").and_then(|v| v.as_i64()) { /* interval_secs not in RepoScheduledTaskEntry */ }
        if let Some(handler) = body.get("handler").and_then(|v| v.as_str()) { task.handler = handler.to_string(); }
        task.created_at = now_str();
        json_success(json!(task))
    } else {
        json_error("任务不存在")
    }
}

pub async fn delete_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    let len = store.entries().len();
    store.entries_mut().retain(|t| t.id != id);
    if store.entries().len() < len {
        json_success(json!({"deleted": true}))
    } else {
        json_error("任务不存在")
    }
}

async fn enable_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = true; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在"),
    }
}

async fn disable_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = false; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在"),
    }
}

pub async fn trigger_scheduled_task(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i64>,
) -> Json<Value> {
    json_ok()
}

async fn pause_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = false; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在"),
    }
}

async fn resume_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = task_write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = true; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在"),
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/scheduled-tasks", get(list_scheduled_tasks).post(create_scheduled_task))
        .route("/api/scheduled-tasks/{id}", get(get_scheduled_task).put(update_scheduled_task).delete(delete_scheduled_task))
        .route("/api/scheduled-tasks/{id}/enable", put(enable_scheduled_task))
        .route("/api/scheduled-tasks/{id}/disable", put(disable_scheduled_task))
        .route("/api/scheduled-tasks/{id}/trigger", post(trigger_scheduled_task))
        .route("/api/scheduled-tasks/{id}/pause", post(pause_scheduled_task))
        .route("/api/scheduled-tasks/{id}/resume", post(resume_scheduled_task))
}
