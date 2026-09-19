//! 安全路由 — IP 白名单 + 敏感审计
//!
//! 合并自 security_ip_routes.rs + security_audit_routes.rs

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;
use crate::repository::{IpWhitelistEntry, SensitiveAuditEntry};

#[derive(Debug, Deserialize)]
pub struct SecurityListQuery {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

fn now_str() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ==================== IP 白名单 ====================

macro_rules! ip_read_store {
    ($state:expr, $field:ident) => {{
        $state.$field.read().await
    }};
}

macro_rules! ip_write_store {
    ($state:expr, $field:ident) => {{
        $state.$field.write().await
    }};
}

async fn list_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SecurityListQuery>,
) -> Json<Value> {
    let store = ip_read_store!(state, ip_whitelist_store);
    let items: Vec<IpWhitelistEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |i| i.ip.clone());
    let items: Vec<IpWhitelistEntry> = match &q.status {
        Some(s) if s == "enabled" => items.into_iter().filter(|i| i.is_active).collect(),
        Some(s) if s == "disabled" => items.into_iter().filter(|i| !i.is_active).collect(),
        _ => items,
    };
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    json_success(json!({"list": list, "total": total}))
}

async fn create_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = ip_write_store!(state, ip_whitelist_store);
    let new_id = store.entries().iter().map(|r| r.id).max().unwrap_or(0) + 1;
    let now = now_str();
    let entry = IpWhitelistEntry {
        id: new_id,
        ip: body.get("ip_address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        description: body.get("description").and_then(|v| v.as_str()).map(String::from),
        is_active: true,
        created_at: now.clone(),
    };
    store.entries_mut().push(entry.clone());
    json_success(json!(entry))
}

async fn get_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = ip_read_store!(state, ip_whitelist_store);
    match store.entries().iter().find(|r| r.id == id) {
        Some(entry) => json_success(json!(entry)),
        None => json_error("记录不存在"),
    }
}

async fn update_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = ip_write_store!(state, ip_whitelist_store);
    match store.entries_mut().iter_mut().find(|r| r.id == id) {
        Some(entry) => {
            if let Some(ip) = body.get("ip_address").and_then(|v| v.as_str()) {
                entry.ip = ip.to_string();
            }
            if let Some(desc) = body.get("description") {
                entry.description = desc.as_str().map(String::from);
            }
            if let Some(active) = body.get("is_active").and_then(|v| v.as_bool()) {
                entry.is_active = active;
            }
            json_success(json!(entry))
        }
        None => json_error("记录不存在"),
    }
}

async fn delete_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = ip_write_store!(state, ip_whitelist_store);
    let len_before = store.entries().len();
    store.entries_mut().retain(|r| r.id != id);
    if store.entries().len() < len_before {
        json_success(json!({"deleted": true}))
    } else {
        json_error("记录不存在")
    }
}

// ==================== 敏感审计 ====================

macro_rules! audit_read_store {
    ($state:expr, $field:ident) => {{
        $state.$field.read().await
    }};
}

macro_rules! audit_write_store {
    ($state:expr, $field:ident) => {{
        $state.$field.write().await
    }};
}

async fn list_sensitive_audits(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SecurityListQuery>,
) -> Json<Value> {
    let store = audit_read_store!(state, sensitive_audit_store);
    let items: Vec<SensitiveAuditEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |i| i.resource.clone());
    let items: Vec<SensitiveAuditEntry> = match &q.status {
        Some(s) if !s.is_empty() => items.into_iter().filter(|i| i.action == *s).collect(),
        _ => items,
    };
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    json_success(json!({"list": list, "total": total}))
}

async fn initiate_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = audit_write_store!(state, sensitive_audit_store);
    let new_id = store.entries().iter().map(|a| a.id).max().unwrap_or(0) + 1;
    let now = now_str();
    let audit = SensitiveAuditEntry {
        id: new_id,
        user_id: body.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0),
        action: body.get("operation_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        resource: body.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        detail: body.get("ip_address").and_then(|v| v.as_str()).map(String::from),
        created_at: now.clone(),
    };
    store.entries_mut().push(audit.clone());
    json_success(json!(audit))
}

async fn approve_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = audit_write_store!(state, sensitive_audit_store);
    match store.entries_mut().iter_mut().find(|a| a.id == id) {
        Some(audit) => {
            audit.action = "approved".to_string();
            audit.detail = body.get("reason").and_then(|v| v.as_str()).map(String::from);
            audit.created_at = now_str();
            json_success(json!(audit))
        }
        None => json_error("审计记录不存在"),
    }
}

async fn cancel_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = audit_write_store!(state, sensitive_audit_store);
    match store.entries_mut().iter_mut().find(|a| a.id == id) {
        Some(audit) => {
            audit.action = "cancelled".to_string();
            audit.detail = body.get("reason").and_then(|v| v.as_str()).map(String::from);
            audit.created_at = now_str();
            json_success(json!(audit))
        }
        None => json_error("审计记录不存在"),
    }
}

async fn verify_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = audit_read_store!(state, sensitive_audit_store);
    match store.entries().iter().find(|a| a.id == id) {
        Some(audit) => {
            let verified = audit.action == "approved";
            json_success(json!({"operation_id": id, "verified": verified, "action": &audit.action}))
        }
        None => json_error("审计记录不存在"),
    }
}

async fn resend_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = audit_write_store!(state, sensitive_audit_store);
    match store.entries_mut().iter_mut().find(|a| a.id == id) {
        Some(audit) => {
            audit.created_at = now_str();
            json_success(json!({"operation_id": id, "resent": true, "action": &audit.action}))
        }
        None => json_error("审计记录不存在"),
    }
}

async fn delete_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = audit_write_store!(state, sensitive_audit_store);
    let len = store.entries().len();
    store.entries_mut().retain(|a| a.id != id);
    if store.entries().len() < len {
        json_success(json!({"deleted": true}))
    } else {
        json_error("记录不存在")
    }
}

async fn batch_delete_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if let Some(ids) = body.get("ids").and_then(|v| v.as_array()) {
        let store = audit_write_store!(state, sensitive_audit_store);
        store.entries_mut().retain(|i| !ids.iter().any(|id| id.as_i64() == Some(i.id)));
    }
    json_ok()
}

async fn pending_sensitive_audits(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let store = audit_read_store!(state, sensitive_audit_store);
    let entries = store.entries();
    let total = entries.len();
    let pending: Vec<&SensitiveAuditEntry> = entries.iter().filter(|s| s.action == "pending").collect();
    json_success(json!({
        "list": pending,
        "total": total
    }))
}

async fn sensitive_audit_statistics(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let store = audit_read_store!(state, sensitive_audit_store);
    let total = store.entries().len();
    let pending = store.entries().iter().filter(|s| s.action == "pending").count();
    let approved = store.entries().iter().filter(|s| s.action == "approved").count();
    let cancelled = store.entries().iter().filter(|s| s.action == "cancelled").count();
    json_success(json!({"total": total, "pending": pending, "approved": approved, "cancelled": cancelled}))
}

async fn sensitive_audit_types() -> Json<Value> {
    json_success(json!(["login", "operation", "data_access", "permission_change", "config_change"]))
}

async fn sensitive_audit_expire_time() -> Json<Value> {
    json_success(json!({"expire_seconds": 86400}))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // IP 白名单
        .route("/api/security/ip-whitelist", get(list_ip_whitelist).post(create_ip_whitelist))
        .route("/api/security/ip-whitelist/{id}", get(get_ip_whitelist).put(update_ip_whitelist).delete(delete_ip_whitelist))
        // 敏感审计
        .route("/api/security/sensitive-audit", get(list_sensitive_audits))
        .route("/api/security/sensitive-audit/initiate", post(initiate_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/approve", put(approve_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/cancel", put(cancel_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/verify", post(verify_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/resend", post(resend_sensitive_audit))
        .route("/api/security/sensitive-audit/{id}", delete(delete_sensitive_audit))
        .route("/api/security/sensitive-audit/batch", delete(batch_delete_sensitive_audit))
        .route("/api/security/sensitive-audit/pending", get(pending_sensitive_audits))
        .route("/api/security/sensitive-audit/statistics", get(sensitive_audit_statistics))
        .route("/api/security/sensitive-audit/types", get(sensitive_audit_types))
        .route("/api/security/sensitive-audit/expire-time", get(sensitive_audit_expire_time))
}
