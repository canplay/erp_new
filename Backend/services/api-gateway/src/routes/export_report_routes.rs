//! 导出任务 + 报表路由

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post},
    Json,
};
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;
use crate::repository::ReportEntry as RepoReportEntry;
use crate::repository::DataSourceEntry as RepoDataSourceEntry;
use crate::repository::ReportTemplateEntry as RepoReportTemplateEntry;

use crate::routes::helpers::*;
use super::scheduled_task_routes::{create_scheduled_task, update_scheduled_task, delete_scheduled_task, trigger_scheduled_task};

// ==================== 导出任务 ====================

async fn list_export_tasks() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn create_export_task() -> Json<Value> {
    json_success(json!({"id": 0}))
}
async fn get_export_task(axum::extract::Path(id): axum::extract::Path<i64>) -> Json<Value> {
    json_success(json!({"id": id, "status": "pending", "progress": 0}))
}
async fn cancel_export_task(axum::extract::Path(_id): axum::extract::Path<i64>) -> Json<Value> { json_ok() }
async fn retry_export_task(axum::extract::Path(_id): axum::extract::Path<i64>) -> Json<Value> { json_ok() }
async fn export_task_progress(axum::extract::Path(id): axum::extract::Path<i64>) -> Json<Value> {
    json_success(json!({"id": id, "progress": 0, "status": "pending"}))
}
async fn download_export_task(axum::extract::Path(_id): axum::extract::Path<i64>) -> Json<Value> {
    json_success(json!({"url": " "}))
}
async fn batch_create_export_task() -> Json<Value> {
    json_success(json!({"ids": []}))
}
async fn cleanup_export_tasks() -> Json<Value> { json_ok() }

async fn count_export_tasks_handler() -> Json<Value> {
    json_success(json!({"count": 0}))
}

async fn export_tasks_stats_handler() -> Json<Value> {
    json_success(json!({"total": 0, "pending": 0, "running": 0, "completed": 0, "failed": 0}))
}

async fn export_users_file() -> Json<Value> {
    json_error("导出待实现")
}
async fn export_login_logs_file() -> Json<Value> {
    json_error("导出待实现")
}
async fn export_operation_logs_file() -> Json<Value> {
    json_error("导出待实现")
}
async fn export_audit_logs_file() -> Json<Value> {
    json_error("导出待实现")
}

// ==================== 报表 ====================

fn now_str() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

macro_rules! report_read_store {
    ($state:expr, $field:ident) => {{
        $state.$field.read().await
    }};
}

macro_rules! report_write_store {
    ($state:expr, $field:ident) => {{
        $state.$field.write().await
    }};
}

async fn list_reports(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = report_read_store!(state, report_store);
    let items: Vec<RepoReportEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |r| r.name.clone());
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    let list: Vec<Value> = list.iter().map(|r: &RepoReportEntry| {
        let config = r.params.clone().unwrap_or_default();
        json!({
            "id": r.id,
            "name": r.name,
            "reportType": r.template_id.map(|t| t.to_string()).unwrap_or_default(),
            "description": Value::Null,
            "queryParams": serde_json::from_str(&config).unwrap_or_else(|_| json!({})),
            "status": if r.is_active { "active" } else { "inactive" },
            "created_by": "admin",
            "created_at": &r.created_at,
            "updated_at": &r.created_at,
        })
    }).collect();
    json_success(json!({"list": list, "total": total}))
}

async fn list_data_sources(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = report_read_store!(state, data_source_store);
    let items: Vec<RepoDataSourceEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |d| d.name.clone());
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    json_success(json!({"list": list, "total": total}))
}

async fn list_report_templates(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = report_read_store!(state, report_template_store);
    let items: Vec<RepoReportTemplateEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |t| t.name.clone());
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    let list: Vec<Value> = list.iter().map(|t: &RepoReportTemplateEntry| {
        json!({
            "id": t.id,
            "name": t.name,
            "templateType": Value::Null,
            "description": Value::Null,
            "config": Value::Null,
            "created_at": &t.created_at,
            "updated_at": &t.created_at,
        })
    }).collect();
    json_success(json!({"list": list, "total": total}))
}

async fn create_report_from_template(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(template_id): axum::extract::Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name_display = name.clone();
    let template = {
        let store = report_read_store!(state, report_template_store);
        store.entries().iter().find(|t| t.id == template_id).cloned()
    };
    let template = match template {
        Some(t) => t,
        None => return json_error("模板不存在"),
    };
    let config = template.config.clone().unwrap_or_default();
    let report_store = report_write_store!(state, report_store);
    let new_id = report_store.entries().iter().map(|r| r.id).max().unwrap_or(0) + 1;
    let now = now_str();
    report_store.entries_mut().push(RepoReportEntry {
        id: new_id,
        name: if name.is_empty() { template.name.clone() } else { name },
        template_id: Some(template.id),
        params: Some(config.to_string()),
        is_active: true,
        created_at: now.clone(),
    });
    json_success(json!({"id": new_id, "name": name_display}))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 导出任务
        .route("/api/export/tasks", get(list_export_tasks).post(create_export_task))
        .route("/api/export/tasks/create", post(create_export_task))
        .route("/api/export/tasks/list", get(list_export_tasks))
        .route("/api/export/tasks/count", get(count_export_tasks_handler))
        .route("/api/export/tasks/stats", get(export_tasks_stats_handler))
        .route("/api/export/tasks/batch-create", post(batch_create_export_task))
        .route("/api/export/tasks/cleanup", post(cleanup_export_tasks))
        .route("/api/export/tasks/{id}", get(get_export_task))
        .route("/api/export/tasks/{id}/cancel", post(cancel_export_task))
        .route("/api/export/tasks/{id}/retry", post(retry_export_task))
        .route("/api/export/tasks/{id}/progress", get(export_task_progress))
        .route("/api/export/tasks/{id}/download", get(download_export_task))
        .route("/api/export/users", get(export_users_file))
        .route("/api/export/login-logs", get(export_login_logs_file))
        .route("/api/export/operation-logs", get(export_operation_logs_file))
        .route("/api/export/audit-logs", get(export_audit_logs_file))
        // 报表
        .route("/api/reports", get(list_reports).post(create_scheduled_task))
        .route("/api/data-sources", get(list_data_sources))
        .route("/api/report-templates", get(list_report_templates))
        .route("/api/report-templates/{template_id}/create", post(create_report_from_template))
        .route("/api/reports/{id}", get(get_export_task).put(update_scheduled_task).delete(delete_scheduled_task))
        .route("/api/reports/generate/{id}", post(trigger_scheduled_task))
        .route("/api/reports/download/{id}", get(download_export_task))
        .route("/api/reports/execute/{id}", post(trigger_scheduled_task))
        .route("/api/reports/export/{id}", post(trigger_scheduled_task))
        .route("/api/reports/data/{id}", get(list_reports))
        .route("/api/reports/history/{id}", get(list_reports))
}
