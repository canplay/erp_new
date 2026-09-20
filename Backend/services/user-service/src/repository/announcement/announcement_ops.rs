//! 公告管理操作
use super::AnnouncementRepository;
use super::types::*;

impl AnnouncementRepository {
    // ==================== 公告管理 ====================

    /// 创建公告
    pub(crate) async fn create(
        &self,
        params: CreateAnnouncementParams<'_>,
    ) -> Result<i64, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r"INSERT INTO announcements
               (title, content, announcement_type, priority, is_pinned, is_active, start_time, end_time, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id" ,
            params.title,
            params.content,
            params.announcement_type,
            params.priority,
            params.is_pinned,
            params.is_active,
            params.start_time,
            params.end_time,
            params.created_by,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// 获取公告详情
    pub(crate) async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Announcement>, AnnouncementRepositoryError> {
        let row = sqlx::query!(
            r#"SELECT a.id, a.title, a.content,
                      COALESCE(a.announcement_type, '') AS "announcement_type!" ,
                      COALESCE(a.priority, 0) AS "priority!" ,
                      COALESCE(a.is_pinned, false) AS "is_pinned!" ,
                      COALESCE(a.is_active, false) AS "is_active!" ,
                      a.start_time, a.end_time,
                      a.created_by,
                      COALESCE(a.created_at, NOW()) AS "created_at!" ,
                      COALESCE(a.updated_at, NOW()) AS "updated_at!" ,
                      u.nickname as created_by_name
               FROM announcements a
               LEFT JOIN users u ON a.created_by = u.id
               WHERE a.id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Announcement {
            id: r.id,
            title: r.title,
            content: r.content,
            announcement_type: r.announcement_type,
            priority: r.priority,
            is_pinned: r.is_pinned,
            is_active: r.is_active,
            start_time: r.start_time,
            end_time: r.end_time,
            created_by: r.created_by,
            created_by_name: r.created_by_name,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// 更新公告
    pub(crate) async fn update(
        &self,
        params: UpdateAnnouncementParams,
    ) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            r"UPDATE announcements
               SET title = COALESCE($1, title),
                   content = COALESCE($2, content),
                   announcement_type = COALESCE($3, announcement_type),
                   priority = COALESCE($4, priority),
                   is_pinned = COALESCE($5, is_pinned),
                   is_active = COALESCE($6, is_active),
                   start_time = COALESCE($7, start_time),
                   end_time = COALESCE($8, end_time),
                   updated_at = NOW()
               WHERE id = $9" ,
            params.title.as_deref(),
            params.content.as_deref(),
            params.announcement_type.as_deref(),
            params.priority,
            params.is_pinned,
            params.is_active,
            params.start_time,
            params.end_time,
            params.id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除公告
    pub(crate) async fn delete(&self, id: i64) -> Result<bool, AnnouncementRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM announcements WHERE id = $1" ,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页查询公告列表
    pub(crate) async fn list(
        &self,
        page: i32,
        page_size: i32,
        is_active: Option<bool>,
    ) -> Result<PaginatedAnnouncements, AnnouncementRepositoryError> {
        let offset = (page - 1).max(0) * page_size;

        let rows = sqlx::query_as!(
            AnnouncementListItem,
            r#"SELECT a.id, a.title,
                      COALESCE(a.announcement_type, '') AS "announcement_type!" ,
                      COALESCE(a.priority, 0) AS "priority!" ,
                      COALESCE(a.is_pinned, false) AS "is_pinned!" ,
                      COALESCE(a.is_active, false) AS "is_active!" ,
                      a.start_time, a.end_time,
                      COALESCE(a.created_at, NOW()) AS "created_at!" ,
                      u.nickname as created_by_name
               FROM announcements a
               LEFT JOIN users u ON a.created_by = u.id
               WHERE ($1::boolean IS NULL OR a.is_active = $1)
               ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC
               LIMIT $2 OFFSET $3"#,
            is_active,
            i64::from(page_size),
            i64::from(offset),
        )
        .fetch_all(&self.pool)
        .await?;

        // 获取总数
        let total_row = sqlx::query!(
            "SELECT COUNT(*) as count FROM announcements WHERE ($1::boolean IS NULL OR is_active = $1)" ,
            is_active,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = total_row.count.unwrap_or(0);

        let announcements: Vec<AnnouncementListItem> = rows
            .into_iter()
            .map(|row| AnnouncementListItem {
                id: row.id,
                title: row.title,
                announcement_type: row.announcement_type,
                priority: row.priority,
                is_pinned: row.is_pinned,
                is_active: row.is_active,
                start_time: row.start_time,
                end_time: row.end_time,
                created_by_name: row.created_by_name,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedAnnouncements {
            announcements,
            total,
        })
    }

    /// 获取活跃公告（公开接口）
    pub(crate) async fn get_active(
        &self,
    ) -> Result<Vec<AnnouncementListItem>, AnnouncementRepositoryError> {
        let rows = sqlx::query_as!(
            AnnouncementListItem,
            r#"SELECT a.id, a.title,
                      COALESCE(a.announcement_type, '') AS "announcement_type!" ,
                      COALESCE(a.priority, 0) AS "priority!" ,
                      COALESCE(a.is_pinned, false) AS "is_pinned!" ,
                      COALESCE(a.is_active, false) AS "is_active!" ,
                      a.start_time, a.end_time,
                      COALESCE(a.created_at, NOW()) AS "created_at!" ,
                      u.nickname as created_by_name
               FROM announcements a
               LEFT JOIN users u ON a.created_by = u.id
               WHERE a.is_active = TRUE
                 AND (a.start_time IS NULL OR a.start_time <= NOW())
                 AND (a.end_time IS NULL OR a.end_time >= NOW())
               ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let announcements: Vec<AnnouncementListItem> = rows
            .into_iter()
            .map(|row| AnnouncementListItem {
                id: row.id,
                title: row.title,
                announcement_type: row.announcement_type,
                priority: row.priority,
                is_pinned: row.is_pinned,
                is_active: row.is_active,
                start_time: row.start_time,
                end_time: row.end_time,
                created_by_name: row.created_by_name,
                created_at: row.created_at,
            })
            .collect();

        Ok(announcements)
    }

}
