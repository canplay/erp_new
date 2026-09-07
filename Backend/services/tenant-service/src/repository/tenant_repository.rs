//! 租户 Repository - 租户数据访问

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;

/// Repository 错误类型
#[derive(Error, Debug)]
pub enum TenantRepositoryError {
    #[error("租户不存在")]
    NotFound,

    #[error("租户已存在")]
    AlreadyExists,

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

/// 租户查询参数
#[derive(Debug, Default)]
pub struct TenantQueryParams {
    pub keyword: Option<String>,
    pub status: Option<i32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 租户详情
#[derive(Debug, Clone)]
pub struct TenantDetail {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub max_users: i32,
    pub max_storage: i64,
    pub status: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 租户列表项
#[derive(Debug, Clone)]
pub struct TenantListItem {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub status: i32,
    pub max_users: i64,
    pub current_users: i64,
    pub created_at: DateTime<Utc>,
}

/// 分页租户响应
#[derive(Debug)]
pub struct PaginatedTenants {
    pub tenants: Vec<TenantListItem>,
    pub total: i64,
}

/// 租户用户记录
#[derive(Debug, Clone, Serialize)]
pub struct TenantUserRecord {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub department: Option<String>,
    pub position: Option<String>,
    pub status: i32,
    pub joined_at: String,
}

/// 租户用户列表响应
#[derive(Debug)]
pub struct PaginatedTenantUsers {
    pub users: Vec<TenantUserRecord>,
    pub total: i64,
}

/// 使用统计
#[derive(Debug)]
pub struct UsageStats {
    pub total_users: i64,
    pub active_users: i64,
    pub used_storage: i64,
    pub max_storage: i64,
    pub monthly_api_calls: i64,
    pub api_call_limit: i64,
}

/// 审计日志
#[derive(Debug, Clone, Serialize)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

/// 审计日志列表响应
#[derive(Debug)]
pub struct PaginatedAuditLogs {
    pub logs: Vec<AuditLog>,
    pub total: i64,
}

/// 租户更新参数
#[derive(Debug, Default)]
pub struct UpdateTenantParams {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
    pub max_users: Option<i32>,
    pub max_storage: Option<i64>,
}

/// 租户 Repository
#[derive(Clone)]
pub struct TenantRepository {
    pool: PgPool,
}

impl TenantRepository {
    /// 创建新的 Repository 实例
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 查询租户
    pub async fn find_by_id(&self, id: i64) -> Result<Option<TenantDetail>, TenantRepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, code, domain, description,
                   COALESCE(max_users, 0) AS "max_users!",
                   COALESCE(max_storage, 0) AS "max_storage!",
                   COALESCE(status, 0) AS "status!",
                   expires_at,
                   COALESCE(created_at, NOW()) AS "created_at!",
                   COALESCE(updated_at, NOW()) AS "updated_at!"
            FROM tenants
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TenantDetail {
            id: r.id,
            name: r.name,
            code: r.code,
            domain: r.domain,
            description: r.description,
            max_users: r.max_users,
            max_storage: r.max_storage,
            status: r.status,
            expires_at: r.expires_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 根据 code 查询租户
    pub async fn find_by_code(
        &self,
        code: &str,
    ) -> Result<Option<TenantDetail>, TenantRepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, code, domain, description,
                   COALESCE(max_users, 0) AS "max_users!",
                   COALESCE(max_storage, 0) AS "max_storage!",
                   COALESCE(status, 0) AS "status!",
                   expires_at,
                   COALESCE(created_at, NOW()) AS "created_at!",
                   COALESCE(updated_at, NOW()) AS "updated_at!"
            FROM tenants
            WHERE code = $1
            "#,
            code,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TenantDetail {
            id: r.id,
            name: r.name,
            code: r.code,
            domain: r.domain,
            description: r.description,
            max_users: r.max_users,
            max_storage: r.max_storage,
            status: r.status,
            expires_at: r.expires_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 分页查询租户列表
    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
    ) -> Result<PaginatedTenants, TenantRepositoryError> {
        let offset = (page - 1) * page_size;

        // 按关键字过滤 / 全量查询, 分别物化为 TenantListItem
        let (items, total) = if let Some(kw) = keyword {
            let keyword_pattern = format!("%{kw}%");
            let rows = sqlx::query!(
                r#"
                SELECT t.id, t.name, t.code,
                       COALESCE(t.status, 0) AS "status!",
                       COALESCE(t.max_users, 0)::bigint AS "max_users!",
                       COALESCE(t.created_at, NOW()) AS "created_at!"
                FROM tenants t
                WHERE t.name ILIKE $1 OR t.code ILIKE $1
                ORDER BY t.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                &keyword_pattern,
                i64::from(page_size),
                i64::from(offset),
            )
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM tenants WHERE name ILIKE $1 OR code ILIKE $1",
                &keyword_pattern,
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

            // 为每个租户补充当前用户数（单独查询, 单次失败不中断整个列表）
            let mut items = Vec::new();
            for r in rows {
                let user_count = sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM tenant_users WHERE tenant_id = $1",
                    r.id,
                )
                .fetch_one(&self.pool)
                .await
                .unwrap_or(Some(0))
                .unwrap_or(0);

                items.push(TenantListItem {
                    id: r.id,
                    name: r.name,
                    code: r.code,
                    status: r.status,
                    max_users: r.max_users,
                    current_users: user_count,
                    created_at: r.created_at,
                });
            }

            (items, total)
        } else {
            let rows = sqlx::query!(
                r#"
                SELECT t.id, t.name, t.code,
                       COALESCE(t.status, 0) AS "status!",
                       COALESCE(t.max_users, 0)::bigint AS "max_users!",
                       COALESCE(t.created_at, NOW()) AS "created_at!"
                FROM tenants t
                ORDER BY t.created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                i64::from(page_size),
                i64::from(offset),
            )
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar!("SELECT COUNT(*) FROM tenants")
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0);

            // 为每个租户补充当前用户数（单独查询, 单次失败不中断整个列表）
            let mut items = Vec::new();
            for r in rows {
                let user_count = sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM tenant_users WHERE tenant_id = $1",
                    r.id,
                )
                .fetch_one(&self.pool)
                .await
                .unwrap_or(Some(0))
                .unwrap_or(0);

                items.push(TenantListItem {
                    id: r.id,
                    name: r.name,
                    code: r.code,
                    status: r.status,
                    max_users: r.max_users,
                    current_users: user_count,
                    created_at: r.created_at,
                });
            }

            (items, total)
        };

        Ok(PaginatedTenants {
            tenants: items,
            total,
        })
    }

    /// 创建租户
    pub async fn create(
        &self,
        name: &str,
        code: &str,
        domain: Option<&str>,
        description: Option<&str>,
        max_users: i32,
        max_storage: i64,
    ) -> Result<i64, TenantRepositoryError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO tenants (name, code, domain, description, max_users, max_storage, status)
            VALUES ($1, $2, $3, $4, $5, $6, 1)
            RETURNING id
            "#,
            name,
            code,
            domain,
            description,
            max_users,
            max_storage,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// 更新租户
    pub async fn update(
        &self,
        id: i64,
        params: UpdateTenantParams,
    ) -> Result<bool, TenantRepositoryError> {
        if params.name.is_none()
            && params.domain.is_none()
            && params.description.is_none()
            && params.status.is_none()
            && params.max_users.is_none()
            && params.max_storage.is_none()
        {
            return Ok(false);
        }

        let result = sqlx::query!(
            r#"
            UPDATE tenants
            SET name = COALESCE($2, name),
                domain = COALESCE($3, domain),
                description = COALESCE($4, description),
                status = COALESCE($5, status),
                max_users = COALESCE($6, max_users),
                max_storage = COALESCE($7, max_storage),
                updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            params.name.as_deref(),
            params.domain.as_deref(),
            params.description.as_deref(),
            params.status,
            params.max_users,
            params.max_storage,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除租户
    pub async fn delete(&self, id: i64) -> Result<bool, TenantRepositoryError> {
        let result = sqlx::query!("DELETE FROM tenants WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取租户用户列表
    pub async fn list_users(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
        _role: Option<&str>,
        _keyword: Option<&str>,
    ) -> Result<PaginatedTenantUsers, TenantRepositoryError> {
        let _offset = (page - 1) * page_size;

        let rows = sqlx::query!(
            r#"
            SELECT tu.id, tu.user_id, u.username, u.email, tu.role, tu.department, tu.position,
                   tu.status, tu.joined_at
            FROM tenant_users tu
            JOIN users u ON u.id = tu.user_id
            WHERE tu.tenant_id = $1
            "#,
            tenant_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let total = rows.len() as i64;
        let users = rows
            .into_iter()
            .map(|r| TenantUserRecord {
                id: r.id,
                user_id: r.user_id,
                username: r.username,
                email: r.email,
                role: r.role,
                department: r.department,
                position: r.position,
                status: r.status,
                joined_at: r.joined_at.to_rfc3339(),
            })
            .collect();

        Ok(PaginatedTenantUsers { users, total })
    }

    /// 添加租户用户
    pub async fn add_user(
        &self,
        tenant_id: i64,
        user_id: i64,
        role: &str,
        department: Option<&str>,
        position: Option<&str>,
    ) -> Result<(), TenantRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO tenant_users (tenant_id, user_id, role, department, position, status)
            VALUES ($1, $2, $3, $4, $5, 1)
            ON CONFLICT (tenant_id, user_id) DO UPDATE SET
                role = EXCLUDED.role,
                department = EXCLUDED.department,
                position = EXCLUDED.position
            "#,
            tenant_id,
            user_id,
            role,
            department,
            position,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新租户用户
    pub async fn update_user(
        &self,
        tenant_id: i64,
        user_id: i64,
        role: Option<&str>,
        department: Option<&str>,
        position: Option<&str>,
    ) -> Result<bool, TenantRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE tenant_users
            SET role = COALESCE($3, role),
                department = COALESCE($4, department),
                position = COALESCE($5, position)
            WHERE tenant_id = $1 AND user_id = $2
            "#,
            tenant_id,
            user_id,
            role,
            department,
            position,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 移除租户用户
    pub async fn remove_user(
        &self,
        tenant_id: i64,
        user_id: i64,
    ) -> Result<bool, TenantRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM tenant_users WHERE tenant_id = $1 AND user_id = $2",
            tenant_id,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取使用统计
    pub async fn get_usage_stats(
        &self,
        tenant_id: i64,
    ) -> Result<UsageStats, TenantRepositoryError> {
        // 获取用户统计
        let user_stats = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "total!",
                   COUNT(*) FILTER (WHERE status = 1) AS "active!"
            FROM tenant_users
            WHERE tenant_id = $1
            "#,
            tenant_id,
        )
        .fetch_one(&self.pool)
        .await?;

        // 获取租户限制
        let tenant = self.find_by_id(tenant_id).await?;

        // 获取存储使用量（通过文件服务 API 或数据库查询）
        // 注意：used_storage 需要从 file-service 获取，这里使用估算值
        let used_storage = self.get_storage_usage(tenant_id).await.unwrap_or(0);

        // 获取 API 调用统计（通过 API Gateway 或日志统计）
        // 注意：monthly_api_calls 需要从 api-gateway 获取，这里使用估算值
        let monthly_api_calls = self.get_api_call_stats(tenant_id).await.unwrap_or(0);

        Ok(UsageStats {
            total_users: user_stats.total,
            active_users: user_stats.active,
            used_storage,
            max_storage: tenant
                .map_or(100 * 1024 * 1024 * 1024, |t| t.max_storage),
            monthly_api_calls,
            api_call_limit: 50000,
        })
    }

    /// 获取审计日志
    pub async fn get_audit_logs(
        &self,
        tenant_id: i64, // 从认证中间件获取的 tenant_id
        page: i32,
        page_size: i32,
        _keyword: Option<&str>,
    ) -> Result<PaginatedAuditLogs, TenantRepositoryError> {
        let offset = (page - 1) * page_size;
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, username, action, resource_type, resource_id, details, ip_address,
                   COALESCE(created_at, NOW()) AS "created_at!"
            FROM audit_logs
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total = rows.len() as i64;
        let logs = rows
            .into_iter()
            .map(|r| AuditLog {
                id: r.id,
                user_id: r.user_id,
                username: r.username,
                action: r.action,
                resource_type: r.resource_type,
                resource_id: r.resource_id,
                details: r.details,
                ip_address: r.ip_address,
                created_at: r.created_at.to_rfc3339(),
            })
            .collect();

        Ok(PaginatedAuditLogs { logs, total })
    }

    /// 获取租户存储使用量
    ///
    /// 通过查询文件表统计租户已使用的存储空间
    async fn get_storage_usage(&self, tenant_id: i64) -> Result<i64, TenantRepositoryError> {
        let size = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(file_size), 0)::bigint FROM files WHERE tenant_id = $1",
            tenant_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(size.unwrap_or(0))
    }

    /// 获取租户 API 调用统计
    ///
    /// 通过查询 API 调用日志统计租户每月 API 调用次数
    async fn get_api_call_stats(&self, tenant_id: i64) -> Result<i64, TenantRepositoryError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM api_usage_logs
            WHERE tenant_id = $1
            AND created_at >= date_trunc('month', NOW())
            "#,
            tenant_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(count.unwrap_or(0))
    }
}
