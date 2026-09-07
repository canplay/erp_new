//! 角色仓储层

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::PgPool;
use thiserror::Error;

/// 角色仓储错误类型
#[derive(Error, Debug)]
pub enum RoleRepositoryError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("角色不存在")]
    NotFound,

    #[error("角色已存在")]
    AlreadyExists,

    #[error("角色有关联用户，无法删除")]
    HasAssociatedUsers,

    #[error("角色有子角色，无法删除")]
    HasChildRoles,
}

/// 角色信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub role_type: String,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub status: i32,
    pub is_default: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// 角色列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleListItem {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub role_type: String,
    pub level: i32,
    pub status: i32,
    pub is_default: bool,
    pub user_count: i64,
    pub created_at: chrono::DateTime<Utc>,
}

/// 权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub permission_type: String,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i32,
}

/// 角色模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTemplate {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub role_type: Option<String>,
    pub is_system: bool,
    pub created_at: chrono::DateTime<Utc>,
}

/// 分页结果
pub struct PaginatedRoles {
    pub roles: Vec<RoleListItem>,
    pub total: i64,
}

/// 角色仓储
#[derive(Clone)]
pub struct RoleRepository {
    pool: PgPool,
}

impl RoleRepository {
    /// 创建新的角色仓储
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建角色
    pub async fn create(
        &self,
        name: &str,
        code: &str,
        description: Option<String>,
        role_type: &str,
        parent_id: Option<i64>,
    ) -> Result<i64, RoleRepositoryError> {
        // 检查代码是否已存在
        let exists = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1) AS "exists!""#,
            code
        )
        .fetch_one(&self.pool)
        .await?
        .exists;

        if exists {
            return Err(RoleRepositoryError::AlreadyExists);
        }

        // 计算层级深度
        let level = if let Some(pid) = parent_id {
            let row = sqlx::query!(
                "SELECT level FROM roles WHERE id = $1",
                pid
            )
            .fetch_optional(&self.pool)
            .await?;

            match row {
                Some(r) => r.level.unwrap_or(0) + 1,
                None => 0,
            }
        } else {
            0
        };

        // 检查层级深度限制
        if level > 3 {
            return Err(RoleRepositoryError::Database(sqlx::Error::Protocol(
                "角色层级不能超过3级".into(),
            )));
        }

        // 插入新角色
        let row = sqlx::query!(
            r"INSERT INTO roles (name, code, description, role_type, parent_id, level, status)
               VALUES ($1, $2, $3, $4, $5, $6, 1)
               RETURNING id",
            name,
            code,
            description.as_deref(),
            role_type,
            parent_id,
            level,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 根据代码查找角色
    pub async fn find_by_code(&self, code: &str) -> Result<Option<Role>, RoleRepositoryError> {
        let row = sqlx::query_as!(
            Role,
            r#"SELECT id, name, code, description,
                      COALESCE(role_type, 'user') AS "role_type!",
                      parent_id,
                      COALESCE(level, 0) AS "level!",
                      COALESCE(sort_order, 0) AS "sort_order!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(is_default, false) AS "is_default!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM roles WHERE code = $1"#,
            code,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 根据ID查找角色
    pub async fn find_by_id(&self, role_id: i64) -> Result<Option<Role>, RoleRepositoryError> {
        let row = sqlx::query_as!(
            Role,
            r#"SELECT id, name, code, description,
                      COALESCE(role_type, 'user') AS "role_type!",
                      parent_id,
                      COALESCE(level, 0) AS "level!",
                      COALESCE(sort_order, 0) AS "sort_order!",
                      COALESCE(status, 1) AS "status!",
                      COALESCE(is_default, false) AS "is_default!",
                      COALESCE(created_at, NOW()) AS "created_at!",
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM roles WHERE id = $1"#,
            role_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 更新角色
    pub async fn update(
        &self,
        code: &str,
        name: Option<String>,
        description: Option<String>,
        status: Option<i32>,
    ) -> Result<Option<Role>, RoleRepositoryError> {
        let row = sqlx::query_as!(
            Role,
            r#"UPDATE roles
               SET name = COALESCE($1, name),
                   description = COALESCE($2, description),
                   status = COALESCE($3, status),
                   updated_at = NOW()
               WHERE code = $4
               RETURNING id, name, code, description,
                         COALESCE(role_type, 'user') AS "role_type!",
                         parent_id,
                         COALESCE(level, 0) AS "level!",
                         COALESCE(sort_order, 0) AS "sort_order!",
                         COALESCE(status, 1) AS "status!",
                         COALESCE(is_default, false) AS "is_default!",
                         COALESCE(created_at, NOW()) AS "created_at!",
                         COALESCE(updated_at, NOW()) AS "updated_at!"
"#,
            name.as_deref(),
            description.as_deref(),
            status,
            code,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 删除角色
    pub async fn delete(&self, code: &str) -> Result<bool, RoleRepositoryError> {
        // 检查是否有子角色
        let role = self.find_by_code(code).await?;
        if let Some(role) = role {
            let child_count =
                sqlx::query!("SELECT COUNT(*) as count FROM roles WHERE parent_id = $1", role.id)
                    .fetch_one(&self.pool)
                    .await?
                    .count
                    .unwrap_or(0);

            if child_count > 0 {
                return Err(RoleRepositoryError::HasChildRoles);
            }

            // 检查是否有用户关联
            let user_count =
                sqlx::query!("SELECT COUNT(*) as count FROM user_roles WHERE role_id = $1", role.id)
                    .fetch_one(&self.pool)
                    .await?
                    .count
                    .unwrap_or(0);

            if user_count > 0 {
                return Err(RoleRepositoryError::HasAssociatedUsers);
            }

            // 删除角色权限关联
            sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1", role.id)
                .execute(&self.pool)
                .await?;

            // 删除角色
            let result = sqlx::query!("DELETE FROM roles WHERE code = $1", code)
                .execute(&self.pool)
                .await?;

            return Ok(result.rows_affected() > 0);
        }

        Ok(false)
    }

    /// 分页查询角色列表
    pub async fn list(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
    ) -> Result<PaginatedRoles, RoleRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            RoleListItem,
            r#"SELECT r.id, r.name, r.code, r.description,
                      COALESCE(r.role_type, 'user') AS "role_type!",
                      COALESCE(r.level, 0) AS "level!",
                      COALESCE(r.status, 1) AS "status!",
                      COALESCE(r.is_default, false) AS "is_default!",
                      COALESCE(r.created_at, NOW()) AS "created_at!",
                      COUNT(ur.id) AS "user_count!"
               FROM roles r
               LEFT JOIN user_roles ur ON r.id = ur.role_id
               WHERE ($1::text IS NULL OR r.name ILIKE '%' || $1 || '%' OR r.code ILIKE '%' || $1 || '%')
               GROUP BY r.id, r.name, r.code, r.description, r.role_type, r.level, r.status, r.is_default, r.created_at
               ORDER BY r.sort_order, r.created_at DESC
               LIMIT $2 OFFSET $3"#,
            keyword,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        // 获取总数
        let total_row = sqlx::query!(
            r"SELECT COUNT(*) as count FROM roles
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')",
            keyword,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let roles: Vec<RoleListItem> = rows
            .into_iter()
            .map(|row| RoleListItem {
                id: row.id,
                name: row.name,
                code: row.code,
                description: row.description,
                role_type: row.role_type,
                level: row.level,
                status: row.status,
                is_default: row.is_default,
                user_count: row.user_count,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedRoles { roles, total })
    }

    /// 获取角色的权限列表
    pub async fn get_permissions(
        &self,
        role_id: i64,
    ) -> Result<Vec<Permission>, RoleRepositoryError> {
        let rows = sqlx::query_as!(
            Permission,
            r#"SELECT p.id, p.name, p.code,
                      COALESCE(p.permission_type, '') AS "permission_type!",
                      p.parent_id,
                      COALESCE(p.path, '') AS "path!",
                      COALESCE(p.method, '') AS "method!",
                      COALESCE(p.icon, '') AS "icon!",
                      COALESCE(p.sort_order, 0) AS "sort_order!",
                      COALESCE(p.status, 1) AS "status!"
               FROM permissions p
               INNER JOIN role_permissions rp ON p.id = rp.permission_id
               WHERE rp.role_id = $1
               ORDER BY p.sort_order"#,
            role_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let permissions: Vec<Permission> = rows
            .into_iter()
            .map(|row| Permission {
                id: row.id,
                name: row.name,
                code: row.code,
                permission_type: row.permission_type,
                parent_id: row.parent_id,
                path: row.path,
                method: row.method,
                icon: row.icon,
                sort_order: row.sort_order,
                status: row.status,
            })
            .collect();

        Ok(permissions)
    }

    /// 根据权限 code 列表查询对应的 ID（用于 set_role_permissions）
    pub async fn resolve_permission_ids(
        &self,
        codes: &[String],
    ) -> Result<Vec<i64>, RoleRepositoryError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query!(
            "SELECT id FROM permissions WHERE code = ANY($1)",
            codes,
        )
        .fetch_all(&self.pool)
        .await?;

        let ids: Vec<i64> = rows.into_iter().map(|r| r.id).collect();
        Ok(ids)
    }

    /// 设置角色权限
    pub async fn set_permissions(
        &self,
        role_id: i64,
        permission_ids: &[i64],
    ) -> Result<(), RoleRepositoryError> {
        // 使用事务
        let mut tx = self.pool.begin().await?;

        // 删除现有权限
        sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1", role_id)
            .execute(&mut *tx)
            .await?;

        // 插入新权限
        for perm_id in permission_ids {
            sqlx::query!(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)",
                role_id,
                perm_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 获取角色下的用户列表
    pub async fn get_users(
        &self,
        role_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<i64>, i64), RoleRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query!(
            "SELECT user_id FROM user_roles WHERE role_id = $1\n               ORDER BY created_at DESC\n               LIMIT $2 OFFSET $3",
            role_id,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!("SELECT COUNT(*) as count FROM user_roles WHERE role_id = $1", role_id)
            .fetch_one(&self.pool)
            .await?;

        let user_ids: Vec<i64> = rows.into_iter().map(|row| row.user_id).collect();
        let total = total_row.count.unwrap_or(0);

        Ok((user_ids, total))
    }

    /// 获取所有权限（树形）
    pub async fn list_permissions(&self) -> Result<Vec<Permission>, RoleRepositoryError> {
        let rows = sqlx::query_as!(
            Permission,
            r#"SELECT id, name, code,
                      COALESCE(permission_type, '') AS "permission_type!",
                      parent_id,
                      COALESCE(path, '') AS "path!",
                      COALESCE(method, '') AS "method!",
                      COALESCE(icon, '') AS "icon!",
                      COALESCE(sort_order, 0) AS "sort_order!",
                      COALESCE(status, 1) AS "status!"
               FROM permissions
               ORDER BY sort_order, id"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let permissions: Vec<Permission> = rows
            .into_iter()
            .map(|row| Permission {
                id: row.id,
                name: row.name,
                code: row.code,
                permission_type: row.permission_type,
                parent_id: row.parent_id,
                path: row.path,
                method: row.method,
                icon: row.icon,
                sort_order: row.sort_order,
                status: row.status,
            })
            .collect();

        Ok(permissions)
    }

    /// 复制角色权限
    pub async fn copy_permissions(
        &self,
        source_role_id: i64,
        target_role_ids: &[i64],
    ) -> Result<(), RoleRepositoryError> {
        let mut tx = self.pool.begin().await?;

        // 获取源角色的权限
        let rows = sqlx::query!(
            "SELECT permission_id FROM role_permissions WHERE role_id = $1",
            source_role_id,
        )
        .fetch_all(&mut *tx)
        .await?;

        let permission_ids: Vec<i64> = rows
            .into_iter()
            .map(|row| row.permission_id)
            .collect();

        // 为每个目标角色设置权限
        for target_id in target_role_ids {
            // 删除现有权限
            sqlx::query!(
                "DELETE FROM role_permissions WHERE role_id = $1",
                target_id,
            )
            .execute(&mut *tx)
            .await?;

            // 复制权限
            for perm_id in &permission_ids {
                sqlx::query!(
                    "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)",
                    target_id,
                    perm_id,
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// 角色模板 CRUD
    /// 创建角色模板
    pub async fn create_template(
        &self,
        name: &str,
        description: Option<String>,
        permissions: Vec<String>,
        role_type: Option<String>,
    ) -> Result<i64, RoleRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO role_templates (name, description, permissions, role_type)
               VALUES ($1, $2, $3, $4)
               RETURNING id",
            name,
            description.as_deref(),
            serde_json::to_value(&permissions).unwrap_or_default(),
            role_type.as_deref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 获取角色模板列表
    pub async fn list_templates(&self) -> Result<Vec<RoleTemplate>, RoleRepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT id, name, description,
                      COALESCE(permissions, '[]'::jsonb) AS permissions,
                      COALESCE(role_type, '') AS role_type,
                      COALESCE(is_system, false) AS is_system,
                      COALESCE(created_at, NOW()) AS created_at
               FROM role_templates
               ORDER BY is_system DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let templates: Vec<RoleTemplate> = rows
            .into_iter()
            .map(|row| {
                let perms: Vec<String> =
                    serde_json::from_value(row.permissions.unwrap_or_default()).unwrap_or_default();

                RoleTemplate {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                    permissions: perms,
                    role_type: row.role_type,
                    is_system: row.is_system.unwrap_or(false),
                    created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
                }
            })
            .collect();

        Ok(templates)
    }

    /// 更新角色模板
    pub async fn update_template(
        &self,
        id: i64,
        name: Option<String>,
        description: Option<String>,
        permissions: Option<Vec<String>>,
    ) -> Result<bool, RoleRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE role_templates
               SET name = COALESCE($1, name),
                   description = COALESCE($2, description),
                   permissions = COALESCE($3, permissions),
                   updated_at = NOW()
               WHERE id = $4 AND is_system = FALSE",
            name.as_deref(),
            description.as_deref(),
            permissions.map(|p| serde_json::to_value(&p).unwrap_or_default()),
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除角色模板
    pub async fn delete_template(&self, id: i64) -> Result<bool, RoleRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM role_templates WHERE id = $1 AND is_system = FALSE",
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ============ 数据权限 ============

    /// 获取角色的数据权限配置
    pub async fn get_data_permissions(
        &self,
        role_id: i64,
    ) -> Result<Vec<serde_json::Value>, RoleRepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT id, role_id, resource_type, data_scope, filter_expression,
                      allowed_department_ids, allowed_user_ids, priority, enabled
               FROM role_data_permissions
               WHERE role_id = $1
               ORDER BY priority DESC, id"#,
            role_id.to_string(),
        )
        .fetch_all(&self.pool)
        .await?;

        let result = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.id,
                "role_id": row.role_id,
                "resource_type": row.resource_type,
                "data_scope": row.data_scope,
                "filter_expression": row.filter_expression,
                "allowed_department_ids": row.allowed_department_ids,
                "allowed_user_ids": row.allowed_user_ids,
                "priority": row.priority,
                "enabled": row.enabled,
            })
        }).collect();

        Ok(result)
    }

    /// 设置角色的数据权限配置
    pub async fn set_data_permissions(
        &self,
        role_id: i64,
        permissions: &[serde_json::Value],
    ) -> Result<(), RoleRepositoryError> {
        // 删除现有数据权限
        sqlx::query!(
            "DELETE FROM role_data_permissions WHERE role_id = $1",
            role_id.to_string(),
        )
        .execute(&self.pool)
        .await?;

        // 插入新数据权限
        for perm in permissions {
            let resource_type = perm
                .get("resource_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let data_scope = perm
                .get("data_scope")
                .and_then(|v| v.as_str())
                .unwrap_or("all");
            let filter_expression = perm.get("filter_expression").and_then(|v| v.as_str());
            let allowed_department_ids = perm.get("allowed_department_ids");
            let allowed_user_ids = perm.get("allowed_user_ids");
            let priority = perm.get("priority").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32;
            let enabled = perm
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);

            sqlx::query!(
                r"INSERT INTO role_data_permissions
                   (id, role_id, resource_type, data_scope, filter_expression,
                    allowed_department_ids, allowed_user_ids, priority, enabled)
                   VALUES (gen_random_uuid()::text, $1, $2, $3, $4, $5, $6, $7, $8)",
                role_id.to_string(),
                resource_type,
                data_scope,
                filter_expression,
                allowed_department_ids,
                allowed_user_ids,
                priority,
                enabled,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // ============ 字段权限 ============

    /// 获取角色的字段权限配置
    pub async fn get_field_permissions(
        &self,
        role_id: i64,
    ) -> Result<Vec<serde_json::Value>, RoleRepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT id, role_id, resource_type, field_name, permission, mask_pattern
               FROM role_field_permissions
               WHERE role_id = $1
               ORDER BY id"#,
            role_id.to_string(),
        )
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "role_id": row.role_id,
                    "resource_type": row.resource_type,
                    "field_name": row.field_name,
                    "permission": row.permission,
                    "mask_pattern": row.mask_pattern,
                })
            })
            .collect();

        Ok(result)
    }

    /// 设置角色的字段权限配置
    pub async fn set_field_permissions(
        &self,
        role_id: i64,
        permissions: &[serde_json::Value],
    ) -> Result<(), RoleRepositoryError> {
        // 删除现有字段权限
        sqlx::query!(
            "DELETE FROM role_field_permissions WHERE role_id = $1",
            role_id.to_string(),
        )
        .execute(&self.pool)
        .await?;

        // 插入新字段权限
        for perm in permissions {
            let resource_type = perm
                .get("resource_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let field_name = perm
                .get("field_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let permission = perm
                .get("permission")
                .and_then(|v| v.as_str())
                .unwrap_or("read_write");
            let mask_pattern = perm.get("mask_pattern").and_then(|v| v.as_str());

            sqlx::query!(
                r"INSERT INTO role_field_permissions
                   (id, role_id, resource_type, field_name, permission, mask_pattern)
                   VALUES (gen_random_uuid()::text, $1, $2, $3, $4, $5)",
                role_id.to_string(),
                resource_type,
                field_name,
                permission,
                mask_pattern,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // ============ 继承权限 ============

    /// 获取角色的继承链
    pub async fn get_inherit_chain(
        &self,
        role_id: i64,
    ) -> Result<Vec<serde_json::Value>, RoleRepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT pi.id, pi.parent_role_id, pi.child_role_id,
                      pi.inherit_data_permissions, pi.inherit_field_permissions,
                      pi.override_child_permissions, pi.priority,
                      r.name as parent_role_name, r.code as parent_role_code
               FROM permission_inheritances pi
               JOIN roles r ON r.id::text = pi.parent_role_id
               WHERE pi.child_role_id = $1::text
               ORDER BY pi.priority DESC"#,
            role_id.to_string(),
        )
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "parent_role_id": row.parent_role_id,
                    "child_role_id": row.child_role_id,
                    "parent_role_name": row.parent_role_name,
                    "parent_role_code": row.parent_role_code,
                    "inherit_data_permissions": row.inherit_data_permissions,
                    "inherit_field_permissions": row.inherit_field_permissions,
                    "override_child_permissions": row.override_child_permissions,
                    "priority": row.priority,
                })
            })
            .collect();

        Ok(result)
    }

    /// 设置角色的继承关系
    pub async fn set_inherit(
        &self,
        role_id: i64,
        inherit_from: &[String],
    ) -> Result<(), RoleRepositoryError> {
        // 删除现有继承关系
        sqlx::query!(
            "DELETE FROM permission_inheritances WHERE child_role_id = $1::text",
            role_id.to_string(),
        )
        .execute(&self.pool)
        .await?;

        // 创建新的继承关系
        for (idx, parent_code) in inherit_from.iter().enumerate() {
            if let Some(parent) = self.find_by_code(parent_code).await? {
                sqlx::query!(
                    r"INSERT INTO permission_inheritances
                       (id, parent_role_id, child_role_id, inherit_data_permissions,
                        inherit_field_permissions, override_child_permissions, priority)
                       VALUES (gen_random_uuid()::text, $1, $2, TRUE, TRUE, FALSE, $3)",
                    parent.id.to_string(),
                    role_id.to_string(),
                    idx as i32,
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// 移除角色的继承关系
    pub async fn remove_inherit(&self, role_id: i64) -> Result<bool, RoleRepositoryError> {
        let result =
            sqlx::query!(
                "DELETE FROM permission_inheritances WHERE child_role_id = $1::text",
                role_id.to_string(),
            )
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 应用角色模板到角色
    pub async fn apply_template(
        &self,
        template_id: i64,
        role_code: &str,
    ) -> Result<(), RoleRepositoryError> {
        let role = self
            .find_by_code(role_code)
            .await?
            .ok_or(RoleRepositoryError::NotFound)?;

        let template_rows = sqlx::query!(
            "SELECT permissions FROM role_templates WHERE id = $1",
            template_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = template_rows {
            let perms: Vec<String> =
                serde_json::from_value(row.permissions.unwrap_or_default()).unwrap_or_default();

            // 获取权限ID
            let mut permission_ids = Vec::new();
            for code in perms {
                let perm_row = sqlx::query!(
                    "SELECT id FROM permissions WHERE code = $1",
                    code,
                )
                .fetch_optional(&self.pool)
                .await?;

                if let Some(pr) = perm_row {
                    permission_ids.push(pr.id);
                }
            }

            // 设置角色权限
            self.set_permissions(role.id, &permission_ids).await?;
        }

        Ok(())
    }
}
