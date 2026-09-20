//! 字段权限操作
use super::RoleRepository;
use super::types::RoleRepositoryError;

impl RoleRepository {
    // ============ 字段权限 ============

    /// 获取角色的字段权限配置
    pub(crate) async fn get_field_permissions(
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

    /// 设置角色的字段权限配置（N+1 修复：批量 INSERT + 事务）
    pub(crate) async fn set_field_permissions(
        &self,
        role_id: i64,
        permissions: &[serde_json::Value],
    ) -> Result<(), RoleRepositoryError> {
        let mut tx = self.pool.begin().await?;

        // 删除现有字段权限
        sqlx::query!(
            "DELETE FROM role_field_permissions WHERE role_id = $1" ,
            role_id.to_string(),
        )
        .execute(&mut *tx)
        .await?;

        // N+1 修复：使用 UNNEST 批量 INSERT，替代逐条插入
        if !permissions.is_empty() {
            let resource_types: Vec<&str> = permissions.iter()
                .map(|p| p.get("resource_type").and_then(|v| v.as_str()).unwrap_or(""))
                .collect();
            let field_names: Vec<&str> = permissions.iter()
                .map(|p| p.get("field_name").and_then(|v| v.as_str()).unwrap_or(""))
                .collect();
            let perms: Vec<&str> = permissions.iter()
                .map(|p| p.get("permission").and_then(|v| v.as_str()).unwrap_or("read_write"))
                .collect();
            let mask_patterns: Vec<Option<&str>> = permissions.iter()
                .map(|p| p.get("mask_pattern").and_then(|v| v.as_str()))
                .collect();

            sqlx::query(
                r#"INSERT INTO role_field_permissions
                   (id, role_id, resource_type, field_name, permission, mask_pattern)
                   SELECT gen_random_uuid()::text, $1, u.resource_type, u.field_name,
                          u.permission, u.mask_pattern
                   FROM UNNEST($2::text[], $3::text[], $4::text[], $5::text[])
                   AS u(resource_type, field_name, permission, mask_pattern)"#,
            )
            .bind(role_id.to_string())
            .bind(&resource_types[..])
            .bind(&field_names[..])
            .bind(&perms[..])
            .bind(&mask_patterns[..])
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

}
