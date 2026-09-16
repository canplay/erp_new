//! CORS 中间件
//!
//! 基于 `tower_http::cors::CorsLayer` 的跨域配置。
//!
//! # 功能
//! - 从环境变量 `CORS_ALLOWED_ORIGINS` 读取允许的源
//! - 支持通配符 `*`
//! - 暴露所有请求头，允许所有方法
//! - 24 小时预检缓存

use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, AllowMethods, CorsLayer};

use std::time::Duration;

/// CORS 配置
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<HeaderValue>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                HeaderValue::from_static("http://localhost:3000" ),
                HeaderValue::from_static("http://localhost:8090" ),
            ],
        }
    }
}

impl CorsConfig {
    /// 从环境变量创建 CORS 配置（启动时调用）
    ///
    /// 安全修复: 当 CORS_ALLOWED_ORIGINS 未设置时，默认拒绝所有跨域请求（返回空列表），
    /// 而不是允许 localhost。这遵循最小权限原则。
    pub fn from_env() -> Self {
        let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS" )
            .ok()
            .map(|s| {
                s.split(',')
                    .filter_map(|origin| {
                        let trimmed = origin.trim();
                        if trimmed.is_empty() {
                            return None;
                        }
                        // 安全修复: 不再将解析失败回退到通配符 "*"，而是跳过无效源
                        trimmed.parse::<HeaderValue>().ok()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(); // 未设置环境变量时返回空列表（拒绝所有）
        Self { allowed_origins }
    }
}

/// 创建 CORS 层（基于预计算的配置）
pub fn cors_layer(config: &CorsConfig) -> CorsLayer {
    let allowed_origins = &config.allowed_origins;

    let mut cors = CorsLayer::new()
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ]))
        .allow_headers(Any)
        .expose_headers(Any)
        .max_age(Duration::from_hours(24));

    if allowed_origins.len() == 1 && allowed_origins[0] == HeaderValue::from_static("*" ) {
        cors = cors.allow_origin(Any);
    } else {
        cors = cors.allow_origin(allowed_origins.iter().cloned().collect::<Vec<_>>());
    }

    cors
}
