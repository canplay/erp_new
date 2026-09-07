//! 应用状态管理模块
//!
//! 提供统一的 gRPC 和 HTTP 应用状态类型，简化服务开发。
//!
//! # 设计理念
//!
//! - `AppState<T>`: gRPC 服务状态，可包含任意业务数据
//! - `HttpAppState<T>`: HTTP 服务状态，包装 gRPC 状态并添加 HTTP 特有字段
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use common::app_state::{AppState, HttpAppState};
//! use std::sync::Arc;
//!
//! // 创建 gRPC 状态
//! let grpc_state = Arc::new(AppState::new(()));
//!
//! // 创建 HTTP 状态
//! let http_state = HttpAppState::from_inner(grpc_state.clone());
//! ```

use std::sync::Arc;

/// gRPC 服务应用状态
///
/// 用于在 gRPC 服务处理器之间共享数据。
/// 可以包含数据库连接池、JWT 服务、业务逻辑等。
///
#[derive(Debug, Clone)]
pub struct AppState<T = ()> {
    /// 内部数据
    inner: T,
    /// 服务版本
    pub version: String,
}

impl<T> AppState<T> {
    /// 创建新的应用状态
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// 获取内部数据的引用
    pub const fn inner(&self) -> &T {
        &self.inner
    }

    /// 获取内部数据的可变引用
    pub const fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// 获取内部数据的所有权（克隆）
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// 将状态转换为 Arc
    pub fn into_arc(self) -> Arc<Self>
    where
        T: Clone,
    {
        Arc::new(self)
    }
}

impl<T> AppState<T>
where
    T: Clone,
{
    /// 从已有的 Arc 创建新的 `AppState`
    pub fn from_arc(inner: Arc<T>) -> Self {
        Self {
            inner: (*inner).clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl AppState<()> {
    /// 创建空的默认状态
    #[must_use]
    pub fn default_state() -> Self {
        Self {
            inner: (),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// HTTP 服务应用状态
///
/// 包装 gRPC 状态并添加 HTTP 特有字段。
/// 用于在 HTTP 处理器之间共享数据。
#[derive(Debug, Clone)]
pub struct HttpAppState<T = ()> {
    /// 底层 gRPC 状态
    pub inner: Arc<AppState<T>>,
    /// 追踪 ID（用于请求跟踪）
    pub trace_id: Option<String>,
    /// 请求时间戳
    pub request_id: Option<u64>,
}

impl<T> HttpAppState<T> {
    /// 从 gRPC 状态创建 HTTP 状态
    pub const fn from_inner(inner: Arc<AppState<T>>) -> Self {
        Self {
            inner,
            trace_id: None,
            request_id: None,
        }
    }

    /// 创建新的 HTTP 状态
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(AppState::new(inner)),
            trace_id: None,
            request_id: None,
        }
    }

    /// 获取 gRPC 状态的引用
    #[must_use]
    pub const fn grpc_state(&self) -> &Arc<AppState<T>> {
        &self.inner
    }
}

impl<T: Clone> From<Arc<AppState<T>>> for HttpAppState<T> {
    fn from(inner: Arc<AppState<T>>) -> Self {
        Self::from_inner(inner)
    }
}

/// 状态提取器用于 Axum
///
/// 使用方法：
/// ```rust
/// async fn handler(
///     State(state): State<Arc<AppState<MyState>>>
/// ) -> Result<Json<Response>, AppError> {
///     // 使用 state.inner() 获取业务数据
/// }
/// ```
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_basic() {
        let state = AppState::new(42);
        assert_eq!(*state.inner(), 42);
    }

    #[test]
    fn test_http_app_state() {
        let inner = Arc::new(AppState::new(100));
        let http_state = HttpAppState::from_inner(inner);
        assert_eq!(*http_state.inner.inner(), 100);
    }
}
