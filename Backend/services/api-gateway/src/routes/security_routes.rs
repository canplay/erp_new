//! 安全/报表/导出/定时任务模块路由 — 真实内存存储实现
//!
//! 使用 Arc<std::sync::RwLock<Vec<T>>> 做内存存储，重启后数据丢失。
//! 生产环境应替换为对应 service 的 gRPC 调用。

use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AppState;
use crate::routes::helpers::*;
use crate::repository::IpWhitelistEntry as RepoIpWhitelistEntry;
use crate::repository::SensitiveAuditEntry as RepoSensitiveAuditEntry;
use crate::repository::ScheduledTaskEntry as RepoScheduledTaskEntry;
use crate::repository::ReportEntry as RepoReportEntry;
use crate::repository::DataSourceEntry as RepoDataSourceEntry;
use crate::repository::ReportTemplateEntry as RepoReportTemplateEntry;

// ==================== 数据结构 ====================

/// IP 白名单规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistRule {
    pub id: i64,
    pub name: String,
    pub ip_address: Option<String>,
    pub ip_start: Option<String>,
    pub ip_end: Option<String>,
    pub mask: Option<String>,
    pub target_type: String,
    pub target_ids: Option<Vec<i64>>,
    pub description: Option<String>,
    pub priority: i32,
    pub is_enabled: bool,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// 敏感审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveAudit {
    pub id: i64,
    pub operation_id: String,
    pub user_id: Option<i64>,
    pub username: String,
    pub operation_type: String,
    pub description: String,
    pub ip_address: String,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: i64,
    pub name: String,
    pub task_type: String,
    pub cron_expr: Option<String>,
    pub interval_secs: Option<i64>,
    pub handler: String,
    pub params: Option<Value>,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportItem {
    pub id: i64,
    pub name: String,
    pub report_type: String,
    pub config: Option<Value>,
    pub status: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 序列化为前端期望字段名（report_type→reportType，config→description/queryParams）
pub fn report_item_to_json(r: &ReportItem) -> Value {
    let config = r.config.clone().unwrap_or_else(|| json!({}));
    json!({
        "id": r.id,
        "name": r.name,
        "reportType": r.report_type,
        "description": config.get("description" ).cloned().unwrap_or(Value::Null),
        "queryParams": config.get("queryParams" ).cloned().unwrap_or_else(|| json!({})),
        "status": r.status,
        "created_by": r.created_by,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
    })
}

/// 数据源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: i64,
    pub name: String,
    pub ds_type: String,
    pub config: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// 报表模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: i64,
    pub name: String,
    pub template_type: String,
    pub config: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// 报表模板种子数据（内存 store，进程重启后重建）
pub fn seed_report_templates() -> Vec<ReportTemplate> {
    let now = now_str();
    vec![
        ReportTemplate {
            id: 1,
            name: "月度销售汇总表".to_string(),
            template_type: "table".to_string(),
            config: Some(json!({
                "description": "按月份汇总销售金额与订单量" ,
                "queryParams": { "start_date": " ", "end_date": " " },
                "columns": ["月份" , "销售额" , "订单数" , "环比" ]
            })),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        ReportTemplate {
            id: 2,
            name: "用户增长趋势图".to_string(),
            template_type: "chart".to_string(),
            config: Some(json!({
                "description": "展示用户注册量随时间变化趋势" ,
                "queryParams": { "days": 30 },
                "chartType": "line"
            })),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        ReportTemplate {
            id: 3,
            name: "运营总览看板".to_string(),
            template_type: "dashboard".to_string(),
            config: Some(json!({
                "description": "关键指标一屏总览：用户、登录、活跃、转化" ,
                "queryParams": {},
                "widgets": ["total_users" , "login_attempts" , "active_users" ]
            })),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        ReportTemplate {
            id: 4,
            name: "登录失败分析表".to_string(),
            template_type: "table".to_string(),
            config: Some(json!({
                "description": "按 IP/用户统计登录失败次数" ,
                "queryParams": { "start_date": " ", "end_date": " " },
                "columns": ["用户名" , "IP" , "失败次数" , "最近失败时间" ]
            })),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        ReportTemplate {
            id: 5,
            name: "停车收入趋势图".to_string(),
            template_type: "chart".to_string(),
            config: Some(json!({
                "description": "停车场每日收入与车流量趋势" ,
                "queryParams": { "days": 7 },
                "chartType": "bar"
            })),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
    ]
}

/// 序列化为前端期望的字段名（template_type→reportType, config→description/queryParams）
pub fn report_template_to_json(t: &ReportTemplate) -> Value {
    let config = t.config.clone().unwrap_or_else(|| json!({}));
    json!({
        "id": t.id,
        "name": t.name,
        "reportType": t.template_type,
        "description": config.get("description" ).cloned().unwrap_or(Value::Null),
        "queryParams": config.get("queryParams" ).cloned().unwrap_or_else(|| json!({})),
        "config": t.config,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })
}

// ==================== 查询参数 ====================

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

// ==================== 辅助函数 ====================

fn now_str() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ" ).to_string()
}

fn paginate<T: Clone>(items: &[T], page: usize, page_size: usize) -> (Vec<T>, usize) {
    let total = items.len();
    let start = (page.saturating_sub(1)) * page_size;
    let end = std::cmp::min(start + page_size, total);
    let list = if start < total { items[start..end].to_vec() } else { vec![] };
    (list, total)
}

fn filter_by_keyword<T, F>(items: Vec<T>, keyword: &Option<String>, f: F) -> Vec<T>
where F: Fn(&T) -> String {
    match keyword {
        Some(kw) if !kw.is_empty() => {
            let kw = kw.to_lowercase();
            items.into_iter().filter(|i| f(i).to_lowercase().contains(&kw)).collect()
        }
        _ => items,
    }
}

macro_rules! read_store {
    ($state:expr, $field:ident) => {{
        $state.$field.read().await
    }};
}

macro_rules! write_store {
    ($state:expr, $field:ident) => {{
        $state.$field.write().await
    }};
}

// ==================== IP 白名单 Handler ====================

/// GET /api/security/ip-whitelist
async fn list_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = read_store!(state, ip_whitelist_store);
    let items: Vec<RepoIpWhitelistEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |i| i.ip.clone());
    let items: Vec<RepoIpWhitelistEntry> = match &q.status {
        Some(s) if s == "enabled" => items.into_iter().filter(|i| i.is_active).collect(),
        Some(s) if s == "disabled" => items.into_iter().filter(|i| !i.is_active).collect(),
        _ => items,
    };
    let (list, total) = paginate(&items, q.page.unwrap_or(1), q.page_size.unwrap_or(20));
    json_success(json!({"list": list, "total": total}))
}

/// POST /api/security/ip-whitelist
async fn create_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = write_store!(state, ip_whitelist_store);
    let new_id = store.entries().iter().map(|r| r.id).max().unwrap_or(0) + 1;
    let now = now_str();
    let entry = RepoIpWhitelistEntry {
        id: new_id,
        ip: body.get("ip_address" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        description: body.get("description" ).and_then(|v| v.as_str()).map(String::from),
        is_active: true,
        created_at: now.clone(),
    };
    store.entries_mut().push(entry.clone());
    json_success(json!(entry))
}

/// GET /api/security/ip-whitelist/{id}
async fn get_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = read_store!(state, ip_whitelist_store);
    match store.entries().iter().find(|r| r.id == id) {
        Some(rule) => json_success(json!(rule)),
        None => json_error("规则不存在" ),
    }
}

/// PUT /api/security/ip-whitelist/{id}
async fn update_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = write_store!(state, ip_whitelist_store);
    if let Some(entry) = store.entries_mut().iter_mut().find(|r| r.id == id) {
        if let Some(ip) = body.get("ip_address" ).and_then(|v| v.as_str()) { entry.ip = ip.to_string(); }
        if let Some(desc) = body.get("description" ).and_then(|v| v.as_str()) { entry.description = Some(desc.to_string()); }
        entry.created_at = now_str();
        json_success(json!(entry))
    } else {
        json_error("规则不存在" )
    }
}

/// DELETE /api/security/ip-whitelist/{id}
async fn delete_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, ip_whitelist_store);
    let len = store.entries().len();
    store.entries_mut().retain(|r| r.id != id);
    if store.entries().len() < len {
        json_success(json!({"deleted": true}))
    } else {
        json_error("规则不存在" )
    }
}

/// PUT /api/security/ip-whitelist/{id}/enable
async fn enable_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, ip_whitelist_store);
    match store.entries_mut().iter_mut().find(|r| r.id == id) {
        Some(entry) => { entry.is_active = true; entry.created_at = now_str(); json_success(json!(entry)) }
        None => json_error("规则不存在" ),
    }
}

/// PUT /api/security/ip-whitelist/{id}/disable
async fn disable_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, ip_whitelist_store);
    match store.entries_mut().iter_mut().find(|r| r.id == id) {
        Some(entry) => { entry.is_active = false; entry.created_at = now_str(); json_success(json!(entry)) }
        None => json_error("规则不存在" ),
    }
}

// ==================== 敏感审计 Handler ====================

async fn list_sensitive_audits(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = read_store!(state, sensitive_audit_store);
    let items: Vec<RepoSensitiveAuditEntry> = store.entries().clone();
    drop(store);
    let items = filter_by_keyword(items, &q.keyword, |i| i.resource.clone());
    let items: Vec<RepoSensitiveAuditEntry> = match &q.status {
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
    let store = write_store!(state, sensitive_audit_store);
    let new_id = store.entries().iter().map(|a| a.id).max().unwrap_or(0) + 1;
    let now = now_str();
    let audit = RepoSensitiveAuditEntry {
        id: new_id,
        user_id: body.get("user_id" ).and_then(|v| v.as_i64()).unwrap_or(0),
        action: body.get("operation_type" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        resource: body.get("description" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        detail: body.get("ip_address" ).and_then(|v| v.as_str()).map(String::from),
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
    let store = write_store!(state, sensitive_audit_store);
    match store.entries_mut().iter_mut().find(|a| a.id == id) {
        Some(audit) => {
            audit.action = "approved".to_string();
            audit.detail = body.get("reason" ).and_then(|v| v.as_str()).map(String::from);
            audit.created_at = now_str();
            json_success(json!(audit))
        }
        None => json_error("审计记录不存在" ),
    }
}

async fn cancel_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = write_store!(state, sensitive_audit_store);
    match store.entries_mut().iter_mut().find(|a| a.id == id) {
        Some(audit) => {
            audit.action = "cancelled".to_string();
            audit.detail = body.get("reason" ).and_then(|v| v.as_str()).map(String::from);
            audit.created_at = now_str();
            json_success(json!(audit))
        }
        None => json_error("审计记录不存在" ),
    }
}

async fn verify_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = read_store!(state, sensitive_audit_store);
    match store.entries().iter().find(|a| a.id == id) {
        Some(audit) => {
            let verified = audit.action == "approved";
            json_success(json!({"operation_id": id, "verified": verified, "action": &audit.action}))
        }
        None => json_error("审计记录不存在" ),
    }
}

async fn resend_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, sensitive_audit_store);
    match store.entries_mut().iter_mut().find(|a| a.id == id) {
        Some(audit) => {
            audit.created_at = now_str();
            json_success(json!({"operation_id": id, "resent": true, "action": &audit.action}))
        }
        None => json_error("审计记录不存在" ),
    }
}

async fn delete_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, sensitive_audit_store);
    let len = store.entries().len();
    store.entries_mut().retain(|a| a.id != id);
    if store.entries().len() < len {
        json_success(json!({"deleted": true}))
    } else {
        json_error("记录不存在" )
    }
}

// ==================== 定时任务 Handler ====================

async fn list_scheduled_tasks(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = read_store!(state, scheduled_task_store);
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

async fn create_scheduled_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    let new_id = store.entries().iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let now = now_str();
    let task = RepoScheduledTaskEntry {
        id: new_id,
        name: body.get("name" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        cron: body.get("cron_expr" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        handler: body.get("handler" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string(),
        is_active: body.get("enabled" ).and_then(|v| v.as_bool()).unwrap_or(true),
        created_at: now.clone(),
    };
    store.entries_mut().push(task.clone());
    json_success(json!(task))
}

async fn get_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = read_store!(state, scheduled_task_store);
    match store.entries().iter().find(|t| t.id == id) {
        Some(task) => json_success(json!(task)),
        None => json_error("任务不存在" ),
    }
}

async fn update_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    if let Some(task) = store.entries_mut().iter_mut().find(|t| t.id == id) {
        if let Some(name) = body.get("name" ).and_then(|v| v.as_str()) { task.name = name.to_string(); }
        if let Some(expr) = body.get("cron_expr" ).and_then(|v| v.as_str()) { task.cron = expr.to_string(); }
        if let Some(_secs) = body.get("interval_secs" ).and_then(|v| v.as_i64()) { /* interval_secs not in RepoScheduledTaskEntry */ }
        if let Some(handler) = body.get("handler" ).and_then(|v| v.as_str()) { task.handler = handler.to_string(); }
        task.created_at = now_str();
        json_success(json!(task))
    } else {
        json_error("任务不存在" )
    }
}

async fn delete_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    let len = store.entries().len();
    store.entries_mut().retain(|t| t.id != id);
    if store.entries().len() < len {
        json_success(json!({"deleted": true}))
    } else {
        json_error("任务不存在" )
    }
}

async fn enable_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = true; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在" ),
    }
}

async fn disable_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = false; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在" ),
    }
}

// ==================== 定时任务操作 ====================

async fn trigger_scheduled_task(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i64>,
) -> Json<Value> {
    json_ok()
}

async fn pause_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = false; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在" ),
    }
}

async fn resume_scheduled_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<Value> {
    let store = write_store!(state, scheduled_task_store);
    match store.entries_mut().iter_mut().find(|t| t.id == id) {
        Some(task) => { task.is_active = true; task.created_at = now_str(); json_success(json!(task)) }
        None => json_error("任务不存在" ),
    }
}

// ==================== 报表 Handler ====================

async fn list_reports(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Json<Value> {
    let store = read_store!(state, report_store);
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
            "created_by": "admin" ,
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
    let store = read_store!(state, data_source_store);
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
    let store = read_store!(state, report_template_store);
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

/// 从模板创建报表（POST /api/report-templates/{templateId}/create）
async fn create_report_from_template(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(template_id): axum::extract::Path<i64>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let name = body.get("name" ).and_then(|v| v.as_str()).unwrap_or("" ).to_string();
    let name_display = name.clone();
    let template = {
        let store = read_store!(state, report_template_store);
        store.entries().iter().find(|t| t.id == template_id).cloned()
    };
    let template = match template {
        Some(t) => t,
        None => return json_error("模板不存在" ),
    };
    let config = template.config.clone().unwrap_or_default();
    let report_store = write_store!(state, report_store);
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

// ==================== IP 白名单扩展 Handler ====================

async fn batch_delete_ip_whitelist(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let store = write_store!(state, ip_whitelist_store);
    store.entries_mut().clear();
    json_ok()
}

async fn batch_enable_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if let Some(ids) = body.get("ids" ).and_then(|v| v.as_array()) {
        let store = write_store!(state, ip_whitelist_store);
        for item in store.entries_mut().iter_mut() {
            if ids.iter().any(|id| id.as_i64() == Some(item.id)) {
                item.is_active = true;
            }
        }
    }
    json_ok()
}

async fn batch_disable_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if let Some(ids) = body.get("ids" ).and_then(|v| v.as_array()) {
        let store = write_store!(state, ip_whitelist_store);
        for item in store.entries_mut().iter_mut() {
            if ids.iter().any(|id| id.as_i64() == Some(item.id)) {
                item.is_active = false;
            }
        }
    }
    json_ok()
}

async fn reorder_ip_whitelist() -> Json<Value> { json_ok() }

async fn check_ip_whitelist(
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let ip = q.get("ip" ).cloned().unwrap_or_default();
    let store = read_store!(state, ip_whitelist_store);
    let found = store.entries().iter().any(|r| r.ip.as_str() == ip.as_str() && r.is_active);
    json_success(json!({"allowed": found, "ip": ip}))
}

async fn ip_whitelist_statistics(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let store = read_store!(state, ip_whitelist_store);
    let total = store.entries().len();
    let enabled = store.entries().iter().filter(|r| r.is_active).count();
    json_success(json!({"total": total, "enabled": enabled, "disabled": total - enabled}))
}

async fn ip_whitelist_location() -> Json<Value> {
    json_success(json!({"country": " ", "city": " ", "isp": " "}))
}

// ==================== 敏感审计扩展 Handler ====================

async fn batch_delete_sensitive_audit(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if let Some(ids) = body.get("ids" ).and_then(|v| v.as_array()) {
        let store = write_store!(state, sensitive_audit_store);
        store.entries_mut().retain(|i| !ids.iter().any(|id| id.as_i64() == Some(i.id)));
    }
    json_ok()
}

async fn pending_sensitive_audits(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let store = read_store!(state, sensitive_audit_store);
    let entries = store.entries();
    let total = entries.len();
    let pending: Vec<&RepoSensitiveAuditEntry> = entries.iter().filter(|s| s.action == "pending" ).collect();
    json_success(json!({
        "list": pending,
        "total": total
    }))
}

async fn sensitive_audit_statistics(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let store = read_store!(state, sensitive_audit_store);
    let total = store.entries().len();
    let pending = store.entries().iter().filter(|s| s.action == "pending" ).count();
    let approved = store.entries().iter().filter(|s| s.action == "approved" ).count();
    let cancelled = store.entries().iter().filter(|s| s.action == "cancelled" ).count();
    json_success(json!({"total": total, "pending": pending, "approved": approved, "cancelled": cancelled}))
}

async fn sensitive_audit_types() -> Json<Value> {
    json_success(json!(["login" , "operation" , "data_access" , "permission_change" , "config_change" ]))
}

async fn sensitive_audit_expire_time() -> Json<Value> {
    json_success(json!({"expire_seconds": 86400}))
}

// ==================== 导出任务 Handler ====================

async fn list_export_tasks() -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn create_export_task() -> Json<Value> {
    json_success(json!({"id": 0}))
}
async fn get_export_task(axum::extract::Path(id): axum::extract::Path<i64>) -> Json<Value> {
    json_success(json!({"id": id, "status": "pending" , "progress": 0}))
}
async fn cancel_export_task(axum::extract::Path(_id): axum::extract::Path<i64>) -> Json<Value> { json_ok() }
async fn retry_export_task(axum::extract::Path(_id): axum::extract::Path<i64>) -> Json<Value> { json_ok() }
async fn export_task_progress(axum::extract::Path(id): axum::extract::Path<i64>) -> Json<Value> {
    json_success(json!({"id": id, "progress": 0, "status": "pending" }))
}
async fn download_export_task(axum::extract::Path(_id): axum::extract::Path<i64>) -> Json<Value> {
    json_success(json!({"url": " "}))
}
async fn batch_create_export_task() -> Json<Value> {
    json_success(json!({"ids": []}))
}
async fn cleanup_export_tasks() -> Json<Value> { json_ok() }

// 导出任务统计
async fn count_export_tasks_handler() -> Json<Value> {
    json_success(json!({"count": 0}))
}

async fn export_tasks_stats_handler() -> Json<Value> {
    json_success(json!({"total": 0, "pending": 0, "running": 0, "completed": 0, "failed": 0}))
}

async fn export_users_file() -> Json<Value> {
    json_error("导出待实现" )
}
async fn export_login_logs_file() -> Json<Value> {
    json_error("导出待实现" )
}
async fn export_operation_logs_file() -> Json<Value> {
    json_error("导出待实现" )
}
async fn export_audit_logs_file() -> Json<Value> {
    json_error("导出待实现" )
}

// ==================== 路由定义 ====================

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // IP 白名单
        .route("/api/security/ip-whitelist" , get(list_ip_whitelist).post(create_ip_whitelist))
        .route("/api/security/ip-whitelist/{id}" , get(get_ip_whitelist).put(update_ip_whitelist).delete(delete_ip_whitelist))
        .route("/api/security/ip-whitelist/{id}/enable" , put(enable_ip_whitelist))
        .route("/api/security/ip-whitelist/{id}/disable" , put(disable_ip_whitelist))
        .route("/api/security/ip-whitelist/batch" , delete(batch_delete_ip_whitelist))
        .route("/api/security/ip-whitelist/batch-enable" , put(batch_enable_ip_whitelist))
        .route("/api/security/ip-whitelist/batch-disable" , put(batch_disable_ip_whitelist))
        .route("/api/security/ip-whitelist/reorder" , put(reorder_ip_whitelist))
        .route("/api/security/ip-whitelist/check" , get(check_ip_whitelist))
        .route("/api/security/ip-whitelist/statistics" , get(ip_whitelist_statistics))
        .route("/api/security/ip-whitelist/location" , get(ip_whitelist_location))
        // 敏感审计
        .route("/api/security/sensitive-audit" , get(list_sensitive_audits))
        .route("/api/security/sensitive-audit/initiate" , post(initiate_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/approve" , put(approve_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/cancel" , put(cancel_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/verify" , axum::routing::post(verify_sensitive_audit))
        .route("/api/security/sensitive-audit/{operation_id}/resend" , axum::routing::post(resend_sensitive_audit))
        .route("/api/security/sensitive-audit/{id}" , delete(delete_sensitive_audit))
        .route("/api/security/sensitive-audit/batch" , delete(batch_delete_sensitive_audit))
        .route("/api/security/sensitive-audit/pending" , get(pending_sensitive_audits))
        .route("/api/security/sensitive-audit/statistics" , get(sensitive_audit_statistics))
        .route("/api/security/sensitive-audit/types" , get(sensitive_audit_types))
        .route("/api/security/sensitive-audit/expire-time" , get(sensitive_audit_expire_time))
        // 定时任务
        .route("/api/scheduled-tasks" , get(list_scheduled_tasks).post(create_scheduled_task))
        .route("/api/scheduled-tasks/{id}" , get(get_scheduled_task).put(update_scheduled_task).delete(delete_scheduled_task))
        .route("/api/scheduled-tasks/{id}/enable" , put(enable_scheduled_task))
        .route("/api/scheduled-tasks/{id}/disable" , put(disable_scheduled_task))
        .route("/api/scheduled-tasks/{id}/trigger" , post(trigger_scheduled_task))
        .route("/api/scheduled-tasks/{id}/pause" , post(pause_scheduled_task))
        .route("/api/scheduled-tasks/{id}/resume" , post(resume_scheduled_task))
        // 导出任务
        .route("/api/export/tasks" , get(list_export_tasks).post(create_export_task))
        .route("/api/export/tasks/create" , post(create_export_task))
        .route("/api/export/tasks/list" , get(list_export_tasks))
        .route("/api/export/tasks/count" , get(count_export_tasks_handler))
        .route("/api/export/tasks/stats" , get(export_tasks_stats_handler))
        .route("/api/export/tasks/batch-create" , post(batch_create_export_task))
        .route("/api/export/tasks/cleanup" , post(cleanup_export_tasks))
        .route("/api/export/tasks/{id}" , get(get_export_task))
        .route("/api/export/tasks/{id}/cancel" , post(cancel_export_task))
        .route("/api/export/tasks/{id}/retry" , post(retry_export_task))
        .route("/api/export/tasks/{id}/progress" , get(export_task_progress))
        .route("/api/export/tasks/{id}/download" , get(download_export_task))
        .route("/api/export/users" , get(export_users_file))
        .route("/api/export/login-logs" , get(export_login_logs_file))
        .route("/api/export/operation-logs" , get(export_operation_logs_file))
        .route("/api/export/audit-logs" , get(export_audit_logs_file))
        // 报表
        .route("/api/reports" , get(list_reports).post(create_scheduled_task))
        .route("/api/data-sources" , get(list_data_sources))
        .route("/api/report-templates" , get(list_report_templates))
        .route("/api/report-templates/{template_id}/create" , post(create_report_from_template))
        .route("/api/reports/{id}" , get(get_export_task).put(update_scheduled_task).delete(delete_scheduled_task))
        .route("/api/reports/generate/{id}" , post(trigger_scheduled_task))
        .route("/api/reports/download/{id}" , get(download_export_task))
        .route("/api/reports/execute/{id}" , post(trigger_scheduled_task))
        .route("/api/reports/export/{id}" , post(trigger_scheduled_task))
        .route("/api/reports/data/{id}" , get(list_reports))
        .route("/api/reports/history/{id}" , get(list_reports))
}
