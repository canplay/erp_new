//! 数据权限操作
use super::RoleRepository;
use super::types::*;
use super::types::RoleRepositoryError;

impl RoleRepository {
    // ============ 数据权限 ============

    /// 获取角色的数据权限配置
    pub(crate) async fn get_data_permissions(
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

    /// 设置角色的数据权限配置（N+1 修复：批量 INSERT + 事务）
    pub(crate) async fn set_data_permissions(
        &self,
        role_id: i64,
        permissions: &[serde_json::Value],
    ) -> Result<(), RoleRepositoryError> {
        let mut tx = self.pool.begin().await?;

        // 删除现有数据权限
        sqlx::query!(
            "DELETE FROM role_data_permissions WHERE role_id = $1" ,
            role_id.to_string(),
        )
        .execute(&mut *tx)
        .await?;

        // N+1 修复：使用 UNNEST 批量 INSERT，替代逐条插入
        if !permissions.is_empty() {
            let resource_types: Vec<&str> = permissions.iter()
                .map(|p| p.get("resource_type").and_then(|v| v.as_str()).unwrap_or(""))
                .collect();
            let data_scopes: Vec<&str> = permissions.iter()
                .map(|p| p.get("data_scope").and_then(|v| v.as_str()).unwrap_or("all"))
                .collect();
            let filter_expressions: Vec<Option<&str>> = permissions.iter()
                .map(|p| p.get("filter_expression").and_then(|v| v.as_str()))
                .collect();
            let priorities: Vec<i32> = permissions.iter()
                .map(|p| p.get("priority").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32)
                .collect();
            let enableds: Vec<bool> = permissions.iter()
                .map(|p| p.get("enabled").and_then(serde_json::Value::as_bool).unwrap_or(true))
                .collect();

            sqlx::query(
                r#"INSERT INTO role_data_permissions
                   (id, role_id, resource_type, data_scope, filter_expression,
                    allowed_department_ids, allowed_user_ids, priority, enabled)
                   SELECT gen_random_uuid()::text, $1, u.resource_type, u.data_scope,
                          u.filter_expression, u.allowed_department_ids, u.allowed_user_ids,
                          u.priority, u.enabled
                   FROM UNNEST($2::text[], $3::text[], $4::text[], $5::int4[], $6::bool[])
                   AS u(resource_type, data_scope, filter_expression, priority, enabled)"#,
            )
            .bind(role_id.to_string())
            .bind(&resource_types[..])
            .bind(&data_scopes[..])
            .bind(&filter_expressions[..])
            .bind(&priorities[..])
            .bind(&enableds[..])
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

}
