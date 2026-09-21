//! 配置模块入口
//!
//! 导出所有子模块，提供统一配置加载入口。

pub mod database;
pub mod redis;
pub mod jwt;
pub mod service;
pub mod meilisearch;

pub use database::{
    DatabaseConfig, PoolConfig, ServiceDatabaseConfig,
    create_db_pool, create_service_db_pool,
};
pub use redis::RedisConfig;
pub use jwt::JwtConfig;
pub use service::ServiceDiscoveryConfig;
pub use meilisearch::MeilisearchConfig;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::sync::Arc;

/// 服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl ServiceConfig {
    /// 获取服务地址
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}" , self.host, self.port)
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub meilisearch: MeilisearchConfig,
}

impl AppConfig {
    /// 从配置文件和环境变量加载配置
    pub fn load() -> Result<Self, ConfigError> {
        // 校验必要的环境变量
        if std::env::var("DATABASE_URL" ).is_err() {
            return Err(ConfigError::Message(
                "DATABASE_URL is required".to_string(),
            ));
        }

        if std::env::var("JWT_SECRET" ).is_err() {
            return Err(ConfigError::Message(
                "JWT_SECRET is required".to_string(),
            ));
        }

        let config = Config::builder()
            .set_default("service.name" , "service" )?
            .set_default("service.host" , "0.0.0.0" )?
            .set_default("service.port" , 8080)?
            .set_default("database.max_connections" , 10)?
            .set_default("database.min_connections" , 2)?
            .set_default("database.connect_timeout_secs" , 30)?
            .set_default("database.idle_timeout_secs" , 600)?
            .set_default("database.max_lifetime_secs" , 1800)?
            .set_default("redis.max_connections" , 10)?
            .set_default("jwt.access_token_expiry_secs" , 3600)?
            .set_default("jwt.refresh_token_expiry_secs" , 604800)?
            .add_source(File::with_name("config" ).required(false))
            .add_source(Environment::with_prefix("APP" ).separator("__" ))
            .build()?;

        config.try_deserialize()
    }

    /// 创建全局配置实例
    ///
    /// # Errors
    /// 如果配置文件不存在或格式错误，返回错误
    pub fn global() -> Result<Arc<Self>, ConfigError> {
        Ok(Arc::new(Self::load()?))
    }

    /// 验证所有配置项
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.database.validate()?;
        self.redis.validate()?;
        self.jwt.validate()?;
        Ok(())
    }
}

/// 从环境变量解析端口，支持默认值
///
/// 用法: `let port = parse_port_from_env("SERVICE_PORT" , 8080);`
///
/// # 示例
/// ```ignore
/// // 所有服务统一使用:
/// let port = common::config::parse_port_from_env("SERVICE_PORT" , 8080);
/// ```
#[must_use]
pub fn parse_port_from_env(var_name: &str, default: u16) -> u16 {
    std::env::var(var_name)
        .unwrap_or_else(|_| default.to_string())
        .parse()
        .unwrap_or(default)
}
