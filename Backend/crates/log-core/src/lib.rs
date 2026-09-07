//! 统一日志库
//!
//! 提供结构化日志、日志格式化和日志级别控制
//! 支持文件、控制台、syslog 多目标分级输出
//!
//! # 日志级别
//! - TRACE: 详细调试信息
//! - DEBUG: 调试信息
//! - INFO: 一般信息
//! - WARN: 警告信息
//! - ERROR: 错误信息
//! - FATAL: 致命错误
//!
//! # 输出目标
//! - console: 控制台输出
//! - file: 文件输出
//! - syslog: 系统日志

use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

static LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new();

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 环境（development/production）
    pub env: String,
    /// 日志目录
    pub log_dir: PathBuf,
    /// 文件名模式
    pub file_name_pattern: String,
    /// 最大文件大小（MB）
    pub max_file_size: u64,
    /// 最大保留天数
    pub max_keep_days: u32,
    /// 最大保留文件数
    pub max_keep_files: u32,
    /// 控制台输出级别
    pub console_level: LogLevel,
    /// 文件输出级别
    pub file_level: LogLevel,
    /// Syslog输出级别
    pub syslog_level: LogLevel,
    /// 是否启用syslog
    pub syslog_enabled: bool,
    /// Syslog服务器地址
    pub syslog_host: Option<String>,
    /// Syslog端口
    pub syslog_port: Option<u16>,
    /// 服务名称（自动从程序名获取）
    pub service_name: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            env: "development".to_string(),
            log_dir: PathBuf::from("logs"),
            file_name_pattern: "{service}.log".to_string(),
            max_file_size: 100,
            max_keep_days: 30,
            max_keep_files: 10,
            console_level: LogLevel::Debug,
            file_level: LogLevel::Info,
            syslog_level: LogLevel::Warn,
            syslog_enabled: false,
            syslog_host: None,
            syslog_port: None,
            service_name: std::env::var("SERVICE_NAME").unwrap_or_else(|_| {
                std::env::args()
                    .next()
                    .unwrap_or_else(|| "unknown".to_string())
            }),
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// 跟踪级别 - 最详细
    Trace,
    /// 调试级别
    Debug,
    /// 信息级别
    Info,
    /// 警告级别
    Warn,
    /// 错误级别
    Error,
    /// 致命级别
    Fatal,
}

impl LogLevel {
    /// 从字符串解析日志级别
    #[must_use]
    pub fn from_str_ignore_case(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warn" | "warning" => Self::Warn,
            "error" => Self::Error,
            "fatal" | "critical" => Self::Fatal,
            _ => Self::Info,
        }
    }

    /// 从环境变量读取日志级别
    #[must_use]
    pub fn from_env(env_var: &str, default: Self) -> Self {
        std::env::var(env_var)
            .ok()
            .map_or(default, |v| Self::from_str_ignore_case(&v))
    }

    /// 转换为 usize（用于比较）
    #[must_use]
    pub const fn as_usize(&self) -> usize {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
            Self::Fatal => 5,
        }
    }

    /// 转换为 log crate 级别
    #[must_use]
    pub const fn to_log_level(&self) -> log::Level {
        match self {
            Self::Trace => log::Level::Trace,
            Self::Debug => log::Level::Debug,
            Self::Info => log::Level::Info,
            Self::Warn => log::Level::Warn,
            Self::Error => log::Level::Error,
            Self::Fatal => log::Level::Error,
        }
    }

    /// 转换为 tracing crate 级别
    #[must_use]
    pub const fn to_tracing_level(&self) -> tracing::Level {
        match self {
            Self::Trace => tracing::Level::TRACE,
            Self::Debug => tracing::Level::DEBUG,
            Self::Info => tracing::Level::INFO,
            Self::Warn => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
            Self::Fatal => tracing::Level::ERROR,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Fatal => write!(f, "FATAL"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_ignore_case(s))
    }
}

/// 日志上下文信息
#[derive(Debug, Clone, Default, Serialize)]
pub struct LogContext {
    /// 请求 ID（追踪ID）
    pub trace_id: Option<String>,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 请求 ID
    pub request_id: Option<String>,
    /// 操作耗时（毫秒）
    pub duration_ms: Option<u64>,
    /// 服务名称
    pub service: Option<String>,
    /// 操作名称
    pub operation: Option<String>,
    /// 客户端IP
    pub client_ip: Option<String>,
    /// 请求方法
    pub method: Option<String>,
    /// 请求路径
    pub path: Option<String>,
    /// HTTP 状态码
    pub status_code: Option<u16>,
    /// 其他上下文
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl LogContext {
    /// 创建新的日志上下文
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 `trace_id`
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// 设置 `user_id`
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// 设置 `request_id`
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// 设置耗时
    #[must_use]
    pub const fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// 设置服务名称
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    /// 设置操作名称
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// 设置客户端IP
    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    /// 设置 HTTP 方法
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// 设置请求路径
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 设置 HTTP 状态码
    #[must_use]
    pub const fn with_status_code(mut self, code: u16) -> Self {
        self.status_code = Some(code);
        self
    }

    /// 添加额外字段
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    /// 添加多个额外字段
    pub fn with_fields(mut self, fields: impl IntoIterator<Item = (String, String)>) -> Self {
        self.extra.extend(fields);
        self
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// 时间戳（ISO 8601格式）
    pub timestamp: String,
    /// 日志级别
    pub level: LogLevel,
    /// 日志目标（模块）
    pub target: String,
    /// 消息
    pub message: String,
    /// 上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<LogContext>,
    /// 源文件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// 源行号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 线程ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// 服务名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: current_timestamp(),
            level,
            target: target.into(),
            message: message.into(),
            context: None,
            file: None,
            line: None,
            thread_id: None,
            service: None,
        }
    }

    /// 设置上下文
    #[must_use]
    pub fn with_context(mut self, ctx: LogContext) -> Self {
        self.context = Some(ctx);
        self
    }

    /// 设置源码位置
    pub fn with_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    /// 设置线程ID
    pub fn with_thread_id(mut self, thread_id: impl Into<String>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    /// 设置服务名称
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    /// 序列化为 JSON 字符串
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.message.clone())
    }

    /// 序列化为美化格式
    #[must_use]
    pub fn to_pretty(&self) -> String {
        let ctx_str = if let Some(ctx) = &self.context {
            let mut parts = Vec::new();
            if let Some(trace_id) = &ctx.trace_id {
                parts.push(format!("trace_id={trace_id}"));
            }
            if let Some(user_id) = &ctx.user_id {
                parts.push(format!("user_id={user_id}"));
            }
            if let Some(request_id) = &ctx.request_id {
                parts.push(format!("request_id={request_id}"));
            }
            if let Some(duration_ms) = ctx.duration_ms {
                parts.push(format!("duration={duration_ms}ms"));
            }
            if let Some(operation) = &ctx.operation {
                parts.push(format!("op={operation}"));
            }
            for (k, v) in &ctx.extra {
                parts.push(format!("{k}={v}"));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" [{}]", parts.join(" "))
            }
        } else {
            String::new()
        };

        let loc_str = if let (Some(file), Some(line)) = (&self.file, self.line) {
            format!(" ({file}:{line})")
        } else {
            String::new()
        };

        let service_str = if let Some(service) = &self.service {
            format!("[{service}] ")
        } else {
            String::new()
        };

        format!(
            "{} {}{}{}{}{}",
            self.timestamp, service_str, self.level, self.target, ctx_str, loc_str
        )
    }
}

/// 初始化日志系统（兼容旧API）
pub fn init_logging(env: &str) -> anyhow::Result<()> {
    let config = LogConfig {
        env: env.to_string(),
        ..Default::default()
    };
    init_with_config(config)
}

/// 使用配置初始化日志系统
pub fn init_with_config(config: LogConfig) -> anyhow::Result<()> {
    init_with_layers(config, None::<tracing_subscriber::layer::Identity>)
}

/// 使用配置 + 额外追踪层初始化日志系统
///
/// 支持注入 OpenTelemetry 等自定义 Layer 到 tracing 管线中。
/// `extra_layer` 会被组合在 fmt 层之上。
pub fn init_with_layers<L>(config: LogConfig, extra_layer: L) -> anyhow::Result<()>
where
    L: tracing_subscriber::layer::Layer<tracing_subscriber::Registry> + Send + Sync + 'static,
{
    // 存储配置
    let service_name = config.service_name.clone();
    let _ = LOG_CONFIG.set(config.clone());

    // 构建环境过滤器
    let env_filter = if config.env == "production" {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
    };

    // 创建日志目录
    if !config.log_dir.exists() {
        std::fs::create_dir_all(&config.log_dir)?;
    }

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    // EnvFilter 作为独立 Layer 添加，避免 Filtered 层对特定 Subscriber 类型的约束
    if config.env == "production" {
        // 生产环境：JSON格式输出到文件 + stdout（容器日志）
        let file_appender = tracing_appender::rolling::Builder::new()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .max_log_files(config.max_keep_files as usize)
            .filename_suffix("log")
            .build(&config.log_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create log directory: {e}"))?;

        let (non_blocking_file, guard_file) = tracing_appender::non_blocking(file_appender);

        // stdout 层（用于 kubectl logs / Rancher 日志查看）
        let subscriber = {
            // 文件日志：JSON 格式
            let file_layer = tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format::json())
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .with_writer(non_blocking_file);

            // stdout 日志（kubectl logs / Rancher）
            let stdout_layer = tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format::json())
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_ansi(false)
                .with_writer(|| std::io::stdout());

            Registry::default()
                .with(extra_layer)
                .with(file_layer)
                .with(stdout_layer)
                .with(env_filter)
        };

        tracing::subscriber::set_global_default(subscriber)?;

        // 持有文件 guard 防止日志写入线程被提前释放
        let _ = LOG_GUARDS.set(vec![guard_file]);
    } else {
        // 开发环境：彩色美化输出到控制台
        let subscriber = {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(true)
                .pretty();

            Registry::default()
                .with(extra_layer)
                .with(fmt_layer)
                .with(env_filter)
        };

        tracing::subscriber::set_global_default(subscriber)?;
    }

    // 将 log crate 桥接到 tracing subscriber（需在 set_global_default 之后）
    let _ = tracing_log::LogTracer::init();

    // 记录初始化信息
    tracing::info!(
        target: "log_core",
        "{{\"service\":\"{}\",\"event\":\"logging_initialized\",\"env\":\"{}\",\"log_dir\":\"{}\"}}",
        service_name, config.env, config.log_dir.display()
    );

    Ok(())
}

/// 持有 tracing_appender 的 guard，防止日志写入线程被提前释放
static LOG_GUARDS: OnceLock<Vec<tracing_appender::non_blocking::WorkerGuard>> = OnceLock::new();

/// 获取当前时间戳
#[must_use]
pub fn current_timestamp() -> String {
    chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S%.3f")
        .to_string()
}

/// 获取当前 UTC 时间戳
#[must_use]
pub fn current_utc_timestamp() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// 获取 ISO 8601 格式时间戳
#[must_use]
pub fn iso_timestamp() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// 格式化持续时间
#[must_use]
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        format!("{:.2}m", ms as f64 / 60000.0)
    }
}

/// 获取日志配置（如果已初始化）
pub fn get_log_config() -> Option<&'static LogConfig> {
    LOG_CONFIG.get()
}

// =============================================================================
// 日志宏定义
// =============================================================================

/// 记录结构化日志的宏
#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $target:expr, $message:expr, $context:expr) => {
        let ctx = $context;
        let entry = log_core::LogEntry::new($level, $target, $message)
            .with_context(ctx)
            .with_thread_id(format!("{:?}", std::thread::current().id()));

        match $level {
            log_core::LogLevel::Trace | log_core::LogLevel::Debug => {
                tracing::debug!(target: $target, "{}", entry.to_json());
            }
            log_core::LogLevel::Info => {
                tracing::info!(target: $target, "{}", entry.to_json());
            }
            log_core::LogLevel::Warn => {
                tracing::warn!(target: $target, "{}", entry.to_json());
            }
            log_core::LogLevel::Error | log_core::LogLevel::Fatal => {
                tracing::error!(target: $target, "{}", entry.to_json());
            }
        }
    };
}

/// 快速记录日志的宏（带上下文）
#[macro_export]
macro_rules! log_info {
    ($target:expr, $message:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new();
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*
        let entry = log_core::LogEntry::new(log_core::LogLevel::Info, $target, $message)
            .with_context(ctx);
        tracing::info!(target: $target, "{}", entry.to_json());
    };
}

/// 调试日志宏
#[macro_export]
macro_rules! log_debug {
    ($target:expr, $message:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new();
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*
        let entry = log_core::LogEntry::new(log_core::LogLevel::Debug, $target, $message)
            .with_context(ctx);
        tracing::debug!(target: $target, "{}", entry.to_json());
    };
}

/// 警告日志宏
#[macro_export]
macro_rules! log_warn {
    ($target:expr, $message:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new();
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*
        let entry = log_core::LogEntry::new(log_core::LogLevel::Warn, $target, $message)
            .with_context(ctx);
        tracing::warn!(target: $target, "{}", entry.to_json());
    };
}

/// 错误日志宏
#[macro_export]
macro_rules! log_error {
    ($target:expr, $message:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new();
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*
        let entry = log_core::LogEntry::new(log_core::LogLevel::Error, $target, $message)
            .with_context(ctx);
        tracing::error!(target: $target, "{}", entry.to_json());
    };
}

/// 请求日志宏（自动记录请求信息）
#[macro_export]
macro_rules! log_request {
    ($target:expr, $method:expr, $path:expr, $status:expr, $duration_ms:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new()
            .with_field("method", $method)
            .with_field("path", $path)
            .with_field("status", format!("{}", $status))
            .with_duration_ms($duration_ms);
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*
        let level = if $status >= 500 {
            log_core::LogLevel::Error
        } else if $status >= 400 {
            log_core::LogLevel::Warn
        } else {
            log_core::LogLevel::Info
        };
        let entry = log_core::LogEntry::new(level, $target, format!("HTTP {} {} -> {}", $method, $path, $status))
            .with_context(ctx);
        match level {
            log_core::LogLevel::Warn => tracing::warn!(target: $target, "{}", entry.to_json()),
            log_core::LogLevel::Error => tracing::error!(target: $target, "{}", entry.to_json()),
            _ => tracing::info!(target: $target, "{}", entry.to_json()),
        }
    };
}

/// 性能日志宏
#[macro_export]
macro_rules! log_performance {
    ($target:expr, $operation:expr, $duration_ms:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new()
            .with_operation($operation)
            .with_duration_ms($duration_ms);
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*

        let level = if $duration_ms > 1000 {
            log_core::LogLevel::Warn
        } else {
            log_core::LogLevel::Info
        };

        let duration_str = log_core::format_duration($duration_ms);
        let entry = log_core::LogEntry::new(level, $target, format!("Performance: {} took {}", $operation, duration_str))
            .with_context(ctx);

        match level {
            log_core::LogLevel::Warn => tracing::warn!(target: $target, "{}", entry.to_json()),
            _ => tracing::info!(target: $target, "{}", entry.to_json()),
        }
    };
}

/// 安全日志宏
#[macro_export]
macro_rules! log_security {
    ($target:expr, $action:expr, $result:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new()
            .with_field("action", $action)
            .with_field("result", $result);
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*

        let level = match $result {
            "success" | "allowed" => log_core::LogLevel::Info,
            "failure" | "denied" | "blocked" => log_core::LogLevel::Warn,
            _ => log_core::LogLevel::Info,
        };

        let entry = log_core::LogEntry::new(level, $target, format!("Security: {} - {}", $action, $result))
            .with_context(ctx);

        match level {
            log_core::LogLevel::Warn => tracing::warn!(target: $target, "{}", entry.to_json()),
            _ => tracing::info!(target: $target, "{}", entry.to_json()),
        }
    };
}

/// gRPC 日志宏
#[macro_export]
macro_rules! log_grpc {
    ($target:expr, $method:expr, $service:expr, $duration_ms:expr, $status:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new()
            .with_operation($method)
            .with_service($service)
            .with_field("grpc_method", $method)
            .with_field("grpc_service", $service)
            .with_field("status", $status)
            .with_duration_ms($duration_ms);
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*

        let level = match $status {
            "OK" | "0" => log_core::LogLevel::Info,
            _ => log_core::LogLevel::Warn,
        };

        let duration_str = log_core::format_duration($duration_ms);
        let entry = log_core::LogEntry::new(level, $target, format!("gRPC {} on {} took {} ({})", $method, $service, duration_str, $status))
            .with_context(ctx);

        match level {
            log_core::LogLevel::Warn => tracing::warn!(target: $target, "{}", entry.to_json()),
            _ => tracing::info!(target: $target, "{}", entry.to_json()),
        }
    };
}

/// 数据库日志宏
#[macro_export]
macro_rules! log_sql {
    ($target:expr, $query:expr, $duration_ms:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new()
            .with_operation("sql_query")
            .with_field("sql", $query)
            .with_duration_ms($duration_ms);
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*

        let level = if $duration_ms > 500 {
            log_core::LogLevel::Warn
        } else {
            log_core::LogLevel::Debug
        };

        let duration_str = log_core::format_duration($duration_ms);
        let entry = log_core::LogEntry::new(level, $target, format!("SQL query took {}: {}", duration_str, $query))
            .with_context(ctx);

        match level {
            log_core::LogLevel::Warn => tracing::warn!(target: $target, "{}", entry.to_json()),
            _ => tracing::debug!(target: $target, "{}", entry.to_json()),
        }
    };
}

/// 服务启动日志宏
#[macro_export]
macro_rules! log_service_start {
    ($target:expr, $service:expr, $host:expr, $port:expr) => {
        let entry = log_core::LogEntry::new(
            log_core::LogLevel::Info,
            $target,
            format!("{} service started on {}:{}", $service, $host, $port)
        ).with_service($service);
        tracing::info!(target: $target, "{}", entry.to_json());
    };
}

/// 服务关闭日志宏
#[macro_export]
macro_rules! log_service_stop {
    ($target:expr, $service:expr) => {
        let entry = log_core::LogEntry::new(
            log_core::LogLevel::Info,
            $target,
            format!("{} service stopped", $service)
        ).with_service($service);
        tracing::info!(target: $target, "{}", entry.to_json());
    };
}

/// 熔断器状态日志宏
#[macro_export]
macro_rules! log_circuit_breaker {
    ($target:expr, $service:expr, $action:expr, $state:expr $(, $key:ident = $value:expr)*) => {
        let mut ctx = log_core::LogContext::new()
            .with_service($service)
            .with_field("action", $action)
            .with_field("state", $state);
        $(ctx = ctx.with_field(stringify($key), format!("{:?}", $value));)*

        let level = match $state {
            "open" | "forced_open" => log_core::LogLevel::Warn,
            "half_open" => log_core::LogLevel::Info,
            _ => log_core::LogLevel::Debug,
        };

        let entry = log_core::LogEntry::new(level, $target, format!("Circuit breaker {} on {}: {}", $action, $service, $state))
            .with_context(ctx);

        match level {
            log_core::LogLevel::Warn => tracing::warn!(target: $target, "{}", entry.to_json()),
            _ => tracing::info!(target: $target, "{}", entry.to_json()),
        }
    };
}

// =============================================================================
// 服务日志初始化助手
// =============================================================================

/// 为服务初始化日志系统
pub fn init_service_logging(service_name: &str, env: &str) -> anyhow::Result<()> {
    let config = LogConfig {
        env: env.to_string(),
        log_dir: PathBuf::from(format!("logs/{service_name}")),
        file_name_pattern: "{service}.log"
            .to_string()
            .replace("{service}", service_name),
        service_name: service_name.to_string(),
        ..Default::default()
    };
    init_with_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context() {
        let ctx = LogContext::new()
            .with_trace_id("trace-123")
            .with_user_id("user-456")
            .with_duration_ms(100)
            .with_field("action", "login")
            .with_client_ip("192.0.2.1");

        assert_eq!(ctx.trace_id, Some("trace-123".to_string()));
        assert_eq!(ctx.user_id, Some("user-456".to_string()));
        assert_eq!(ctx.duration_ms, Some(100));
        assert_eq!(ctx.extra.get("action"), Some(&"login".to_string()));
        assert_eq!(ctx.client_ip, Some("192.0.2.1".to_string()));
    }

    #[test]
    fn test_log_entry() {
        let ctx = LogContext::new()
            .with_user_id("user-123")
            .with_duration_ms(50);

        let entry = LogEntry::new(LogLevel::Info, "test_module", "Test message")
            .with_context(ctx)
            .with_location("test.rs", 42);

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.target, "test_module");
        assert_eq!(entry.message, "Test message");
        assert!(entry.context.is_some());

        // 测试 JSON 序列化
        let json = entry.to_json();
        assert!(json.contains("test_module"));
        assert!(json.contains("Test message"));

        // 测试美化输出
        let pretty = entry.to_pretty();
        assert!(pretty.contains("INFO"));
        assert!(pretty.contains("test_module"));
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str_ignore_case("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str_ignore_case("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str_ignore_case("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str_ignore_case("warning"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str_ignore_case("error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str_ignore_case("fatal"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str_ignore_case("unknown"), LogLevel::Info); // 默认值
    }

    #[test]
    fn test_duration_format() {
        assert_eq!(format_duration(50), "50ms");
        assert_eq!(format_duration(1000), "1.00s");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(60000), "1.00m");
        assert_eq!(format_duration(90000), "1.50m");
    }

    #[test]
    fn test_timestamp() {
        let ts = current_timestamp();
        assert!(!ts.is_empty());

        let iso = iso_timestamp();
        assert!(iso.ends_with('Z'));
    }
}
