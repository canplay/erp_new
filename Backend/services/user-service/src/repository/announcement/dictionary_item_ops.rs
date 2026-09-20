//! 数据字典项管理操作
use super::AnnouncementRepository;
use super::types::*;

impl AnnouncementRepository {
    // ==================== 数据字典项管理 ====================

    /// 分页查询字典项
    pub(crate) async fn list_dictionary_items(
        &self,
        type_id: Option<i64>,
        type_code: Option<&str>,
        keyword: Option<&str>,
        status: Option<i32>,
    ) -> Result<PaginatedDictionaryItems, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            DictionaryItem,
            r#"SELECT di.id, di.type_id, di.label, di.value,
                      COALESCE(di.sort, 0) AS "sort!" ,
                      COALESCE(di.status, 1) AS "status!" ,
                      COALESCE(di.is_default, false) AS "is_default!" ,
                      di.remark,
                      COALESCE(di.created_at, NOW()) AS "created_at!" ,
                      COALESCE(di.updated_at, NOW()) AS "updated_at!"
               FROM dictionary_items di
               JOIN dictionary_types dt ON di.type_id = dt.id
               WHERE ($1::bigint IS NULL OR di.type_id = $1)
                 AND ($2::text IS NULL OR dt.code = $2)
                 AND ($3::text IS NULL OR di.label ILIKE '%' || $3 || '%' OR di.value ILIKE '%' || $3 || '%')
                 AND ($4::integer IS NULL OR di.status = $4)
               ORDER BY di.sort, di.created_at DESC"#,
            type_id,
            type_code,
            keyword,
            status,
        )
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query!(
            r"SELECT COUNT(*) as count
               FROM dictionary_items di
               JOIN dictionary_types dt ON di.type_id = dt.id
               WHERE ($1::bigint IS NULL OR di.type_id = $1)
                 AND ($2::text IS NULL OR dt.code = $2)
                 AND ($3::text IS NULL OR di.label ILIKE '%' || $3 || '%' OR di.value ILIKE '%' || $3 || '%')
                 AND ($4::integer IS NULL OR di.status = $4)" ,
            type_id,
            type_code,
            keyword,
            status,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let items: Vec<DictionaryItem> = rows
            .into_iter()
            .map(|row| DictionaryItem {
                id: row.id,
                type_id: row.type_id,
                label: row.label,
                value: row.value,
                sort: row.sort,
                status: row.status,
                is_default: row.is_default,
                remark: row.remark,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PaginatedDictionaryItems { items, total })
    }

    /// 根据 ID 获取字典项
    pub(crate) async fn find_dictionary_item_by_id(
        &self,
        id: i64,
    ) -> Result<Option<DictionaryItem>, AnnouncementRepositoryError> {
        let row = sqlx::query_as!(
            DictionaryItem,
            r#"SELECT id, type_id, label, value,
                      COALESCE(sort, 0) AS "sort!" ,
                      COALESCE(status, 1) AS "status!" ,
                      COALESCE(is_default, false) AS "is_default!" ,
                      remark,
                      COALESCE(created_at, NOW()) AS "created_at!" ,
                      COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM dictionary_items WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DictionaryItem {
            id: r.id,
            type_id: r.type_id,
            label: r.label,
            value: r.value,
            sort: r.sort,
            status: r.status,
            is_default: r.is_default,
            remark: r.remark,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 创建字典项
    pub(crate) async fn create_dictionary_item(
        &self,
        params: CreateDictionaryItemParams<'_>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO dictionary_items (type_id, label, value, sort, status, is_default, remark)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id" ,
            params.type_id,
            params.label,
            params.value,
            params.sort,
            params.status,
            params.is_default,
            params.remark,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 更新字典项
    pub(crate) async fn update_dictionary_item(
        &self,
        params: UpdateDictionaryItemParams<'_>,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE dictionary_items
               SET label = COALESCE($1, label),
                   value = COALESCE($2, value),
                   sort = COALESCE($3, sort),
                   status = COALESCE($4, status),
                   is_default = COALESCE($5, is_default),
                   remark = COALESCE($6, remark),
                   updated_at = NOW()
               WHERE id = $7" ,
            params.label,
            params.value,
            params.sort,
            params.status,
            params.is_default,
            params.remark,
            params.id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除字典项
    pub(crate) async fn delete_dictionary_item(
        &self,
        id: i64,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!("DELETE FROM dictionary_items WHERE id = $1" , id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

}
