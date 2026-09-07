//! 公共初始化模块
//!
//! 提供服务初始化、日志设置等公共功能
//!
//! 日志系统使用 log-core crate，支持结构化日志和统一格式
//! 提供 `log::info`!(), `log::error`!() 等宏，兼容现有代码

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// 从 .env 文件加载环境变量（仅用于开发环境）
///
/// 在生产环境中，环境变量应由容器运行时或系统管理。
/// 使用 dotenvy 而非 dotenv，因为 dotenvy 是其活跃的 fork。
pub fn init_env() {
    // 忽略错误，仅在开发环境有 .env 文件时生效
    let _ = dotenvy::dotenv();
}

/// 检测当前环境
const fn detect_env() -> &'static str {
    if cfg!(debug_assertions) {
        "development"
    } else {
        "production"
    }
}

/// 初始化日志系统（统一入口）
///
/// 使用 log-core crate 初始化统一日志系统。
/// - 开发环境：彩色美化输出到控制台
/// - 生产环境：统一 JSON `格式（包含时间戳、级别、服务名、trace_id`）
///
/// 初始化后，代码中应使用 `tracing::info!()`, `tracing::error!()` 等宏。
pub fn init_logging() -> anyhow::Result<()> {
    let env = detect_env();
    log_core::init_logging(env)?;
    tracing::info!("日志系统已初始化: log-core (环境: {env})");
    Ok(())
}

/// 初始化日志系统（带配置）
///
/// 使用自定义配置初始化日志系统。
pub fn init_logging_with_config(config: log_core::LogConfig) -> anyhow::Result<()> {
    log_core::init_with_config(config)?;
    Ok(())
}

/// 初始化日志系统（服务专用）
///
/// 为特定服务初始化日志系统，日志输出到 `logs/{service_name`}/ 目录。
pub fn init_service_logging(service_name: &str) -> anyhow::Result<()> {
    let env = detect_env();
    log_core::init_service_logging(service_name, env)?;
    tracing::info!("日志系统已初始化: {service_name} service");
    Ok(())
}

/// log4rs has been removed - use init_logging() or init_tracing() instead.

/// 刷新日志缓冲区（可选，用于优雅关闭）
pub fn flush_logging() {
    log::logger().flush();
}

/// 初始化服务追踪
///
/// 初始化环境变量、日志系统和 OpenTelemetry，适用于大多数服务。
/// 服务名会传递给 log-core 用于统一 JSON 日志中的 service 字段。
///
/// 在根 span 中统一注入 `service`/`version`/`env` 字段，
/// 确保所有服务的日志都有统一的结构化字段，便于 K8s 日志聚合和过滤。
pub fn init_tracing(service_name: &str) {
    init_env();

    let env = detect_env();
    let version = env!("CARGO_PKG_VERSION");

    // 先设置一个基础的 tracing subscriber 直接输出到 stdout
    // 确保即使 log-core 初始化失败，kubectl logs / Rancher 也能看到日志
    use tracing_subscriber::fmt::time::ChronoLocal;
    let _ = tracing_subscriber::fmt()
        .json()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(ChronoLocal::new("%Y-%m-%dT%H:%M:%S%.3f%:z".to_string()))
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .try_init();
    // 桥接 log crate 到 tracing
    let _ = tracing_log::LogTracer::init();
    
    // 在根 span 中注入统一字段（service/version/env）
    // 确保所有服务的日志都有统一的结构化字段
    let _root_span = tracing::info_span!(
        "service",
        service = service_name,
        version = version,
        env = env,
    );
    tracing::info!(service = %service_name, version = version, env = env, "日志系统初始化完成 (stdout)");

    // 注意：K8s 容器环境使用 stdout 日志即可，由容器运行时自动轮转
    // RKE2 默认配置：containerLogMaxSize=10Mi, containerLogMaxFiles=5
    // 不再尝试初始化 log-core 文件日志，避免冲突和磁盘膨胀
}

/// 初始化日志系统（含可选 OpenTelemetry 层）
///
/// 将 `service_name` 传递给 log-core 的 LogConfig，确保 JSON 日志中包含正确的服务名。
#[allow(dead_code)]
fn init_logging_with_otel(
    otel_layer: Option<Box<dyn tracing_subscriber::layer::Layer<tracing_subscriber::Registry> + Send + Sync + 'static>>,
    service_name: &str,
) -> anyhow::Result<()> {
    let env = detect_env();

    if let Some(layer) = otel_layer {
        log_core::init_with_layers(log_core::LogConfig {
            env: env.to_string(),
            service_name: service_name.to_string(),
            ..Default::default()
        }, layer)?;
    } else {
        log_core::init_with_config(log_core::LogConfig {
            env: env.to_string(),
            service_name: service_name.to_string(),
            ..Default::default()
        })?;
    }

    tracing::info!("日志系统已初始化: log-core (环境: {env}, 服务: {service_name})");
    Ok(())
}

/// 启动服务
pub async fn run_server(
    app: Router,
    addr: SocketAddr,
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("{service_name} starting on {addr}");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// 健康检查响应（JSON 格式，符合 K8s 探针期望）
///
/// 返回 `{ "status": "ok", "service": "<name>" }` 格式。
/// K8s liveness/readiness 探针通常期望 JSON 或 HTTP 200 响应。
#[axum::debug_handler]
pub async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "myai",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
