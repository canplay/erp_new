//! Browser Service HTTP API
//!
//! # API 端点
//!
//! - `POST /browser/create` — 创建浏览器标签页
//! - `POST /browser/:id/navigate` — 导航到 URL
//! - `GET  /browser/:id/text` — 获取页面文本内容
//! - `GET  /browser/:id/html` — 获取页面 HTML
//! - `GET  /browser/:id/title` — 获取页面标题
//! - `POST /browser/:id/click` — 点击元素
//! - `POST /browser/:id/type` — 输入文本
//! - `POST /browser/:id/cookies` — 注入 Cookie
//! - `POST /browser/:id/screenshot` — 截图(base64)
//! - `DELETE /browser/:id` — 关闭标签页
//! - `GET  /browser/:id/elements` — 获取页面元素信息
//! - `GET  /health` — 健康检查
//! - `GET  /browser/pool/status` — 池状态

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::browser_pool::{BrowserPool, BrowserSession};
use crate::helpers::{json_error_fmt, json_error_status, json_ok, json_unavailable};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<BrowserPool>,
    pub sessions: Arc<tokio::sync::Mutex<HashMap<String, BrowserSession>>>,
}

impl AppState {
    pub fn new(pool: Arc<BrowserPool>) -> Self {
        Self {
            pool,
            sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

/// 构建路由
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/browser/create", post(create_session))
        .route("/browser/pool/status", get(pool_status))
        .route("/browser/:id/navigate", post(navigate))
        .route("/browser/:id/text", get(get_text))
        .route("/browser/:id/html", get(get_html))
        .route("/browser/:id/title", get(get_title))
        .route("/browser/:id/click", post(click_element))
        .route("/browser/:id/type", post(type_text))
        .route("/browser/:id/cookies", post(inject_cookies))
        .route("/browser/:id/screenshot", post(take_screenshot))
        .route("/browser/:id/elements", get(get_elements))
        .route("/browser/:id", delete(close_session))
        .with_state(state)
}

// ===== 请求类型 =====

#[derive(Deserialize)]
pub struct NavigateRequest {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ClickRequest {
    pub selector: String,
}

#[derive(Deserialize)]
pub struct TypeRequest {
    pub selector: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct CookieItem {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct CookieRequest {
    pub cookies: Vec<CookieItem>,
}

// ===== 健康检查 =====

async fn health_check() -> Json<Value> {
    json_ok(json!({
        "status": "healthy",
        "service": "browser-service",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ===== 会话管理 =====

/// 创建新的浏览器标签页
async fn create_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page_arc = state.pool.get_page().ok_or_else(|| json_unavailable("浏览器池为空"))?;

    let session_id = Uuid::new_v4().to_string();
    let session = BrowserSession::new(session_id.clone());

    // 从浏览器池的 ChromiumPage 中创建新标签页
    let page = page_arc.lock().await;
    match page.new_tab(None).await {
        Ok(tab) => {
            *session.tab.lock().await = Some(tab);
        }
        Err(e) => {
            return Err(json_error_fmt("无法创建标签页", &e));
        }
    }

    let mut sessions = state.sessions.lock().await;
    sessions.insert(session_id.clone(), session);

    Ok(json_ok(json!({
        "session_id": session_id
    })))
}

/// 获取浏览器池状态
async fn pool_status(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let active_sessions = state.sessions.lock().await.len();
    json_ok(json!({
        "pool_size": state.pool.size(),
        "active_sessions": active_sessions
    }))
}

/// 关闭浏览器标签页
async fn close_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut sessions = state.sessions.lock().await;
    if let Some(session) = sessions.remove(&id) {
        let mut tab_guard = session.tab.lock().await;
        if let Some(tab) = tab_guard.take() {
            let _ = tab.close().await;
        }
        Ok(json_ok(json!({"status": "closed", "session_id": id})))
    } else {
        Err(json_error_status(StatusCode::NOT_FOUND, "会话不存在"))
    }
}

// ===== inline tab 操作辅助宏 =====

/// 简化从 session 获取 tab 引用的模式
macro_rules! with_tab {
    ($state:expr, $id:expr, |$tab:ident| $body:expr) => {{
        let sessions = $state.sessions.lock().await;
        let session = sessions.get(&$id).ok_or_else(|| {
            json_error_status(StatusCode::NOT_FOUND, "会话不存在")
        })?;
        let mut tab_guard = session.tab.lock().await;
        let $tab = tab_guard.as_mut().ok_or_else(|| {
            json_error_status(StatusCode::BAD_REQUEST, "标签页未初始化")
        })?;
        $body
    }};
}

// ===== 页面操作 =====

/// 导航到 URL
async fn navigate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<NavigateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        match tab.get(&req.url).await {
            Ok(success) => {
                let title = tab.title().await.unwrap_or_default();
                Ok(json_ok(json!({"status": if success { "ok" } else { "timeout" }, "title": title, "url": req.url})))
            }
            Err(e) => Err(json_error_fmt("导航失败", &e)),
        }
    })
}

/// 获取页面纯文本内容
async fn get_text(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        let text = tab.ele_text("body").await.unwrap_or(None).unwrap_or_default();
        Ok(json_ok(json!({"text": text})))
    })
}

/// 获取页面 HTML
async fn get_html(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        let html = tab.html().await.unwrap_or_default();
        Ok(json_ok(json!({"html": html})))
    })
}

/// 获取页面标题
async fn get_title(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        let title = tab.title().await.unwrap_or_default();
        Ok(json_ok(json!({"title": title})))
    })
}

/// 点击元素
async fn click_element(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ClickRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        match tab.click(&req.selector).await {
            Ok(()) => Ok(json_ok(json!({"status": "clicked", "selector": req.selector}))),
            Err(e) => Err(json_error_fmt("点击失败", &e)),
        }
    })
}

/// 在元素中输入文本
async fn type_text(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<TypeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        match tab.input(&req.selector, &req.text).await {
            Ok(()) => Ok(json_ok(json!({"status": "typed", "selector": req.selector}))),
            Err(e) => Err(json_error_fmt("输入失败", &e)),
        }
    })
}

/// 注入 Cookie
async fn inject_cookies(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CookieRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        let cookie_params: Vec<drission::cdp::CookieParam> = req
            .cookies
            .iter()
            .map(|c| drission::cdp::CookieParam {
                name: c.name.clone(),
                value: c.value.clone(),
                url: None,
                domain: Some(c.domain.clone()),
                path: c.path.clone(),
                secure: None,
                http_only: None,
                expires: None,
            })
            .collect();

        match tab.set_cookies(cookie_params).await {
            Ok(()) => Ok(json_ok(json!({
                "status": "injected",
                "count": req.cookies.len()
            }))),
            Err(e) => Err(json_error_fmt("Cookie 注入失败", &e)),
        }
    })
}

/// 截图（返回 base64）
async fn take_screenshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    with_tab!(state, id, |tab| {
        match tab.screenshot_base64(false).await {
            Ok(b64) => Ok(json_ok(json!({
                "status": "captured",
                "image_base64": b64
            }))),
            Err(e) => Err(json_error_fmt("截图失败", &e)),
        }
    })
}

/// 获取页面元素（通过 CSS 选择器）
async fn get_elements(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let selector = params.get("selector").cloned().unwrap_or_default();
    if selector.is_empty() {
        return Err(json_error_status(StatusCode::BAD_REQUEST, "缺少 selector 查询参数"));
    }

    with_tab!(state, id, |tab| {
        match tab.eles(&selector).await {
            Ok(elements) => {
                let mut infos = Vec::new();
                for el in &elements {
                    let tag = el.tag().await.unwrap_or_default();
                    let text = el.text().await.unwrap_or_default();
                    let mut attrs = std::collections::HashMap::new();
                    if let Ok(Some(src)) = el.attr("src").await {
                        attrs.insert("src".to_string(), src);
                    }
                    if let Ok(Some(href)) = el.attr("href").await {
                        attrs.insert("href".to_string(), href);
                    }
                    if let Ok(Some(cls)) = el.attr("class").await {
                        attrs.insert("class".to_string(), cls);
                    }
                    infos.push(json!({
                        "tag": tag,
                        "text": text,
                        "attributes": attrs,
                    }));
                }
                Ok(json_ok(json!({"elements": infos, "count": infos.len()})))
            }
            Err(e) => Err(json_error_fmt("查找元素失败", &e)),
        }
    })
}
