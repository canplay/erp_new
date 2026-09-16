//! 优雅关闭信号模块
//!
//! 提供统一的关闭信号处理，支持：
//! - SIGINT (Ctrl+C)
//! - SIGTERM (docker stop / kubectl delete)
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use crate::shutdown_signal;
//!
//! async fn run_server() {
//!     // 服务运行中...
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     tokio::select! {
//!         _ = run_server() => {},
//!         _ = shutdown_signal() => {
//!             tracing::info!("收到关闭信号，开始优雅关闭..." );
//!         }
//!     }
//! }
//! ```

use tokio::signal;
use tokio::time::{Duration, timeout};

/// 创建优雅关闭信号监听器
///
/// 同时监听 SIGINT 和 SIGTERM，保证容器和本地开发都能正常工作。
///
/// # Arguments
/// * `timeout_secs` - 可选的关闭超时时间，默认 30 秒
///
/// # Returns
/// 等待用户发送中断信号（Ctrl+C 或 kill）
pub async fn shutdown_signal_internal(timeout_secs: u64) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler" );
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler" )
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // 同时监听两种关闭信号
    tokio::select! {
        () = ctrl_c => {
            tracing::info!("收到 Ctrl+C 信号" );
        },
        () = terminate => {
            tracing::info!("收到 SIGTERM 信号" );
        },
    }

    // 等待优雅关闭超时
    tracing::info!("正在等待服务优雅关闭（最多 {timeout_secs} 秒）..." );
    timeout(
        Duration::from_secs(timeout_secs),
        tokio::time::sleep(Duration::from_secs(timeout_secs)),
    )
    .await
    .ok();

    tracing::info!("关闭完成" );
}

/// 默认的 30 秒超时关闭信号
pub async fn shutdown_signal_default() {
    shutdown_signal_internal(30).await;
}

/// 创建带超时参数的优雅关闭信号监听器（推荐使用）
pub async fn shutdown_signal_with_timeout(timeout_secs: u64) {
    shutdown_signal_internal(timeout_secs).await;
}

/// 创建优雅关闭信号监听器（无参数版本，使用默认 30 秒超时）
pub async fn shutdown_signal() {
    shutdown_signal_internal(30).await;
}

/// 创建带自定义消息的 shutdown future
///
/// # 参数
/// - `service_name`: 服务名称，用于日志输出
///
/// # 示例
/// ```rust,ignore
/// use service_core::shutdown_with_name;
///
/// let shutdown = shutdown_with_name("auth-service" );
/// ```
pub async fn shutdown_with_name(service_name: &str) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler" );
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler" )
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("[{service_name}] 收到 Ctrl+C 信号，正在关闭..." );
        }
        () = terminate => {
            tracing::info!("[{service_name}] 收到终止信号，正在关闭..." );
        }
    }
}
