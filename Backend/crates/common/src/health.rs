//! 健康检查模块
//!
//! 提供标准的健康检查端点，支持：
//! - HTTP 健康检查端点
//! - Kubernetes livenessProbe 和 readinessProbe
//! - 数据库连接健康检查（含每个服务的连接状态）
//! - Redis 连接健康检查
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use crate::health::health_routes;
//! use axum::Router;
//!
//! let app = Router::new().merge(health_routes());
//! // 带数据库健康检查：
//! let app = Router::new().merge(health_routes_with_db(pool));
//! ```
//!
//! # 端点
//!
//! - `/health` - 基础健康检查
//! - `/health/live` - Kubernetes livenessProbe
//! - `/health/ready` - Kubernetes readinessProbe
//! - `/health/database` - 数据库连接健康检查（需传入连接池）
//! - `/health/redis` - Redis 连接健康检查（需传入连接状态）

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;

/// 健康状态信息
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: String,
}

/// 数据库连接健康状态
#[derive(Debug, Clone, Serialize)]
pub struct DbConnectionHealth {
    pub service: String,
    pub status: String,
    pub latency_ms: u64,
    pub schema: Option<String>,
    pub pool_size: Option<u32>,
    pub error: Option<String>,
}

/// Redis 连接健康状态
#[derive(Debug, Clone, Serialize)]
pub struct RedisConnectionHealth {
    pub service: String,
    pub status: String,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// 数据库健康检查响应
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseHealth {
    pub status: String,
    pub connections: Vec<DbConnectionHealth>,
}

/// Redis 健康检查响应
#[derive(Debug, Clone, Serialize)]
pub struct RedisHealth {
    pub status: String,
    pub connections: Vec<RedisConnectionHealth>,
}

/// 数据库健康检查状态类型
#[derive(Clone)]
pub enum DbHealthState {
    /// 拥有数据库连接池的服务
    Pooled(sqlx::PgPool),
    /// 无数据库连接的服务
    None,
}

/// Redis 健康检查状态类型
#[derive(Clone)]
pub enum RedisHealthState {
    /// 拥有 Redis 连接的服务
    Connected(String),
    /// 无 Redis 连接的服务
    None,
}

impl DbHealthState {
    /// 检查数据库连接健康
    pub async fn check(&self, service_name: &str) -> DbConnectionHealth {
        match self {
            Self::Pooled(pool) => {
                let start = Instant::now();
                match sqlx::raw_sql("SELECT 1" ).execute(pool).await {
                    Ok(_) => {
                        let latency = start.elapsed().as_millis() as u64;
                        DbConnectionHealth {
                            service: service_name.to_string(),
                            status: "healthy".to_string(),
                            latency_ms: latency,
                            schema: std::env::var("SERVICE_SCHEMA" ).ok(),
                            pool_size: Some(pool.size()),
                            error: None,
                        }
                    }
                    Err(e) => {
                        let latency = start.elapsed().as_millis() as u64;
                        DbConnectionHealth {
                            service: service_name.to_string(),
                            status: "unhealthy".to_string(),
                            latency_ms: latency,
                            schema: std::env::var("SERVICE_SCHEMA" ).ok(),
                            pool_size: None,
                            error: Some(e.to_string()),
                        }
                    }
                }
            }
            Self::None => DbConnectionHealth {
                service: service_name.to_string(),
                status: "not_configured".to_string(),
                latency_ms: 0,
                schema: None,
                pool_size: None,
                error: None,
            },
        }
    }
}

impl RedisHealthState {
    /// 检查 Redis 连接健康
    pub async fn check(&self, service_name: &str) -> RedisConnectionHealth {
        match self {
            Self::Connected(url) => {
                let start = Instant::now();
                let url = url.clone();
                // 实际连接 Redis 并执行 PING 命令
                let result = tokio::spawn(async move {
                    let client = match redis::Client::open(url.as_str()) {
                        Ok(c) => c,
                        Err(e) => return Err::<(), Box<dyn std::error::Error + Send + Sync>>(format!("Redis client error: {e}" ).into())
                    };
                    let mut conn = match client.get_multiplexed_async_connection().await {
                        Ok(c) => c,
                        Err(e) => return Err(format!("Redis connect error: {e}" ).into())
                    };
                    let _: () = match redis::cmd("PING" ).query_async(&mut conn).await {
                        Ok(r) => r,
                        Err(e) => return Err(format!("Redis PING error: {e}" ).into())
                    };
                    Ok(())
                })
                .await;
                
                let latency = start.elapsed().as_millis() as u64;
                
                match result {
                    Ok(Ok(())) => RedisConnectionHealth {
                        service: service_name.to_string(),
                        status: "healthy".to_string(),
                        latency_ms: latency,
                        error: None,
                    },
                    Ok(Err(e)) => RedisConnectionHealth {
                        service: service_name.to_string(),
                        status: "unhealthy".to_string(),
                        latency_ms: latency,
                        error: Some(e.to_string()),
                    },
                    Err(e) => RedisConnectionHealth {
                        service: service_name.to_string(),
                        status: "error".to_string(),
                        latency_ms: latency,
                        error: Some(e.to_string()),
                    },
                }
            }
            Self::None => RedisConnectionHealth {
                service: service_name.to_string(),
                status: "not_configured".to_string(),
                latency_ms: 0,
                error: None,
            },
        }
    }
}

/// 创建 HTTP 健康检查路由（扁平式，兼容 K8s probes）
///
/// - `GET /health` - 存活检查（livenessProbe），返回服务基本信息
/// - `GET /ready` - 就绪检查（readinessProbe），确认服务可接收流量
/// - `GET /health/database` - 数据库健康检查（可选，需传入连接池）
/// - `GET /health/redis` - Redis 健康检查（可选，需传入连接状态）
pub fn health_routes() -> Router {
    Router::new()
        .route("/health" , get(liveness_handler))
        .route("/ready" , get(readiness_handler))
}

/// 创建带数据库和 Redis 健康检查的 HTTP 路由
pub fn health_routes_with_db_and_redis(
    db_state: DbHealthState,
    redis_state: RedisHealthState,
) -> Router {
    let db_state = Arc::new(db_state);
    let redis_state = Arc::new(redis_state);
    Router::new()
        .route("/health" , get(liveness_handler))
        .route("/ready" , get(readiness_handler))
        .route("/health/database" , get(database_health_handler))
        .with_state(db_state)
        .route("/health/redis" , get(redis_health_handler))
        .with_state(redis_state)
}

/// 健康检查处理器
#[allow(dead_code)]
async fn health_handler() -> impl IntoResponse {
    let status = HealthStatus {
        status: "healthy".to_string(),
        service: std::env::var("SERVICE_NAME" ).unwrap_or_else(|_| "unknown".to_string()),
        version: std::env::var("SERVICE_VERSION" )
            .unwrap_or_else(|_| env!("CARGO_PKG_VERSION" ).to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    (axum::http::StatusCode::OK, Json(status)).into_response()
}

/// 存活探针检查
async fn liveness_handler() -> &'static str {
    "OK"
}

/// 就绪探针检查
async fn readiness_handler() -> &'static str {
    "OK"
}

/// 数据库健康检查处理器
async fn database_health_handler(
    State(state): State<Arc<DbHealthState>>,
) -> Json<DatabaseHealth> {
    let service_name = std::env::var("SERVICE_NAME" ).unwrap_or_else(|_| "unknown".to_string());
    let conn_health = state.check(&service_name).await;

    let status = match conn_health.status.as_str() {
        "healthy" => "healthy" ,
        "not_configured" => "not_configured" ,
        _ => "degraded" ,
    };

    Json(DatabaseHealth {
        status: status.to_string(),
        connections: vec![conn_health],
    })
}

/// Redis 健康检查处理器
async fn redis_health_handler(
    State(state): State<Arc<RedisHealthState>>,
) -> Json<RedisHealth> {
    let service_name = std::env::var("SERVICE_NAME" ).unwrap_or_else(|_| "unknown".to_string());
    let conn_health = state.check(&service_name).await;

    let status = match conn_health.status.as_str() {
        "healthy" => "healthy" ,
        "not_configured" => "not_configured" ,
        _ => "degraded" ,
    };

    Json(RedisHealth {
        status: status.to_string(),
        connections: vec![conn_health],
    })
}

/// 为多服务场景创建数据库健康检查
pub async fn check_all_databases(
    connections: Vec<(&str, DbHealthState)>,
) -> DatabaseHealth {
    let mut results = Vec::new();
    for (service_name, state) in connections {
        let health = state.check(service_name).await;
        results.push(health);
    }

    let all_healthy = results.iter().all(|c| c.status == "healthy" );

    let status = if all_healthy {
        "healthy"
    } else {
        "degraded"
    };

    DatabaseHealth {
        status: status.to_string(),
        connections: results,
    }
}

/// 为多服务场景创建 Redis 健康检查
pub async fn check_all_redis(
    connections: Vec<(&str, RedisHealthState)>,
) -> RedisHealth {
    let mut results = Vec::new();
    for (service_name, state) in connections {
        let health = state.check(service_name).await;
        results.push(health);
    }

    let all_healthy = results.iter().all(|c| c.status == "healthy" );

    let status = if all_healthy {
        "healthy"
    } else {
        "degraded"
    };

    RedisHealth {
        status: status.to_string(),
        connections: results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            status: "healthy".to_string(),
            service: "test-service".to_string(),
            version: "1.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("healthy" ));
    }

    #[tokio::test]
    async fn test_db_health_state_none() {
        let state = DbHealthState::None;
        let health = state.check("test-service" ).await;
        assert_eq!(health.status, "not_configured" );
        assert_eq!(health.service, "test-service" );
        assert!(health.error.is_none());
    }

    #[tokio::test]
    async fn test_redis_health_state_none() {
        let state = RedisHealthState::None;
        let health = state.check("test-service" ).await;
        assert_eq!(health.status, "not_configured" );
        assert_eq!(health.service, "test-service" );
        assert!(health.error.is_none());
    }
}
