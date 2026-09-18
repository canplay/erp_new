//! API Key 数据仓储层
//!
//! 实现 API Key 的数据库 CRUD 操作

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// API Key 数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub key_id: String,
    pub secret_key_hash: String,
    pub key_hint: String,
    pub permission_level: i32,
    pub allowed_ips: Vec<String>,
    pub rate_limit: i64,
    pub tenant_id: Option<i64>,
    pub user_id: Option<i64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// API Key 使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsageLog {
    pub id: String,
    pub key_id: String,
    pub endpoint: String,
    pub method: String,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i32>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_size: Option<i64>,
    pub response_size: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// API Key 仓储 trait
pub trait ApiKeyRepository: Send + Sync {
    fn create(
        &self,
        key: &ApiKey,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn find_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<ApiKey>, sqlx::Error>> + Send;
    fn find_by_key_id(
        &self,
        key_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<ApiKey>, sqlx::Error>> + Send;
    fn list_by_user(
        &self,
        user_id: i64,
        tenant_id: i64,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<ApiKey>, i64), sqlx::Error>> + Send;
    fn update(
        &self,
        key: &ApiKey,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn delete(&self, id: &str)
    -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn update_last_used(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn log_usage(
        &self,
        log: &ApiKeyUsageLog,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send;
    fn list_usage_logs(
        &self,
        key_id: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> impl std::future::Future<Output = Result<(Vec<ApiKeyUsageLog>, i64), sqlx::Error>> + Send;
    fn delete_usage_logs(
        &self,
        key_id: Option<&str>,
        before_date: Option<DateTime<Utc>>,
    ) -> impl std::future::Future<Output = Result<i64, sqlx::Error>> + Send;
}

/// `PostgreSQL` API Key 仓储实现
pub struct PostgresApiKeyRepository {
    pool: PgPool,
}

impl PostgresApiKeyRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn create(&self, key: &ApiKey) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"
            INSERT INTO api_keys (id, name, description, key_id, secret_key_hash, key_hint,
                permission_level, allowed_ips, rate_limit, tenant_id, user_id, status, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            " ,
            &key.id,
            &key.name,
            key.description.as_ref().map(|s| s.as_str()),
            &key.key_id,
            &key.secret_key_hash,
            &key.key_hint,
            key.permission_level,
            serde_json::to_value(&key.allowed_ips).unwrap_or_default(),
            key.rate_limit,
            key.tenant_id,
            key.user_id,
            &key.status,
            key.created_at,
            key.expires_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, description, key_id, secret_key_hash, key_hint,
                COALESCE(permission_level, 0) AS "permission_level!" ,
                COALESCE(allowed_ips, '[]'::jsonb)::text AS allowed_ips,
                COALESCE(rate_limit, 0) AS "rate_limit!" ,
                tenant_id, user_id,
                COALESCE(status, '') AS "status!" ,
                created_at, updated_at, expires_at, last_used_at
            FROM api_keys WHERE id = $1
"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ApiKey {
            id: r.id,
            name: r.name,
            description: r.description,
            key_id: r.key_id,
            secret_key_hash: r.secret_key_hash,
            key_hint: r.key_hint,
            permission_level: r.permission_level,
            allowed_ips: serde_json::from_str(r.allowed_ips.as_deref().unwrap_or("[]" )).unwrap_or_default(),
            rate_limit: r.rate_limit,
            tenant_id: r.tenant_id,
            user_id: r.user_id,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
            expires_at: r.expires_at,
            last_used_at: r.last_used_at,
        }))
    }

    async fn find_by_key_id(&self, key_id: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, description, key_id, secret_key_hash, key_hint,
                COALESCE(permission_level, 0) AS "permission_level!" ,
                COALESCE(allowed_ips, '[]'::jsonb)::text AS allowed_ips,
                COALESCE(rate_limit, 0) AS "rate_limit!" ,
                tenant_id, user_id,
                COALESCE(status, '') AS "status!" ,
                created_at, updated_at, expires_at, last_used_at
            FROM api_keys WHERE key_id = $1
"#,
            key_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ApiKey {
            id: r.id,
            name: r.name,
            description: r.description,
            key_id: r.key_id,
            secret_key_hash: r.secret_key_hash,
            key_hint: r.key_hint,
            permission_level: r.permission_level,
            allowed_ips: serde_json::from_str(r.allowed_ips.as_deref().unwrap_or("[]" )).unwrap_or_default(),
            rate_limit: r.rate_limit,
            tenant_id: r.tenant_id,
            user_id: r.user_id,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
            expires_at: r.expires_at,
            last_used_at: r.last_used_at,
        }))
    }

    async fn list_by_user(
        &self,
        user_id: i64,
        tenant_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ApiKey>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM api_keys WHERE user_id = $1 AND tenant_id = $2", user_id, tenant_id)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, key_id, secret_key_hash, key_hint,
                COALESCE(permission_level, 0) AS "permission_level!" ,
                COALESCE(allowed_ips, '[]'::jsonb)::text AS allowed_ips,
                COALESCE(rate_limit, 0) AS "rate_limit!" ,
                tenant_id, user_id,
                COALESCE(status, '') AS "status!" ,
                created_at, updated_at, expires_at, last_used_at
            FROM api_keys WHERE user_id = $1 AND tenant_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4
"#,
            user_id as i64,
            tenant_id as i64,
            page_size as i64,
            offset as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        let keys: Vec<ApiKey> = rows
            .into_iter()
            .map(|r| ApiKey {
                id: r.id,
                name: r.name,
                description: r.description,
                key_id: r.key_id,
                secret_key_hash: r.secret_key_hash,
                key_hint: r.key_hint,
                permission_level: r.permission_level,
                allowed_ips: serde_json::from_str(r.allowed_ips.as_deref().unwrap_or("[]" )).unwrap_or_default(),
                rate_limit: r.rate_limit,
                tenant_id: r.tenant_id,
                user_id: r.user_id,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
                last_used_at: r.last_used_at,
            })
            .collect();

        Ok((keys, count))
    }

    async fn update(&self, key: &ApiKey) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"
            UPDATE api_keys SET name = $2, description = $3, permission_level = $4,
                allowed_ips = $5, rate_limit = $6, status = $7, expires_at = $8, updated_at = NOW()
            WHERE id = $1
            " ,
            &key.id,
            &key.name,
            key.description.as_ref().map(|s| s.as_str()),
            key.permission_level,
            serde_json::to_value(&key.allowed_ips).unwrap_or_default(),
            key.rate_limit,
            &key.status,
            key.expires_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM api_keys WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_last_used(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn log_usage(&self, log: &ApiKeyUsageLog) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"
            INSERT INTO api_key_usage_logs 
                (id, key_id, endpoint, method, status_code, latency_ms, ip_address, user_agent, request_size, response_size, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            " ,
            log.id.parse::<i64>().unwrap_or_default(),
            log.key_id,
            &log.endpoint,
            &log.method,
            log.status_code,
            log.latency_ms,
            log.ip_address.as_ref().map(|s| s.as_str()),
            log.user_agent.as_ref().map(|s| s.as_str()),
            log.request_size,
            log.response_size,
            log.created_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_usage_logs(
        &self,
        key_id: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ApiKeyUsageLog>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let (count, logs): (i64, Vec<ApiKeyUsageLog>);

        if let Some(kid) = key_id {
            count = sqlx::query_scalar!("SELECT COUNT(*) FROM api_key_usage_logs WHERE key_id = $1" , kid)
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0);

            let rows = sqlx::query!(
                "SELECT id, key_id, endpoint, method, status_code, latency_ms, ip_address, user_agent, request_size, response_size, created_at FROM api_key_usage_logs WHERE key_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3" ,
                kid,
                page_size as i64,
                offset as i64,
            )
            .fetch_all(&self.pool)
            .await?;

            logs = rows
                .into_iter()
                .map(|r| ApiKeyUsageLog {
                    id: r.id.to_string(),
                    key_id: r.key_id,
                    endpoint: r.endpoint,
                    method: r.method,
                    status_code: r.status_code,
                    latency_ms: r.latency_ms,
                    ip_address: r.ip_address,
                    user_agent: r.user_agent,
                    request_size: r.request_size,
                    response_size: r.response_size,
                    created_at: r.created_at,
                })
                .collect();
        } else {
            count = sqlx::query_scalar!("SELECT COUNT(*) FROM api_key_usage_logs" )
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0);

            let rows = sqlx::query!(
                "SELECT id, key_id, endpoint, method, status_code, latency_ms, ip_address, user_agent, request_size, response_size, created_at FROM api_key_usage_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2" ,
                page_size as i64,
                offset as i64,
            )
            .fetch_all(&self.pool)
            .await?;

            logs = rows
                .into_iter()
                .map(|r| ApiKeyUsageLog {
                    id: r.id.to_string(),
                    key_id: r.key_id,
                    endpoint: r.endpoint,
                    method: r.method,
                    status_code: r.status_code,
                    latency_ms: r.latency_ms,
                    ip_address: r.ip_address,
                    user_agent: r.user_agent,
                    request_size: r.request_size,
                    response_size: r.response_size,
                    created_at: r.created_at,
                })
                .collect();
        }

        Ok((logs, count))
    }

    async fn delete_usage_logs(
        &self,
        key_id: Option<&str>,
        before_date: Option<DateTime<Utc>>,
    ) -> Result<i64, sqlx::Error> {
        let result = if let (Some(kid), Some(date)) = (key_id, before_date) {
            sqlx::query!("DELETE FROM api_key_usage_logs WHERE key_id = $1 AND created_at < $2" , kid, date)
                .execute(&self.pool)
                .await?
        } else if let Some(kid) = key_id {
            sqlx::query!("DELETE FROM api_key_usage_logs WHERE key_id = $1" , kid)
                .execute(&self.pool)
                .await?
        } else if let Some(date) = before_date {
            sqlx::query!("DELETE FROM api_key_usage_logs WHERE created_at < $1" , date)
                .execute(&self.pool)
                .await?
        } else {
            sqlx::query!("DELETE FROM api_key_usage_logs" )
                .execute(&self.pool)
                .await?
        };
        Ok(result.rows_affected() as i64)
    }
}
