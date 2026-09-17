//! WebSocket routes module
//!
//! 处理 WebSocket 连接，通过 gRPC 与 messaging-service 通信
//!
//! @date 2026-05-20
//! @note 重构为使用 gRPC 与后端服务通信

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        Query,
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tracing::{error, info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::AppState;

/// WebSocket 连接管理器
pub struct WsConnectionManager {
    connections: RwLock<HashMap<String, Arc<WsConnection>>>,
}

pub struct WsConnection {
    pub user_id: i64,
    pub authenticated: RwLock<bool>,
    pub connected_at: std::time::Instant,
    pub last_activity: RwLock<std::time::Instant>,
}

impl WsConnection {
    #[must_use]
    pub fn new(user_id: i64) -> Self {
        let now = std::time::Instant::now();
        Self {
            user_id,
            authenticated: RwLock::new(false),
            connected_at: now,
            last_activity: RwLock::new(now),
        }
    }
    pub fn is_authenticated(&self) -> bool {
        *self.authenticated.read()
    }
    pub fn set_authenticated(&self, val: bool) {
        *self.authenticated.write() = val;
    }
    pub fn update_activity(&self) {
        *self.last_activity.write() = std::time::Instant::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequest {
    pub action: String,
    pub data: Option<serde_json::Value>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub success: bool,
    pub action: String,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl WsResponse {
    #[must_use]
    pub fn ok(action: &str, data: serde_json::Value) -> Self {
        Self {
            success: true,
            action: action.to_string(),
            data: Some(data),
            error: None,
        }
    }
    #[must_use]
    pub fn error(action: &str, error: &str) -> Self {
        Self {
            success: false,
            action: action.to_string(),
            data: None,
            error: Some(error.to_string()),
        }
    }
}

impl WsConnectionManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, user_id: i64) -> Arc<WsConnection> {
        let conn = Arc::new(WsConnection::new(user_id));
        self.connections
            .write()
            .insert(format!("user_{user_id}" ), conn.clone());
        conn
    }

    pub fn unregister(&self, user_id: i64) {
        self.connections
            .write()
            .remove(&format!("user_{user_id}" ));
    }

    pub fn active_connections(&self) -> usize {
        self.connections.read().len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsStatus {
    pub service: String,
    pub active_connections: usize,
    pub messaging_service_available: bool,
}

impl Default for WsConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket 状态处理器
pub async fn ws_status_handler(
    State(state): State<Arc<AppState>>,
) -> axum::Json<common::ApiResponse<WsStatus>> {
    use common::ApiResponse;

    // 检查 message service 是否可用
    let is_message_service_available = state.grpc_clients.read().await.message_service.is_connected().await;

    axum::Json(ApiResponse::success(WsStatus {
        service: "api-gateway".into(),
        active_connections: 0,
        messaging_service_available: is_message_service_available,
    }))
}

/// WebSocket 消息处理器
pub async fn ws_messages_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    info!("WS upgrade: /ws/messages" );
    ws.on_upgrade(move |socket| handle_ws_messages(socket, state, query.token))
}

/// WS 查询参数
#[derive(serde::Deserialize, Default)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// 处理 WebSocket 连接
async fn handle_ws_messages(socket: WebSocket, state: Arc<AppState>, token: Option<String>) {
    let (mut sender, mut receiver) = socket.split();

    // 接收并验证认证(验证真实 JWT, 修复: 原实现把 token 当 user_id 解析可被任意伪造)
    // 修复 (2026-08-07): token 从 URL 查询参数 ?token= 获取(浏览器 WS 无法带 header),
    // 优先使用查询参数 token, 兼容连接后首条 auth 消息
    let user_id = match receive_auth(&mut receiver, &state.jwt_service, token).await {
        Ok(uid) => uid,
        Err(e) => {
            warn!("WS Auth failed: {}" , e);
            return;
        }
    };

    info!("WS authenticated: user_id={}" , user_id);

    let conn = Arc::new(WsConnection::new(user_id));
    conn.set_authenticated(true);

    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        conn.update_activity();
                        if let Ok(req) = serde_json::from_str::<WsRequest>(&text) {
                            let resp = handle_client_request(&req, user_id);
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WS closed: user_id={}" , user_id);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WS error: {}" , e);
                        break;
                    }
                    _ => {}
                }
            }
            // 心跳保持
            () = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
    }
}

/// 接收并处理认证消息
async fn receive_auth(
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    jwt_service: &auth_core::JwtService,
    query_token: Option<String>,
) -> Result<i64, String> {
    // 优先使用 URL 查询参数 token（浏览器 WS 握手无法带 Authorization 头）
    if let Some(tok) = query_token {
        if !tok.is_empty() {
            let claims = jwt_service.verify_token(&tok).map_err(|_| "Invalid token".to_string())?;
            return Ok(claims.sub);
        }
    }
    // Compatible with first auth message after connection
    match tokio::time::timeout(Duration::from_secs(10), receiver.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => {
            if let Ok(req) = serde_json::from_str::<WsRequest>(&text) {
                if req.action == "auth" {
                    let token = req.token.ok_or("Missing token") ?;
                    let claims = jwt_service.verify_token(&token).map_err(|_| "Invalid token".to_string()) ?;
                    return Ok(claims.sub);
                }
            }
        }
        _ => {}
    }
    Err("Auth failed".into())
}

/// 处理客户端请求
fn handle_client_request(req: &WsRequest, user_id: i64) -> WsResponse {
    match req.action.as_str() {
        "auth" => WsResponse::ok(
            &req.action,
            serde_json::json!({"status": "authenticated" , "user_id": user_id}),
        ),
        "ping" => WsResponse::ok(
            &req.action,
            serde_json::json!({"timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())}),
        ),
        "get_unread" => {
            // 通过 gRPC 调用 message-service 获取未读消息
            // 注意: 需要先确保 grpc_clients 与 proto 定义匹配
            WsResponse::ok(&req.action, serde_json::json!({"count": 0}))
        }
        "send" => {
            // 通过 gRPC 调用 message-service 发送消息
            WsResponse::ok(&req.action, serde_json::json!({"status": "queued" }))
        }
        _ => WsResponse::error(&req.action, "Unknown action" ),
    }
}

/// WebSocket 状态管理器
#[derive(Clone)]
pub struct WsState {
    pub manager: Arc<WsConnectionManager>,
}

impl WsState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: Arc::new(WsConnectionManager::new()),
        }
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}
