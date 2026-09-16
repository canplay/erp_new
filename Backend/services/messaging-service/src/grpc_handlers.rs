//! gRPC Service Handlers for Messaging
//!
//! 实现 message.proto 中定义的 gRPC 服务
//! 这些是辅助函数，供 API Gateway 调用

use crate::grpc_server::MessagingState;
use crate::repository::MessageRepository;

use std::sync::Arc;
use tonic::Status;

// ============== 消息相关实现 ==============

/// 消息结构（与 HTTP 接口保持一致）
#[derive(Debug, Clone)]
pub struct MessageItem {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub sender_id: Option<i64>,
    pub receiver_id: Option<i64>,
    pub message_type: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub read_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 获取消息列表
pub async fn list_messages(
    state: Arc<MessagingState>,
    user_id: i64,
    page: i32,
    page_size: i32,
    message_type: Option<String>,
    _unread_only: bool,
) -> Result<(Vec<MessageItem>, i64, i64), Status> {
    let page = if page > 0 { i64::from(page) } else { 1 };
    let page_size = if page_size > 0 { i64::from(page_size) } else { 20 };

    match state
        .message_repo
        .list_user_messages(user_id, message_type.as_deref(), page, page_size)
        .await
    {
        Ok((messages, total, unread)) => {
            let items: Vec<MessageItem> = messages
                .into_iter()
                .map(|m| MessageItem {
                    id: m.id,
                    title: m.title,
                    content: m.content,
                    sender_id: m.sender_id,
                    receiver_id: None,
                    message_type: m.msg_type,
                    is_read: false,
                    created_at: m.created_at,
                    read_at: None,
                })
                .collect();

            Ok((items, total, unread))
        }
        Err(e) => {
            tracing::error!("list_messages 查询失败: {e}" );
            Err(Status::internal(format!("Database error: {e}" )))
        }
    }
}

/// 获取消息详情
pub async fn get_message(
    state: Arc<MessagingState>,
    id: i64,
    user_id: i64,
) -> Result<Option<MessageItem>, Status> {
    match state.message_repo.get_message(id, user_id).await {
        Ok(Some(m)) => Ok(Some(MessageItem {
            id: m.id,
            title: m.title,
            content: m.content,
            sender_id: m.sender_id,
            receiver_id: None,
            message_type: m.msg_type,
            is_read: false,
            created_at: m.created_at,
            read_at: None,
        })),
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::error!(
                "get_message 查询失败: id={id}, user_id={user_id}, error={e}"
            );
            Err(Status::internal(format!("Database error: {e}" )))
        }
    }
}

/// 发送消息
pub async fn send_message(
    state: Arc<MessagingState>,
    _user_id: i64,
    sender_id: i64,
    title: String,
    content: String,
    message_type: String,
    target_user_ids: Option<Vec<i64>>,
) -> Result<(i64, i32), Status> {
    use crate::repository::Message;

    let target_ids = target_user_ids.unwrap_or_default();
    if target_ids.is_empty() {
        return Err(Status::invalid_argument("No target users specified" ));
    }

    let message = Message {
        id: 0,
        msg_type: message_type,
        title,
        content,
        sender_id: Some(sender_id),
        sender_name: None,
        priority: 0,
        attachment_urls: None,
        target_type: String::new(),
        target_ids: None,
        expire_time: None,
        created_at: chrono::Utc::now(),
    };

    match state.message_repo.send_message(&message, &target_ids).await {
        Ok(()) => Ok((0, target_ids.len() as i32)),
        Err(e) => {
            tracing::error!("send_message 插入失败: {e}" );
            Err(Status::internal(format!("Database error: {e}" )))
        }
    }
}

/// 标记消息已读
pub async fn mark_as_read(
    state: Arc<MessagingState>,
    user_id: i64,
    message_ids: Option<Vec<i64>>,
) -> Result<i32, Status> {
    match message_ids {
        Some(ids) => {
            let mut count = 0i32;
            for id in &ids {
                match state.message_repo.mark_as_read(*id, user_id).await {
                    Ok(()) => count += 1,
                    Err(e) => {
                        tracing::error!(
                            "mark_as_read 失败: id={id}, user_id={user_id}, error={e}"
                        );
                    }
                }
            }
            Ok(count)
        }
        None => match state.message_repo.mark_all_as_read(user_id).await {
            Ok(count) => Ok(count as i32),
            Err(e) => {
                tracing::error!("mark_all_as_read 失败: user_id={user_id}, error={e}" );
                Err(Status::internal(format!("Database error: {e}" )))
            }
        },
    }
}

/// 删除消息
pub async fn delete_message(
    state: Arc<MessagingState>,
    user_id: i64,
    message_ids: Vec<i64>,
) -> Result<i32, Status> {
    let mut count = 0i32;
    for id in message_ids {
        match state.message_repo.delete_message(id, user_id).await {
            Ok(()) => count += 1,
            Err(e) => {
                tracing::error!(
                    "delete_message 失败: id={id}, user_id={user_id}, error={e}"
                );
            }
        }
    }
    Ok(count)
}

/// 获取未读消息数量
pub async fn get_unread_count(
    state: Arc<MessagingState>,
    user_id: i64,
    message_type: Option<String>,
) -> Result<i64, Status> {
    match state
        .message_repo
        .get_unread_count(user_id, message_type.as_deref())
        .await
    {
        Ok(count) => Ok(count),
        Err(e) => {
            tracing::error!(
                "get_unread_count 失败: user_id={user_id}, type={message_type:?}, error={e}"
            );
            Err(Status::internal(format!("Database error: {e}" )))
        }
    }
}

// ============== 消息模板相关实现 ==============

/// 消息模板结构
#[derive(Debug, Clone)]
pub struct MessageTemplate {
    pub id: i64,
    pub name: String,
    pub template_type: String,
    pub title_template: String,
    pub content_template: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 模板列表项结构（包含总数）
pub struct TemplateListResult {
    pub templates: Vec<MessageTemplate>,
    pub total: i64,
}

/// 模板查询结果行
#[derive(Debug, Clone, sqlx::FromRow)]
struct TemplateRow {
    id: i64,
    name: String,
    template_type: String,
    title_template: String,
    content_template: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// 获取模板列表
pub async fn list_templates(
    state: Arc<MessagingState>,
    page: i32,
    page_size: i32,
    template_type: Option<String>,
) -> Result<TemplateListResult, Status> {
    let page = if page > 0 { i64::from(page) } else { 1 };
    let page_size = if page_size > 0 { i64::from(page_size) } else { 20 };

    let offset = (page - 1) * page_size;

    // 构建查询（模板类型为空时传 NULL，等价于不过滤）
    let type_filter = template_type.as_deref();

    let rows = sqlx::query_as!(
        TemplateRow,
        r#"SELECT id, name, template_type, title_template, content_template,
           COALESCE(is_active, false) AS "is_active!" ,
           COALESCE(created_at, NOW()) AS "created_at!" ,
           COALESCE(updated_at, NOW()) AS "updated_at!"
           FROM message_templates
           WHERE ($1::text IS NULL OR template_type = $1)
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
        type_filter,
        page_size,
        offset,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    let templates: Vec<MessageTemplate> = rows
        .into_iter()
        .map(|row| MessageTemplate {
            id: row.id,
            name: row.name,
            template_type: row.template_type,
            title_template: row.title_template,
            content_template: row.content_template,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    // 获取总数
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM message_templates WHERE ($1::text IS NULL OR template_type = $1)" ,
        type_filter,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| Status::internal(format!("Database error: {e}" )))?
    .unwrap_or(0);

    Ok(TemplateListResult { templates, total })
}

/// 获取单个模板
pub async fn get_template(
    state: Arc<MessagingState>,
    id: i64,
) -> Result<Option<MessageTemplate>, Status> {
    let result = sqlx::query_as!(
        TemplateRow,
        r#"SELECT id, name, template_type, title_template, content_template,
           COALESCE(is_active, false) AS "is_active!" ,
           COALESCE(created_at, NOW()) AS "created_at!" ,
           COALESCE(updated_at, NOW()) AS "updated_at!"
           FROM message_templates
           WHERE id = $1"#,
        id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    match result {
        Some(row) => Ok(Some(MessageTemplate {
            id: row.id,
            name: row.name,
            template_type: row.template_type,
            title_template: row.title_template,
            content_template: row.content_template,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })),
        None => Ok(None),
    }
}

/// 创建模板
pub async fn create_template(
    state: Arc<MessagingState>,
    name: String,
    template_type: String,
    title_template: String,
    content_template: String,
    is_active: bool,
) -> Result<i64, Status> {
    let result = sqlx::query!(
        r#"INSERT INTO message_templates (name, template_type, title_template, content_template, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
           RETURNING id"#,
        &name,
        &template_type,
        &title_template,
        &content_template,
        is_active,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    let id = result.id;

    tracing::info!("创建消息模板成功: id={id}, name={name}" );

    Ok(id)
}

/// 更新模板
pub async fn update_template(
    state: Arc<MessagingState>,
    id: i64,
    name: Option<String>,
    template_type: Option<String>,
    title_template: Option<String>,
    content_template: Option<String>,
    is_active: Option<bool>,
) -> Result<bool, Status> {
    let result = sqlx::query!(
        r#"UPDATE message_templates
           SET name = COALESCE(NULLIF($2, ''), name),
               template_type = COALESCE(NULLIF($3, ''), template_type),
               title_template = COALESCE(NULLIF($4, ''), title_template),
               content_template = COALESCE(NULLIF($5, ''), content_template),
               is_active = COALESCE($6, is_active),
               updated_at = NOW()
           WHERE id = $1
           RETURNING id"#,
        id,
        name.as_deref(),
        template_type.as_deref(),
        title_template.as_deref(),
        content_template.as_deref(),
        is_active,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    match result {
        Some(row) => {
            tracing::info!("更新消息模板成功: id={}" , row.id);
            Ok(true)
        }
        None => Ok(false),
    }
}

/// 删除模板
pub async fn delete_template(state: Arc<MessagingState>, id: i64) -> Result<bool, Status> {
    let result = sqlx::query!("DELETE FROM message_templates WHERE id = $1" , id)
        .execute(&state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}" )))?;

    let success = result.rows_affected() > 0;

    if success {
        tracing::info!("删除消息模板成功: id={id}" );
    } else {
        tracing::warn!("删除消息模板失败: id={id} 不存在" );
    }

    Ok(success)
}
