//! 数据库连接配置
//!
//! 包含 `PoolConfig`、`DatabaseConfig`、`ServiceDatabaseConfig` 以及连接池创建函数。

use config::ConfigError;
use serde::Deserialize;

/// 数据库连接池配置
#[derive(Debug, Clone, Deserialize)]
pub struct PoolConfig {
    /// 最大连接数
    pub max_connections: u32,
    /// 最小空闲连接数
    pub min_connections: u32,
    /// 连接超时（秒）
    pub connect_timeout_secs: u64,
    /// 空闲连接超时（秒）
    pub idle_timeout_secs: u64,
    /// 连接最大存活时间（秒）
    pub max_lifetime_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL" ).unwrap_or_else(|_| {
                "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string()
            }),
            max_connections: Self::pool_config_from_env().max_connections,
            min_connections: Self::pool_config_from_env().min_connections,
            connect_timeout_secs: Self::pool_config_from_env().connect_timeout_secs,
            idle_timeout_secs: Self::pool_config_from_env().idle_timeout_secs,
            max_lifetime_secs: Self::pool_config_from_env().max_lifetime_secs,
        }
    }
}

impl DatabaseConfig {
    /// 从环境变量读取连接池配置
    fn pool_config_from_env() -> PoolConfig {
        PoolConfig {
            max_connections: std::env::var("DB_POOL_MAX_CONNECTIONS" )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DB_POOL_MIN_CONNECTIONS" )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            connect_timeout_secs: std::env::var("DB_POOL_CONNECT_TIMEOUT" )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            idle_timeout_secs: std::env::var("DB_POOL_IDLE_TIMEOUT" )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            max_lifetime_secs: std::env::var("DB_POOL_MAX_LIFETIME" )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1800),
        }
    }

    /// 验证数据库连接配置是否有效
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::Message("数据库 URL 不能为空".to_string()));
        }
        if self.max_connections < self.min_connections {
            return Err(ConfigError::Message(
                "最大连接数必须大于等于最小连接数".to_string(),
            ));
        }
        Ok(())
    }
}

/// 服务级数据库配置
///
/// 支持每个服务使用独立的数据库连接：
/// - `{SERVICE_NAME}_DB_URL`（大写，带下划线，如 `USER_SERVICE_DB_URL`）
/// - `SERVICE_DB_URL`（通用回退）
/// - `DATABASE_URL`（最终回退）
///
/// 可选 schema 隔离：通过 `SERVICE_SCHEMA` 环境变量指定
#[derive(Debug, Clone)]
pub struct ServiceDatabaseConfig {
    pub url: String,
    pub schema: Option<String>,
    pub pool: PoolConfig,
    pub service_name: String,
}

impl ServiceDatabaseConfig {
    /// 根据服务名解析数据库 URL
    ///
    /// 优先级：
    /// 1. `{SERVICE_NAME}_DB_URL`（服务专用，如 `USER_SERVICE_DB_URL`）
    /// 2. `SERVICE_DB_URL`（通用环境变量）
    /// 3. `DATABASE_URL`（全局回退）
    #[must_use]
    pub fn resolve_url(service_name: &str) -> String {
        let service_key = format!("{}_DB_URL" , service_name.to_uppercase().replace('-', "_" ));
        std::env::var(&service_key)
            .or_else(|_| std::env::var("SERVICE_DB_URL" ))
            .unwrap_or_else(|_| {
                std::env::var("DATABASE_URL" ).unwrap_or_else(|_| {
                    "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string()
                })
            })
    }

    /// 创建服务级数据库配置
    #[must_use]
    pub fn for_service(service_name: &str) -> Self {
        let url = Self::resolve_url(service_name);
        let schema = std::env::var("SERVICE_SCHEMA" ).ok();
        let pool = PoolConfig::default();
        Self {
            url,
            schema,
            pool,
            service_name: service_name.to_string(),
        }
    }

    /// 获取带 schema 选项的连接 URL（如果配置了 schema）
    #[must_use]
    pub fn connection_url(&self) -> String {
        if let Some(ref schema) = self.schema {
            let encoded = urlencoding(schema);
            if self.url.contains('?') {
                format!("{}&options=--search_path%3D{}" , self.url, encoded)
            } else {
                format!("{}?options=--search_path%3D{}" , self.url, encoded)
            }
        } else {
            self.url.clone()
        }
    }

    /// 获取设置 `search_path` 的 SQL 语句（schema 隔离）
    #[must_use]
    pub fn schema_sql(&self) -> Option<String> {
        self.schema
            .as_ref()
            .map(|s| format!("SET search_path TO {s}" ))
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::Message("数据库 URL 不能为空".to_string()));
        }
        Ok(())
    }
}

/// URL 编码（针对 schema 名称）
///
/// 使用 `urlencoding` crate 统一处理，避免手动实现遗漏字符。
fn urlencoding(input: &str) -> String {
    urlencoding::encode(input).into_owned()
}

/// 从配置创建数据库连接池
pub async fn create_db_pool(config: &DatabaseConfig) -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_secs))
        .connect(&config.url)
        .await
}

/// 从服务级配置创建数据库连接池
pub async fn create_service_db_pool(
    config: &ServiceDatabaseConfig,
) -> Result<sqlx::PgPool, sqlx::Error> {
    let pool_config = &config.pool;
    let opts = sqlx::postgres::PgPoolOptions::new()
        .max_connections(pool_config.max_connections)
        .min_connections(pool_config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(pool_config.connect_timeout_secs))
        .idle_timeout(std::time::Duration::from_secs(pool_config.idle_timeout_secs))
        .max_lifetime(std::time::Duration::from_secs(pool_config.max_lifetime_secs));

    let pool = opts.connect(&config.connection_url()).await?;

    // 如果配置了 schema 隔离，设置 search_path
    if let Some(sql) = config.schema_sql() {
        // B11 豁免: schema 迁移 SQL 运行时动态生成, 无法用编译期宏
        sqlx::query(&sql).execute(&pool).await?;
        tracing::info!(
            "Schema 隔离已启用: service={}, schema={}" ,
            config.service_name,
            config.schema.as_deref().unwrap_or("" )
        );
    }

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert!(!config.url.is_empty());
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }

    #[test]
    fn test_database_config_validate() {
        let mut config = DatabaseConfig::default();
        assert!(config.validate().is_ok());

        config.url = "".to_string();
        assert!(config.validate().is_err());

        config.url = "postgres://localhost/test".to_string();
        config.max_connections = 2;
        config.min_connections = 5;
        assert!(config.validate().is_err());
    }
}
