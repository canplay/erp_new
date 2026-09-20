//! 角色与权限管理操作
use super::RoleRepository;
use super::types::*;
use super::types::RoleRepositoryError;

impl RoleRepository {
    /// 创建角色
    pub(crate) async fn create(
        &self,
        name: &str,
        code: &str,
        description: Option<String>,
        role_type: &str,
        parent_id: Option<i64>,
    ) -> Result<i64, RoleRepositoryError> {
        // 检查代码是否已存在
        let exists = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1) AS "exists!" "#,
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
                "SELECT level FROM roles WHERE id = $1" ,
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
               RETURNING id" ,
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
    pub(crate) async fn find_by_code(&self, code: &str) -> Result<Option<Role>, RoleRepositoryError> {
        let row = sqlx::query_as!(
            Role,
            r#"SELECT id, name, code, description,
                      COALESCE(role_type, 'user') AS "role_type!" ,
                      parent_id,
                      COALESCE(level, 0) AS "level!" ,
                      COALESCE(sort_order, 0) AS "sort_order!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(is_default, false) AS "is_default!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM roles WHERE code = $1"#,
            code,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 根据ID查找角色
    pub(crate) async fn find_by_id(&self, role_id: i64) -> Result<Option<Role>, RoleRepositoryError> {
        let row = sqlx::query_as!(
            Role,
            r#"SELECT id, name, code, description,
                      COALESCE(role_type, 'user') AS "role_type!" ,
                      parent_id,
                      COALESCE(level, 0) AS "level!" ,
                      COALESCE(sort_order, 0) AS "sort_order!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(is_default, false) AS "is_default!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM roles WHERE id = $1"#,
            role_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// 更新角色
    pub(crate) async fn update(
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
                         COALESCE(role_type, 'user') AS "role_type!" ,
                         parent_id,
                         COALESCE(level, 0) AS "level!" ,
                         COALESCE(sort_order, 0) AS "sort_order!" ,
                         COALESCE(status, 1) AS "status!" ,
                         COALESCE(is_default, false) AS "is_default!" ,
                         COALESCE(created_at, NOW()) AS "created_at!" ,
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
    pub(crate) async fn delete(&self, code: &str) -> Result<bool, RoleRepositoryError> {
        // 检查是否有子角色
        let role = self.find_by_code(code).await?;
        if let Some(role) = role {
            let child_count =
                sqlx::query!("SELECT COUNT(*) as count FROM roles WHERE parent_id = $1" , role.id)
                    .fetch_one(&self.pool)
                    .await?
                    .count
                    .unwrap_or(0);

            if child_count > 0 {
                return Err(RoleRepositoryError::HasChildRoles);
            }

            // 检查是否有用户关联
            let user_count =
                sqlx::query!("SELECT COUNT(*) as count FROM user_roles WHERE role_id = $1" , role.id)
                    .fetch_one(&self.pool)
                    .await?
                    .count
                    .unwrap_or(0);

            if user_count > 0 {
                return Err(RoleRepositoryError::HasAssociatedUsers);
            }

            // 删除角色权限关联
            sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1" , role.id)
                .execute(&self.pool)
                .await?;

            // 删除角色
            let result = sqlx::query!("DELETE FROM roles WHERE code = $1" , code)
                .execute(&self.pool)
                .await?;

            return Ok(result.rows_affected() > 0);
        }

        Ok(false)
    }

    /// 分页查询角色列表
    pub(crate) async fn list(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
    ) -> Result<PaginatedRoles, RoleRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            RoleListItem,
            r#"SELECT r.id, r.name, r.code, r.description,
                      COALESCE(r.role_type, 'user') AS "role_type!" ,
                      COALESCE(r.level, 0) AS "level!" ,
                      COALESCE(r.status, 1) AS "status!" ,
                      COALESCE(r.is_default, false) AS "is_default!" ,
                      COALESCE(r.created_at, NOW()) AS "created_at!" ,
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
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')" ,
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
    pub(crate) async fn get_permissions(
        &self,
        role_id: i64,
    ) -> Result<Vec<Permission>, RoleRepositoryError> {
        let rows = sqlx::query_as!(
            Permission,
            r#"SELECT p.id, p.name, p.code,
                      COALESCE(p.permission_type, '') AS "permission_type!" ,
                      p.parent_id,
                      COALESCE(p.path, '') AS "path!" ,
                      COALESCE(p.method, '') AS "method!" ,
                      COALESCE(p.icon, '') AS "icon!" ,
                      COALESCE(p.sort_order, 0) AS "sort_order!" ,
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
    pub(crate) async fn resolve_permission_ids(
        &self,
        codes: &[String],
    ) -> Result<Vec<i64>, RoleRepositoryError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query!(
            "SELECT id FROM permissions WHERE code = ANY($1)" ,
            codes,
        )
        .fetch_all(&self.pool)
        .await?;

        let ids: Vec<i64> = rows.into_iter().map(|r| r.id).collect();
        Ok(ids)
    }

    /// 设置角色权限（N+1 修复：使用 UNNEST 批量 INSERT）
    pub(crate) async fn set_permissions(
        &self,
        role_id: i64,
        permission_ids: &[i64],
    ) -> Result<(), RoleRepositoryError> {
        // 使用事务
        let mut tx = self.pool.begin().await?;

        // 删除现有权限
        sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1" , role_id)
            .execute(&mut *tx)
            .await?;

        // N+1 修复：使用 UNNEST 批量插入，替代逐条 INSERT
        if !permission_ids.is_empty() {
            sqlx::query(
                r#"INSERT INTO role_permissions (role_id, permission_id)
                   SELECT $1, unnest($2::bigint[])"#,
            )
            .bind(role_id)
            .bind(permission_ids)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 获取角色下的用户列表
    pub(crate) async fn get_users(
        &self,
        role_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<i64>, i64), RoleRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query!(
            "SELECT user_id FROM user_roles WHERE role_id = $1\n               ORDER BY created_at DESC\n               LIMIT $2 OFFSET $3" ,
            role_id,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!("SELECT COUNT(*) as count FROM user_roles WHERE role_id = $1" , role_id)
            .fetch_one(&self.pool)
            .await?;

        let user_ids: Vec<i64> = rows.into_iter().map(|row| row.user_id).collect();
        let total = total_row.count.unwrap_or(0);

        Ok((user_ids, total))
    }

    /// 获取所有权限（树形）
    pub(crate) async fn list_permissions(&self) -> Result<Vec<Permission>, RoleRepositoryError> {
        let rows = sqlx::query_as!(
            Permission,
            r#"SELECT id, name, code,
                      COALESCE(permission_type, '') AS "permission_type!" ,
                      parent_id,
                      COALESCE(path, '') AS "path!" ,
                      COALESCE(method, '') AS "method!" ,
                      COALESCE(icon, '') AS "icon!" ,
                      COALESCE(sort_order, 0) AS "sort_order!" ,
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

    /// 复制角色权限（N+1 修复：使用 INSERT ... SELECT 替代嵌套循环）
    pub(crate) async fn copy_permissions(
        &self,
        source_role_id: i64,
        target_role_ids: &[i64],
    ) -> Result<(), RoleRepositoryError> {
        let mut tx = self.pool.begin().await?;

        for target_id in target_role_ids {
            // 删除现有权限
            sqlx::query!(
                "DELETE FROM role_permissions WHERE role_id = $1" ,
                target_id,
            )
            .execute(&mut *tx)
            .await?;

            // N+1 修复：使用 INSERT ... SELECT 复制权限，替代逐条 SELECT + INSERT
            sqlx::query(
                r#"INSERT INTO role_permissions (role_id, permission_id)
                   SELECT $1, permission_id FROM role_permissions WHERE role_id = $2"#,
            )
            .bind(target_id)
            .bind(source_role_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 角色模板 CRUD
    /// 创建角色模板
    pub(crate) async fn create_template(
        &self,
        name: &str,
        description: Option<String>,
        permissions: Vec<String>,
        role_type: Option<String>,
    ) -> Result<i64, RoleRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO role_templates (name, description, permissions, role_type)
               VALUES ($1, $2, $3, $4)
               RETURNING id" ,
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
    pub(crate) async fn list_templates(&self) -> Result<Vec<RoleTemplate>, RoleRepositoryError> {
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
    pub(crate) async fn update_template(
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
               WHERE id = $4 AND is_system = FALSE" ,
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
    pub(crate) async fn delete_template(&self, id: i64) -> Result<bool, RoleRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM role_templates WHERE id = $1 AND is_system = FALSE" ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

}
