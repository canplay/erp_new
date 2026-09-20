//! 继承权限操作
use sqlx::Row;
use super::RoleRepository;
use super::types::RoleRepositoryError;

impl RoleRepository {
    // ============ 继承权限 ============

    /// 获取角色的继承链
    pub(crate) async fn get_inherit_chain(
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

    /// 设置角色的继承关系（N+1 修复：WHERE code = ANY($1)）
    pub(crate) async fn set_inherit(
        &self,
        role_id: i64,
        inherit_from: &[String],
    ) -> Result<(), RoleRepositoryError> {
        let mut tx = self.pool.begin().await?;

        // 删除现有继承关系
        sqlx::query!(
            "DELETE FROM permission_inheritances WHERE child_role_id = $1::text" ,
            role_id.to_string(),
        )
        .execute(&mut *tx)
        .await?;

        // N+1 修复：使用 WHERE code = ANY($1) 批量查询，替代循环内逐条 find_by_code
        if !inherit_from.is_empty() {
            let rows = sqlx::query(
                "SELECT id FROM roles WHERE code = ANY($1)" ,
            )
            .bind(inherit_from)
            .fetch_all(&mut *tx)
            .await?;

            for (idx, row) in rows.iter().enumerate() {
                let parent_id: i64 = row.get("id");
                sqlx::query(
                    r#"INSERT INTO permission_inheritances
                       (id, parent_role_id, child_role_id, inherit_data_permissions,
                        inherit_field_permissions, override_child_permissions, priority)
                       VALUES (gen_random_uuid()::text, $1, $2, TRUE, TRUE, FALSE, $3)"#,
                )
                .bind(parent_id.to_string())
                .bind(role_id.to_string())
                .bind(idx as i32)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// 移除角色的继承关系
    pub(crate) async fn remove_inherit(&self, role_id: i64) -> Result<bool, RoleRepositoryError> {
        let result =
            sqlx::query!(
                "DELETE FROM permission_inheritances WHERE child_role_id = $1::text" ,
                role_id.to_string(),
            )
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 应用角色模板到角色（N+1 修复：复用 resolve_permission_ids 批量查询）
    pub(crate) async fn apply_template(
        &self,
        template_id: i64,
        role_code: &str,
    ) -> Result<(), RoleRepositoryError> {
        let role = self
            .find_by_code(role_code)
            .await?
            .ok_or(RoleRepositoryError::NotFound)?;

        let template_rows = sqlx::query!(
            "SELECT permissions FROM role_templates WHERE id = $1" ,
            template_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = template_rows {
            let perms: Vec<String> =
                serde_json::from_value(row.permissions.unwrap_or_default()).unwrap_or_default();

            // N+1 修复：复用 resolve_permission_ids 批量查询，替代循环内逐条 SELECT
            let permission_ids = self.resolve_permission_ids(&perms).await?;

            // 设置角色权限
            self.set_permissions(role.id, &permission_ids).await?;
        }

        Ok(())
    }
}
