//! WebSocket service module

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State};
use axum::response::IntoResponse;
use axum::Json;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::grpc_server::AppState;

pub struct WsSession {
    pub user_id: i64,
    pub active: RwLock<bool>,
    pub last_activity: RwLock<std::time::Instant>,
}

impl WsSession {
    pub fn new(user_id: i64) -> Self {
        Self { user_id, active: RwLock::new(true), last_activity: RwLock::new(std::time::Instant::now()) }
    }
    pub async fn is_active(&self) -> bool { *self.active.read().await }
    pub async fn update_activity(&self) { *self.last_activity.write().await = std::time::Instant::now(); }
    pub async fn deactivate(&self) { *self.active.write().await = false; }
}

pub struct WsSessionManager { sessions: RwLock<Vec<Arc<WsSession>>> }
impl WsSessionManager { pub fn new() -> Self { Self { sessions: RwLock::new(Vec::new()) } } }
impl Default for WsSessionManager { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequest { pub action: String, pub data: Option<serde_json::Value>, pub token: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse { pub success: bool, pub action: String, pub data: Option<serde_json::Value>, pub error: Option<String> }

impl WsResponse {
    pub fn ok(action: &str, data: serde_json::Value) -> Self { Self { success: true, action: action.to_string(), data: Some(data), error: None } }
    pub fn error(action: &str, error: &str) -> Self { Self { success: false, action: action.to_string(), data: None, error: Some(error.to_string()) } }
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    info!("WebSocket upgrade: /ws");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let user_id = match receive_auth(&mut receiver).await {
        Ok(uid) => uid,
        Err(e) => { warn!("Auth failed: {}", e); return; }
    };
    info!("WebSocket connected: user_id={}", user_id);
    let session = Arc::new(WsSession::new(user_id));
    loop {
        tokio::select! {
            msg = receiver.next() => { if let Some(Ok(Message::Text(text))) = msg { session.update_activity().await; let _ = handle_message(&text, &mut sender, &state, user_id).await; } }
            _ = tokio::time::sleep(Duration::from_secs(60)) => { if !session.is_active().await { break; } }
        }
    }
}

async fn receive_auth(receiver: &mut futures_util::stream::SplitStream<WebSocket>) -> Result<i64, String> {
    if let Some(Ok(Message::Text(text))) = tokio::time::timeout(Duration::from_secs(10), receiver.next()).await.ok().flatten() {
        if let Ok(request) = serde_json::from_str::<WsRequest>(&text) { if request.action == "auth" { return Ok(request.token.unwrap_or("1".to_string()).parse().unwrap_or(1)); } }
    }
    Err("Auth failed".to_string())
}

async fn handle_message(text: &str, sender: &mut futures_util::stream::SplitSink<WebSocket, Message>, state: &AppState, user_id: i64) -> Result<(), String> {
    let request: WsRequest = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let action = &request.action;
    let response = match action.as_str() {
        "send" => { let _ = state.message_repo.send_message(&Default::default(), &[user_id]).await; WsResponse::ok(action, serde_json::json!({"status":"sent"})) }
        "get_unread" => { let count = state.message_repo.get_unread_count(user_id, None).await.unwrap_or(0); WsResponse::ok(action, serde_json::json!({"count":count})) }
        _ => WsResponse::error(action, "Unknown action"),
    };
    let json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
    sender.send(Message::Text(json.into())).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn ws_status_handler() -> impl IntoResponse { Json(serde_json::json!({"success":true,"data":{"service":"messaging-service","websocket":"enabled"}})) }
