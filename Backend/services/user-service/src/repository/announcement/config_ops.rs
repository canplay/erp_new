//! 系统配置管理操作
use super::AnnouncementRepository;
use super::types::*;

impl AnnouncementRepository {
    // ==================== 系统配置管理 ====================

    /// 获取所有配置（按分类）
    pub(crate) async fn get_all_configs(&self) -> Result<Vec<SystemConfig>, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            SystemConfig,
            r#"SELECT id, category, config_key, config_value,
                      COALESCE(value_type, '') AS "value_type!" ,
                      COALESCE(label, '') AS "label!" ,
                      description,
                      COALESCE(sort_order, 0) AS "sort_order!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM system_configs
               WHERE status = 1
               ORDER BY category, sort_order"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let configs: Vec<SystemConfig> = rows
            .into_iter()
            .map(|row| SystemConfig {
                id: row.id,
                category: row.category,
                config_key: row.config_key,
                config_value: row.config_value,
                value_type: row.value_type,
                label: row.label,
                description: row.description,
                sort_order: row.sort_order,
                status: row.status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(configs)
    }

    /// 获取单个配置
    pub(crate) async fn get_config(
        &self,
        key: &str,
    ) -> Result<Option<SystemConfig>, AnnouncementRepositoryError> {
        let row = sqlx::query_as!(
            SystemConfig,
            r#"SELECT id, category, config_key, config_value,
                      COALESCE(value_type, '') AS "value_type!" ,
                      COALESCE(label, '') AS "label!" ,
                      description,
                      COALESCE(sort_order, 0) AS "sort_order!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM system_configs
               WHERE config_key = $1"#,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SystemConfig {
            id: r.id,
            category: r.category,
            config_key: r.config_key,
            config_value: r.config_value,
            value_type: r.value_type,
            label: r.label,
            description: r.description,
            sort_order: r.sort_order,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 更新配置
    pub(crate) async fn update_config(
        &self,
        key: &str,
        value: &str,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            "UPDATE system_configs SET config_value = $1, updated_at = NOW() WHERE config_key = $2" ,
            value,
            key,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 重置配置到默认值（从初始数据重新获取）
    pub(crate) async fn reset_config(&self, key: &str) -> Result<bool, AnnouncementRepositoryError> {
        // 这里假设有默认值存储，简化处理：设置为 NULL
        let result = sqlx::query!(
            "UPDATE system_configs SET config_value = NULL, updated_at = NOW() WHERE config_key = $1" ,
            key,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== 系统配置增强 ====================

    /// 获取系统配置列表（全量，支持分类过滤）
    pub(crate) async fn list_system_configs(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<SystemConfig>, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            SystemConfig,
            r#"SELECT id, category, config_key, config_value,
                      COALESCE(value_type, '') AS "value_type!" ,
                      COALESCE(label, '') AS "label!" ,
                      description,
                      COALESCE(sort_order, 0) AS "sort_order!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM system_configs
               WHERE ($1::text IS NULL OR category = $1)
                 AND status = 1
               ORDER BY category, sort_order"#,
            category,
        )
        .fetch_all(&self.pool)
        .await?;

        let configs: Vec<SystemConfig> = rows
            .into_iter()
            .map(|row| SystemConfig {
                id: row.id,
                category: row.category,
                config_key: row.config_key,
                config_value: row.config_value,
                value_type: row.value_type,
                label: row.label,
                description: row.description,
                sort_order: row.sort_order,
                status: row.status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(configs)
    }

    /// 批量更新系统配置（N+1 修复：使用 UPDATE ... FROM UNNEST）
    pub(crate) async fn batch_update_system_configs(
        &self,
        configs: &[(String, String)],
    ) -> Result<bool, AnnouncementRepositoryError> {
        if configs.is_empty() {
            return Ok(true);
        }

        let keys: Vec<&str> = configs.iter().map(|(k, _)| k.as_str()).collect();
        let values: Vec<&str> = configs.iter().map(|(_, v)| v.as_str()).collect();

        sqlx::query(
            r#"UPDATE system_configs AS sc
               SET config_value = u.value, updated_at = NOW()
               FROM UNNEST($1::text[], $2::text[]) AS u(key, value)
               WHERE sc.config_key = u.key"#,
        )
        .bind(&keys[..])
        .bind(&values[..])
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

}
