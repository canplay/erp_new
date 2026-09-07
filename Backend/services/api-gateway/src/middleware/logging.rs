//! 请求日志中间件
//!
//! 提供详细的请求追踪、性能监控和结构化日志。
//!
//! # 功能
//! - 从 OpenTelemetry 获取 trace ID
//! - 统一获取客户端真实 IP（支持多 header 降级）
//! - 按状态码级别记录日志（info/warn/error）
//! - 慢请求告警（>1s）

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// 日志中间件 - 增强版
///
/// 提供详细的请求追踪、性能监控和结构化日志。
/// 自动从 OpenTelemetry 获取 trace ID，如果 `OTel` 未启用则回退到 x-trace-id 头。
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let _query = request.uri().query().map(std::string::ToString::to_string);
    let start_time = Instant::now();

    // 从 OpenTelemetry 获取 trace ID
    let trace_id = common::otel::current_trace_id().or_else(|| {
        request
            .headers()
            .get("x-trace-id")
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string)
    })
    .unwrap_or_else(|| {
        format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        )
    });

    // 获取客户端 IP
    // P5修复: 使用 enhanced_logging::get_client_ip 统一获取客户端真实IP（支持多header降级）
    let client_ip = crate::enhanced_logging::get_client_ip(&request)
        .map(|s| s.split(',').next().unwrap_or(&s).trim().to_string());

    // 获取用户 ID
    let user_id = request
        .headers()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    // 记录请求开始
    tracing::info!(
        "→ 请求开始 | trace_id={trace_id} method={method} path={path} client_ip={client_ip:?} user_id={user_id:?}"
    );

    // 执行处理器
    let response = next.run(request).await;

    // 计算耗时
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let status = response.status();
    let status_code = status.as_u16();

    // 根据状态码选择日志级别
    match status_code {
        200..=299 => {
            tracing::info!(
                "← 请求完成 | trace_id={trace_id} status={status_code} duration_ms={duration_ms}"
            );
        }
        400..=499 => {
            tracing::warn!(
                "← 客户端错误 | trace_id={trace_id} status={status_code} duration_ms={duration_ms}"
            );
        }
        500..=599 => {
            tracing::error!(
                "← 服务端错误 | trace_id={trace_id} status={status_code} duration_ms={duration_ms}"
            );
        }
        _ => {}
    }

    // 慢请求警告 (超过 1 秒)
    if duration_ms > 1000 {
        tracing::warn!(
            "⚠️ 慢请求 | trace_id={trace_id} duration_ms={duration_ms} path={path}"
        );
    }

    response
}
