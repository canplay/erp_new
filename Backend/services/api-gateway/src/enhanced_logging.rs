//! Enhanced logging module
//! WebSocket support added

use axum::{
    extract::Request,
    http::{HeaderValue, header::HeaderName},
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use std::hash::{BuildHasher, Hasher};
use std::time::{Duration, Instant};

const TRACE_ID_HEADER: &str = "x-trace-id";
const REQUEST_ID_HEADER: &str = "x-request-id";

#[derive(Debug, Clone, Serialize)]
pub struct RequestLog {
    pub trace_id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessLog {
    pub trace_id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub user_id: Option<String>,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none" )]
    pub content_length: Option<u64>,
    pub duration_ms: u64,
    pub timestamp: String,
    pub service: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum WebSocketLogType {
    Connect,
    Disconnect,
    Message,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebSocketConnectLog {
    pub trace_id: String,
    pub connection_id: String,
    pub client_ip: String,
    pub user_id: Option<String>,
    pub path: String,
    pub protocol: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebSocketMessageLog {
    pub trace_id: String,
    pub connection_id: String,
    pub direction: String,
    pub message_type: String,
    pub size_bytes: usize,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebSocketErrorLog {
    pub trace_id: String,
    pub connection_id: String,
    pub error_type: String,
    pub message: String,
    pub context: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub trace_id: String,
    pub request_id: String,
    pub start_time: Instant,
    pub user_id: Option<String>,
}

impl RequestContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            trace_id: generate_id("trace" ),
            request_id: generate_id("req" ),
            start_time: Instant::now(),
            user_id: None,
        }
    }

    pub fn from_request(request: &Request) -> Self {
        let mut ctx = Self::new();
        if let Some(t) = request.headers().get(TRACE_ID_HEADER)
            && let Ok(s) = t.to_str() {
                ctx.trace_id = s.to_string();
            }
        if let Some(r) = request.headers().get(REQUEST_ID_HEADER)
            && let Ok(s) = r.to_str() {
                ctx.request_id = s.to_string();
            }
        if let Some(u) = request.extensions().get::<String>() {
            ctx.user_id = Some(u.clone());
        }
        ctx
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_id(prefix: &str) -> String {
    use std::collections::hash_map::RandomState;
    use std::time::SystemTime;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let state = RandomState::new();
    let mut h = state.build_hasher();
    h.write_u128(ts);
    let r = h.finish() as u32;
    format!("{}-{}-{:016x}{:08x}" , prefix, 1, ts, r)
}

pub fn get_client_ip(request: &Request) -> Option<String> {
    let ips = [
        "x-forwarded-for" ,
        "x-real-ip" ,
        "cf-connecting-ip" ,
        "true-client-ip" ,
    ];
    for h in ips {
        if let Some(v) = request.headers().get(h)
            && let Ok(s) = v.to_str() {
                let ip = s.split(',').next().unwrap_or(s).trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
    }
    None
}

pub fn get_user_id(request: &Request) -> Option<String> {
    request
        .headers()
        .get("x-user-id" )
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string)
}

pub fn get_user_agent(request: &Request) -> Option<String> {
    request
        .headers()
        .get("user-agent" )
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string)
}

pub fn get_content_length(request: &Request) -> Option<u64> {
    request
        .headers()
        .get("content-length" )
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let t = Instant::now();
    let m = request.method().clone();
    let p = request.uri().path().to_string();
    let tid = common::otel::current_trace_id().or_else(|| {
        request
            .headers()
            .get(TRACE_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string)
    }).unwrap_or_else(|| generate_id("trace" ));
    let rid = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok()).map_or_else(|| generate_id("req" ), std::string::ToString::to_string);
    let cip = get_client_ip(&request);
    let uid = get_user_id(&request);
    tracing::info!(
        "Request start | trace_id={} request_id={} method={} path={} client_ip={} user_id={}" ,
        tid,
        rid,
        m,
        p,
        cip.as_deref().unwrap_or("-" ),
        uid.as_deref().unwrap_or("-" )
    );
    let r = next.run(request).await;
    let dm = t.elapsed().as_millis() as u64;
    let sc = r.status().as_u16();
    match sc {
        200..=299 => tracing::info!(
            "Request complete | trace_id={tid} status={sc} duration_ms={dm}"
        ),
        400..=499 => tracing::warn!(
            "Client error | trace_id={tid} status={sc} duration_ms={dm}"
        ),
        500..=599 => tracing::error!(
            "Server error | trace_id={tid} status={sc} duration_ms={dm}"
        ),
        _ => {}
    }
    if dm > 1000 {
        tracing::warn!(
            "Slow request | trace_id={tid} duration_ms={dm} path={p}"
        );
    }
    r
}

pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let rid = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok()).map_or_else(|| generate_id("req" ), std::string::ToString::to_string);
    let tid = request
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|v| v.to_str().ok()).map_or_else(|| generate_id("trace" ), std::string::ToString::to_string);
    let mut req = request;
    req.extensions_mut().insert(rid.clone());
    req.extensions_mut().insert(tid.clone());
    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    h.insert(
        HeaderName::from_static(REQUEST_ID_HEADER),
        HeaderValue::from_str(&rid).unwrap_or_else(|_| HeaderValue::from_static("" )),
    );
    h.insert(
        HeaderName::from_static(TRACE_ID_HEADER),
        HeaderValue::from_str(&tid).unwrap_or_else(|_| HeaderValue::from_static("" )),
    );
    resp
}

pub async fn health_logging_middleware(request: Request, next: Next) -> Response {
    let t = Instant::now();
    let p = request.uri().path().to_string();
    let r = next.run(request).await;
    let dm = t.elapsed().as_millis() as u64;
    tracing::debug!(
        "health_check | path={} duration_ms={} status={}" ,
        p,
        dm,
        r.status().as_u16()
    );
    r
}

pub fn log_ws_connect(
    trace_id: &str,
    connection_id: &str,
    client_ip: &str,
    user_id: Option<&str>,
    path: &str,
    protocol: &str,
) {
    let l = WebSocketConnectLog {
        trace_id: trace_id.to_string(),
        connection_id: connection_id.to_string(),
        client_ip: client_ip.to_string(),
        user_id: user_id.map(std::string::ToString::to_string),
        path: path.to_string(),
        protocol: protocol.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    if let Ok(s) = serde_json::to_string(&l) {
        tracing::info!("WebSocket connect | {s}" );
    }
}

pub fn log_ws_message(
    trace_id: &str,
    connection_id: &str,
    direction: &str,
    message_type: &str,
    size_bytes: usize,
) {
    let l = WebSocketMessageLog {
        trace_id: trace_id.to_string(),
        connection_id: connection_id.to_string(),
        direction: direction.to_string(),
        message_type: message_type.to_string(),
        size_bytes,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    if let Ok(s) = serde_json::to_string(&l) {
        match direction {
            "inbound" => tracing::debug!("WS inbound | {s}" ),
            "outbound" => tracing::debug!("WS outbound | {s}" ),
            _ => tracing::debug!("WS message | {s}" ),
        }
    }
}

pub fn log_ws_error(
    trace_id: &str,
    connection_id: &str,
    error_type: &str,
    message: &str,
    context: serde_json::Value,
) {
    let l = WebSocketErrorLog {
        trace_id: trace_id.to_string(),
        connection_id: connection_id.to_string(),
        error_type: error_type.to_string(),
        message: message.to_string(),
        context,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    if let Ok(s) = serde_json::to_string(&l) {
        tracing::error!("WebSocket error | {s}" );
    }
}

pub fn get_trace_id(request: &Request) -> Option<String> {
    request.extensions().get::<String>().cloned()
}
