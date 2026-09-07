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
                HeaderValue::from_static("http://localhost:3000"),
                HeaderValue::from_static("http://localhost:8090"),
            ],
        }
    }
}

impl CorsConfig {
    /// 从环境变量创建 CORS 配置（启动时调用）
    pub fn from_env() -> Self {
        let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .map_or_else(|| {
                vec![
                    HeaderValue::from_static("http://localhost:3000"),
                    HeaderValue::from_static("http://localhost:8090"),
                ]
            }, |s| {
                s.split(',')
                    .map(|origin| {
                        origin
                            .trim()
                            .parse::<HeaderValue>()
                            .unwrap_or_else(|_| HeaderValue::from_static("*"))
                    })
                    .collect::<Vec<_>>()
            });
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

    if allowed_origins.len() == 1 && allowed_origins[0] == HeaderValue::from_static("*") {
        cors = cors.allow_origin(Any);
    } else {
        cors = cors.allow_origin(allowed_origins.iter().cloned().collect::<Vec<_>>());
    }

    cors
}
