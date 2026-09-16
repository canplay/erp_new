//! OpenTelemetry 初始化与追踪工具
//!
//! 提供统一的 OpenTelemetry 初始化，支持通过环境变量配置：
//! - `OTEL_ENABLED`: 是否启用 OpenTelemetry（默认 `true`）
//! - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP 导出端点（默认 `http://localhost:4317`）
//! - `OTEL_SERVICE_NAME`: 服务名称（默认由调用方传入）

use opentelemetry::trace::{SpanId, TraceContextExt, TraceId, TracerProvider};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::OnceLock;
use tracing_opentelemetry::OpenTelemetrySpanExt;

static OTEL_GUARD: OnceLock<OtelGuard> = OnceLock::new();

/// OpenTelemetry 生命周期守卫
///
/// 保持此对象存活以确保 `OTel` 数据正确导出。
/// 在服务关闭时调用 `shutdown()` 方法。
pub struct OtelGuard {
    tracer_provider: Option<SdkTracerProvider>,
}

impl OtelGuard {
    const fn new(provider: Option<SdkTracerProvider>) -> Self {
        Self {
            tracer_provider: provider,
        }
    }

    /// 关闭 OpenTelemetry，刷新所有尚未导出的 Span
    pub fn shutdown(&self) {
        if let Some(provider) = &self.tracer_provider
            && let Err(e) = provider.shutdown() {
                tracing::warn!("OpenTelemetry provider shutdown error: {e}" );
            }
    }
}

/// 初始化 OpenTelemetry 并返回 tracing Layer
///
/// 根据环境变量决定是否实际初始化：
/// - `OTEL_ENABLED=false` 时返回 `None`
/// - 否则配置 OTLP 导出器并返回 `Some(layer)`
///
/// # 参数
/// - `service_name`: 服务名称，会设置为 `service.name` 资源属性
pub fn init_otel(
    service_name: &str,
) -> Option<Box<dyn tracing_subscriber::layer::Layer<tracing_subscriber::Registry> + Send + Sync + 'static>>
{
    let enabled = std::env::var("OTEL_ENABLED" )
        .ok()
        .is_none_or(|v| v == "true" || v == "1" );

    if !enabled {
        tracing::info!("OpenTelemetry 已禁用 (OTEL_ENABLED=false)" );
        return None;
    }

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT" )
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    tracing::info!("OpenTelemetry 初始化中, 端点: {endpoint}" );

    let tracer_provider = match build_tracer_provider(service_name, &endpoint) {
        Ok(provider) => provider,
        Err(e) => {
            tracing::warn!(
                "OpenTelemetry 初始化失败: {e}, 将使用无追踪模式运行"
            );
            return None;
        }
    };

    let tracer = tracer_provider.tracer(service_name.to_string());

    let _ = OTEL_GUARD.set(OtelGuard::new(Some(tracer_provider)));

    Some(Box::new(tracing_opentelemetry::layer().with_tracer(tracer)))
}

/// 构建 `TracerProvider`
fn build_tracer_provider(
    service_name: &str,
    endpoint: &str,
) -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attribute(KeyValue::new("service.name" , service_name.to_string()))
                .with_attribute(KeyValue::new("service.version" , env!("CARGO_PKG_VERSION" )))
                .build(),
        )
        .build();

    Ok(provider)
}

/// 获取当前 trace ID 的十六进制字符串
///
/// 如果当前没有活跃的追踪上下文，返回 `None`。
#[must_use]
pub fn current_trace_id() -> Option<String> {
    let span = tracing::Span::current();
    let ctx = span.context();
    let span_ref = ctx.span();
    let span_context = span_ref.span_context();
    let trace_id = span_context.trace_id();
    if trace_id == TraceId::INVALID {
        None
    } else {
        Some(format!("{trace_id:032x}" ))
    }
}

/// 获取当前 span ID 的十六进制字符串
#[must_use]
pub fn current_span_id() -> Option<String> {
    let span = tracing::Span::current();
    let ctx = span.context();
    let span_ref = ctx.span();
    let span_context = span_ref.span_context();
    let span_id = span_context.span_id();
    if span_id == SpanId::INVALID {
        None
    } else {
        Some(format!("{span_id:016x}" ))
    }
}

/// 检查 OpenTelemetry 当前是否已启用
pub fn is_otel_enabled() -> bool {
    OTEL_GUARD
        .get()
        .is_some_and(|g| g.tracer_provider.is_some())
}

/// 手动关闭 OpenTelemetry（通常在服务关闭时调用）
pub fn shutdown_otel() {
    if let Some(guard) = OTEL_GUARD.get() {
        guard.shutdown();
    }
}

/// 追踪上下文信息（用于 /trace 端点）
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: Option<String>,
    pub service_name: String,
    pub otel_enabled: bool,
}

/// 获取当前追踪上下文信息
#[must_use]
pub fn get_trace_context(service_name: &str) -> TraceContext {
    TraceContext {
        trace_id: current_trace_id().unwrap_or_else(|| "none".to_string()),
        span_id: current_span_id(),
        service_name: service_name.to_string(),
        otel_enabled: is_otel_enabled(),
    }
}
