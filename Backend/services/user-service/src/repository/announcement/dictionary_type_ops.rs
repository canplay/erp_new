//! 数据字典类型管理操作
use super::AnnouncementRepository;
use super::types::*;

impl AnnouncementRepository {
    // ==================== 数据字典类型管理 ====================

    /// 分页查询字典类型
    pub(crate) async fn list_dictionary_types(
        &self,
        page: i32,
        page_size: i32,
        keyword: Option<&str>,
        status: Option<i32>,
    ) -> Result<PaginatedDictionaryTypes, AnnouncementRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            DictionaryType,
            r#"SELECT id, code, name, description,
                      COALESCE(sort, 0) AS "sort!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM dictionary_types
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')
                 AND ($2::integer IS NULL OR status = $2)
               ORDER BY sort, created_at DESC
               LIMIT $3 OFFSET $4"#,
            keyword,
            status,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!(
            r"SELECT COUNT(*) as count FROM dictionary_types
               WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR code ILIKE '%' || $1 || '%')
                 AND ($2::integer IS NULL OR status = $2)" ,
            keyword,
            status,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let types: Vec<DictionaryType> = rows
            .into_iter()
            .map(|row| DictionaryType {
                id: row.id,
                code: row.code,
                name: row.name,
                description: row.description,
                sort: row.sort,
                status: row.status,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PaginatedDictionaryTypes { types, total })
    }

    /// 根据 ID 获取字典类型
    pub(crate) async fn find_dictionary_type_by_id(
        &self,
        id: i64,
    ) -> Result<Option<DictionaryType>, AnnouncementRepositoryError> {
        let row = sqlx::query_as!(
            DictionaryType,
            r#"SELECT id, code, name, description,
                      COALESCE(sort, 0) AS "sort!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM dictionary_types WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DictionaryType {
            id: r.id,
            code: r.code,
            name: r.name,
            description: r.description,
            sort: r.sort,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 创建字典类型
    pub(crate) async fn create_dictionary_type(
        &self,
        code: &str,
        name: &str,
        description: Option<&str>,
        sort: Option<i32>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO dictionary_types (code, name, description, sort, status)
               VALUES ($1, $2, $3, $4, 1)
               RETURNING id" ,
            code,
            name,
            description,
            sort,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 更新字典类型
    pub(crate) async fn update_dictionary_type(
        &self,
        id: i64,
        name: Option<String>,
        description: Option<String>,
        sort: Option<i32>,
        status: Option<i32>,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE dictionary_types
               SET name = COALESCE($1, name),
                   description = COALESCE($2, description),
                   sort = COALESCE($3, sort),
                   status = COALESCE($4, status),
                   updated_at = NOW()
               WHERE id = $5" ,
             name.as_deref(),
             description.as_deref(),
             sort,
            status,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除字典类型
    pub(crate) async fn delete_dictionary_type(&self, id: i64) -> Result<bool, AnnouncementRepositoryError> {
        // 先删除该类型下的所有字典项
        sqlx::query!("DELETE FROM dictionary_items WHERE type_id = $1" , id)
            .execute(&self.pool)
            .await?;

        let result = sqlx::query!("DELETE FROM dictionary_types WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

}
